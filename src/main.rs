mod ffi;

use ffi::{
    class_addMethod, msg_send_signatures::*, objc_allocateClassPair, objc_getClass, objc_msgSend,
    objc_registerClassPair, NSPoint, NSRange, NSRect, NSSize, ObjCClass, ObjCObject, Sel,
};

use std::{
    collections::HashMap,
    ffi::{c_char, c_void, CStr, CString},
    fs::read_to_string,
    mem::transmute,
    ptr::null_mut,
    thread::spawn,
};

/// Cast objc_msgSend to the right type and call it
/// This ensures the compiler uses the correct calling convention
fn msg_send_id(obj: *mut c_void, sel: *mut c_void) -> *mut c_void {
    unsafe {
        let f: MsgSendId = transmute(objc_msgSend as *const ());
        f(obj, sel)
    }
}

fn msg_send_void_id(obj: *mut c_void, sel: *mut c_void, arg: *mut c_void) {
    unsafe {
        let f: MsgSendVoidId = transmute(objc_msgSend as *const ());
        f(obj, sel, arg)
    }
}

fn msg_send_void_int(obj: *mut c_void, sel: *mut c_void, arg: i32) {
    unsafe {
        let f: MsgSendVoidInt = transmute(objc_msgSend as *const ());
        f(obj, sel, arg)
    }
}

fn msg_send_id_rect(obj: *mut c_void, sel: *mut c_void, rect: NSRect) -> *mut c_void {
    unsafe {
        let f: MsgSendIdRect = transmute(objc_msgSend as *const ());
        f(obj, sel, rect)
    }
}

fn msg_send_id_rect_int_int_int(
    obj: *mut c_void,
    sel: *mut c_void,
    rect: NSRect,
    style: i32,
    backing: i32,
    defer: i32,
) -> *mut c_void {
    unsafe {
        let f: MsgSendIdRectIntIntInt = transmute(objc_msgSend as *const ());
        f(obj, sel, rect, style, backing, defer)
    }
}

/// Get screen frame (NSRect)
fn msg_send_rect(obj: *mut c_void, sel: *mut c_void) -> NSRect {
    unsafe {
        let f: extern "C" fn(*mut c_void, *mut c_void) -> NSRect =
            transmute(objc_msgSend as *const ());
        f(obj, sel)
    }
}

fn msg_send_void_bool(obj: *mut c_void, sel: *mut c_void, arg: bool) {
    unsafe {
        let f: MsgSendVoidBool = transmute(objc_msgSend as *const ());
        f(obj, sel, arg)
    }
}

/// Format a number with thousands separators
fn format_with_thousands(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

/// Set the font size for the text view
fn set_text_view_font_size(text_view: &ObjCObject, size: f64) {
    unsafe {
        // Clamp font size between 8 and 48 points
        let clamped_size = if size < 8.0 {
            8.0
        } else if size > 48.0 {
            48.0
        } else {
            size
        };

        let ns_font_class = match ObjCClass::get("NSFont") {
            Some(c) => c,
            None => return,
        };

        // Get system font with the new size: [NSFont systemFontOfSize:size]
        type MsgSendIdDouble = extern "C" fn(*mut c_void, *mut c_void, f64) -> *mut c_void;
        let f: MsgSendIdDouble = transmute(objc_msgSend as *const ());
        let font = f(
            ns_font_class.as_ptr(),
            Sel::get("systemFontOfSize:").as_ptr(),
            clamped_size,
        );

        if !font.is_null() {
            // Set the font on the text view: [textView setFont:font]
            msg_send_void_id(text_view.as_ptr(), Sel::get("setFont:").as_ptr(), font);
        }
    }
}

/// Callback for mouse enter event on text view (for hover popups)
extern "C" fn mouse_entered(_self: *mut c_void, _sel: *mut c_void, event: *mut c_void) {
    unsafe {
        let event = ObjCObject::from_ptr(event);

        // Get tracking area from event
        let tracking_area = msg_send_id(event.as_ptr(), Sel::get("trackingArea").as_ptr());
        if tracking_area.is_null() {
            return;
        }

        // Get user info dictionary from tracking area
        let tracking_area = ObjCObject::from_ptr(tracking_area);
        let user_info = msg_send_id(tracking_area.as_ptr(), Sel::get("userInfo").as_ptr());
        if user_info.is_null() {
            eprintln!("Hover: mouse entered tracked area");
            return;
        }

        // In a full implementation, we'd show a tooltip/popup here
        // For now, just log it
        eprintln!("Hover: mouse entered - would show popup");
    }
}

/// Callback for mouse exit event on text view
extern "C" fn mouse_exited(_self: *mut c_void, _sel: *mut c_void, _event: *mut c_void) {
    unsafe {
        eprintln!("Hover: mouse exited - would hide popup");
    }
}

/// Apply random red highlighting to words in the text view (visual effect for errors)
/// This creates a spell-check-like effect
fn apply_random_underlines(text_view: &ObjCObject) {
    unsafe {
        // Get the text storage (NSTextStorage is mutable attributed string)
        let text_storage = msg_send_id(text_view.as_ptr(), Sel::get("textStorage").as_ptr());
        if text_storage.is_null() {
            return;
        }
        let text_storage = ObjCObject::from_ptr(text_storage);

        // Get the current string to find word boundaries
        let text_str = msg_send_id(text_storage.as_ptr(), Sel::get("string").as_ptr());
        if text_str.is_null() {
            return;
        }

        // Get the length of the string in UTF-16 characters (what NSRange uses)
        type MsgSendUsize = extern "C" fn(*mut c_void, *mut c_void) -> usize;
        let f_len: MsgSendUsize = transmute(objc_msgSend as *const ());
        let utf16_len = f_len(text_str, Sel::get("length").as_ptr());

        if utf16_len == 0 {
            return;
        }

        let text_cstr: *const c_char = {
            let f: extern "C" fn(*mut c_void, *mut c_void) -> *const c_char =
                transmute(objc_msgSend as *const ());
            f(text_str, Sel::get("UTF8String").as_ptr())
        };

        if text_cstr.is_null() {
            return;
        }

        let text = CStr::from_ptr(text_cstr).to_string_lossy();

        // Build a UTF-8 byte position to UTF-16 character position mapping
        // Only iterate through valid char boundaries to avoid panicking on multi-byte chars
        let mut utf8_to_utf16: HashMap<usize, usize> = HashMap::new();
        let mut utf16_pos = 0;
        utf8_to_utf16.insert(0, 0);

        for (byte_pos, ch) in text.char_indices() {
            utf16_pos += ch.encode_utf16(&mut [0; 2]).len();
            // Map the position AFTER this character
            let next_pos = byte_pos + ch.len_utf8();
            utf8_to_utf16.insert(next_pos, utf16_pos);
        }

        // Find word boundaries (in UTF-8 byte positions)
        let mut words: Vec<(usize, usize)> = Vec::new(); // (start byte pos, end byte pos) pairs
        let mut in_word = false;
        let mut word_start = 0;

        for (pos, ch) in text.char_indices() {
            if ch.is_alphabetic() || ch.is_numeric() {
                if !in_word {
                    word_start = pos;
                    in_word = true;
                }
            } else {
                if in_word {
                    words.push((word_start, pos));
                    in_word = false;
                }
            }
        }
        if in_word {
            words.push((word_start, text.len()));
        }

        // Get yellow color for highlights (background)
        let ns_color_class = match ObjCClass::get("NSColor") {
            Some(c) => c,
            None => return,
        };
        let highlight_color =
            msg_send_id(ns_color_class.as_ptr(), Sel::get("yellowColor").as_ptr());

        // Apply highlighting to every 5th word to simulate spell-check marks
        let mut applied_count = 0;
        for (idx, (start_byte, end_byte)) in words.iter().enumerate() {
            if idx % 5 == 2 {
                // Convert UTF-8 byte positions to UTF-16 character positions
                let start_utf16 = match utf8_to_utf16.get(start_byte) {
                    Some(&pos) => pos,
                    None => continue, // Skip if byte position not found
                };

                let end_utf16 = match utf8_to_utf16.get(end_byte) {
                    Some(&pos) => pos,
                    None => utf16_len, // Use full length if we're at the end
                };

                let word_len = if end_utf16 > start_utf16 {
                    end_utf16 - start_utf16
                } else {
                    continue; // Skip invalid ranges
                };

                // Validate that the range is within bounds
                if start_utf16 >= utf16_len || start_utf16 + word_len > utf16_len {
                    continue;
                }

                let range = NSRange {
                    location: start_utf16,
                    length: word_len,
                };

                type MsgSendVoidIdIdRange =
                    extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void, NSRange);
                let f: MsgSendVoidIdIdRange = transmute(objc_msgSend as *const ());

                // Apply yellow background highlighting
                let bg_attr_name = create_nsstring("NSBackgroundColorAttributeName");
                f(
                    text_storage.as_ptr(),
                    Sel::get("addAttribute:value:range:").as_ptr() as *mut c_void,
                    bg_attr_name,
                    highlight_color,
                    range,
                );

                // Apply red foreground color text
                let orange_color =
                    msg_send_id(ns_color_class.as_ptr(), Sel::get("redColor").as_ptr());
                let color_attr_name = create_nsstring("NSForegroundColorAttributeName");
                f(
                    text_storage.as_ptr(),
                    Sel::get("addAttribute:value:range:").as_ptr() as *mut c_void,
                    color_attr_name,
                    orange_color,
                    range,
                );

                applied_count += 1;
            }
        }

        if applied_count > 0 {
            eprintln!("Applied highlighting to {} words", applied_count);

            // Force the text view to redraw by notifying it of the attribute change
            // Send setNeedsDisplay:YES to mark the view as needing redraw
            msg_send_void_bool(
                text_view.as_ptr(),
                Sel::get("setNeedsDisplay:").as_ptr(),
                true,
            );
        }
    }
}

/// Helper to create NSString from Rust string
fn create_nsstring(s: &str) -> *mut c_void {
    let c_str = match CString::new(s) {
        Ok(c) => c,
        Err(_) => return null_mut(),
    };
    let ns_string_class = match ObjCClass::get("NSString") {
        Some(c) => c,
        None => return null_mut(),
    };
    let f: MsgSendIdCStr = unsafe { transmute(objc_msgSend as *const ()) };
    f(
        ns_string_class.as_ptr(),
        Sel::get("stringWithUTF8String:").as_ptr(),
        c_str.as_ptr(),
    )
}

/// Setup the application menu with Quit option, and File menu with Open
fn setup_menu(app: &ObjCObject, app_delegate: &ObjCObject) {
    let ns_menu_class = ObjCClass::get("NSMenu").expect("Failed to get NSMenu class");
    let ns_menu_item_class = ObjCClass::get("NSMenuItem").expect("Failed to get NSMenuItem class");

    // Create main menu
    let main_menu = msg_send_id(ns_menu_class.as_ptr(), Sel::get("alloc").as_ptr());
    let main_menu = msg_send_id(main_menu, Sel::get("init").as_ptr());
    let main_menu = ObjCObject::from_ptr(main_menu);

    // Create app submenu
    let app_submenu = msg_send_id(ns_menu_class.as_ptr(), Sel::get("alloc").as_ptr());
    let app_submenu = msg_send_id(app_submenu, Sel::get("init").as_ptr());
    let app_submenu = ObjCObject::from_ptr(app_submenu);

    // Create quit menu item
    let quit_title_obj = create_nsstring("Quit rust_macos_gui");
    let quit_item = msg_send_id(ns_menu_item_class.as_ptr(), Sel::get("alloc").as_ptr());
    let q_str = create_nsstring("q");

    // initWithTitle:action:keyEquivalent:
    type MsgSendIdIdIdId = extern "C" fn(
        *mut c_void,
        *mut c_void,
        *mut c_void,
        *mut c_void,
        *mut c_void,
    ) -> *mut c_void;
    let quit_item = unsafe {
        let f: MsgSendIdIdIdId = transmute(objc_msgSend as *const ());
        f(
            quit_item,
            Sel::get("initWithTitle:action:keyEquivalent:").as_ptr(),
            quit_title_obj,
            Sel::get("terminate:").as_ptr(),
            q_str,
        )
    };
    let quit_item = ObjCObject::from_ptr(quit_item);

    // Add quit item to app submenu
    msg_send_void_id(
        app_submenu.as_ptr(),
        Sel::get("addItem:").as_ptr(),
        quit_item.as_ptr(),
    );

    // Create app menu item (with no title initially)
    let app_menu_item = msg_send_id(ns_menu_item_class.as_ptr(), Sel::get("alloc").as_ptr());
    let app_menu_item = msg_send_id(app_menu_item, Sel::get("init").as_ptr());
    let app_menu_item = ObjCObject::from_ptr(app_menu_item);

    // Set submenu on app menu item
    msg_send_void_id(
        app_menu_item.as_ptr(),
        Sel::get("setSubmenu:").as_ptr(),
        app_submenu.as_ptr(),
    );

    // Add app menu item to main menu
    msg_send_void_id(
        main_menu.as_ptr(),
        Sel::get("addItem:").as_ptr(),
        app_menu_item.as_ptr(),
    );

    // Create File menu
    let file_submenu = msg_send_id(ns_menu_class.as_ptr(), Sel::get("alloc").as_ptr());
    let file_submenu = msg_send_id(file_submenu, Sel::get("init").as_ptr());
    let file_submenu = ObjCObject::from_ptr(file_submenu);

    // Create Open menu item
    let open_title_obj = create_nsstring("Open");
    let open_item = msg_send_id(ns_menu_item_class.as_ptr(), Sel::get("alloc").as_ptr());
    let o_str = create_nsstring("o");

    let open_item = unsafe {
        let f: MsgSendIdIdIdId = transmute(objc_msgSend as *const ());
        f(
            open_item,
            Sel::get("initWithTitle:action:keyEquivalent:").as_ptr(),
            open_title_obj,
            Sel::get("open_file:").as_ptr(),
            o_str,
        )
    };
    let open_item = ObjCObject::from_ptr(open_item);

    // Set target for the Open menu item
    msg_send_void_id(
        open_item.as_ptr(),
        Sel::get("setTarget:").as_ptr(),
        app_delegate.as_ptr(),
    );

    // Add open item to file submenu
    msg_send_void_id(
        file_submenu.as_ptr(),
        Sel::get("addItem:").as_ptr(),
        open_item.as_ptr(),
    );

    // Create File menu item
    let file_menu_item = msg_send_id(ns_menu_item_class.as_ptr(), Sel::get("alloc").as_ptr());
    let file_menu_item = msg_send_id(file_menu_item, Sel::get("init").as_ptr());
    let file_menu_item = ObjCObject::from_ptr(file_menu_item);

    // Set submenu on file menu item
    let file_title = create_nsstring("File");
    msg_send_void_id(
        file_menu_item.as_ptr(),
        Sel::get("setTitle:").as_ptr(),
        file_title,
    );
    msg_send_void_id(
        file_menu_item.as_ptr(),
        Sel::get("setSubmenu:").as_ptr(),
        file_submenu.as_ptr(),
    );

    // Add file menu item to main menu
    msg_send_void_id(
        main_menu.as_ptr(),
        Sel::get("addItem:").as_ptr(),
        file_menu_item.as_ptr(),
    );

    // Set main menu on app
    msg_send_void_id(
        app.as_ptr(),
        Sel::get("setMainMenu:").as_ptr(),
        main_menu.as_ptr(),
    );
}

fn main() {
    let ns_app_class = ObjCClass::get("NSApplication").expect("Failed to get NSApplication class");

    let shared_app = msg_send_id(
        ns_app_class.as_ptr(),
        Sel::get("sharedApplication").as_ptr(),
    );
    let shared_app = ObjCObject::from_ptr(shared_app);

    // Set activation policy to Regular (makes app appear in Dock and be frontmost)
    msg_send_void_int(
        shared_app.as_ptr(),
        Sel::get("setActivationPolicy:").as_ptr(),
        0,
    );

    // Create and set app delegate BEFORE finishLaunching so callback gets triggered
    let app_delegate = create_app_delegate(&shared_app);
    msg_send_void_id(
        shared_app.as_ptr(),
        Sel::get("setDelegate:").as_ptr(),
        app_delegate.as_ptr(),
    );

    // Store app delegate globally for menu actions
    unsafe {
        APP_DELEGATE = Some(app_delegate.clone());
    }

    // Setup main menu with Quit option and Open action BEFORE finishLaunching
    setup_menu(&shared_app, &app_delegate);

    // finishLaunching triggers applicationDidFinishLaunching: callback which creates window
    unsafe {
        let f: MsgSendId = transmute(objc_msgSend as *const ());
        f(shared_app.as_ptr(), Sel::get("finishLaunching").as_ptr());
    }

    // Run the event loop (no arguments needed)
    unsafe {
        let f: MsgSendId = transmute(objc_msgSend as *const ());
        f(shared_app.as_ptr(), Sel::get("run").as_ptr());
    }
}

fn create_window() -> ObjCObject {
    // Get main screen and its frame
    let ns_screen_class = ObjCClass::get("NSScreen").expect("Failed to get NSScreen class");
    let main_screen = msg_send_id(ns_screen_class.as_ptr(), Sel::get("mainScreen").as_ptr());
    let screen_frame = msg_send_rect(main_screen, Sel::get("frame").as_ptr());

    // Calculate window size as 3/4 of screen
    let frame = NSRect {
        origin: NSPoint {
            x: screen_frame.size.width * 0.125,
            y: screen_frame.size.height * 0.125,
        },
        size: NSSize {
            width: screen_frame.size.width * 0.75,
            height: screen_frame.size.height * 0.75,
        },
    };

    let ns_window_class = ObjCClass::get("NSWindow").expect("Failed to get NSWindow class");

    let alloc_window = msg_send_id(ns_window_class.as_ptr(), Sel::get("alloc").as_ptr());

    let window = msg_send_id_rect_int_int_int(
        alloc_window,
        Sel::get("initWithContentRect:styleMask:backing:defer:").as_ptr(),
        frame,
        15, // styleMask: Titled | Closable | Miniaturizable | Resizable
        2,  // backing: Buffered
        0,  // defer: NO
    );

    let window = ObjCObject::from_ptr(window);
    window
}

/// Setup toolbar, text view, and status bar for the window
fn setup_window_ui(window: &ObjCObject) {
    // Get window content view
    let content_view = msg_send_id(window.as_ptr(), Sel::get("contentView").as_ptr());
    if content_view.is_null() {
        return;
    }
    let content_view = ObjCObject::from_ptr(content_view);

    // Create UI elements (they'll be laid out later in main() after window is shown)
    let (_toolbar, _text_view, _status_bar) = create_ui_elements(&content_view);

    // Create and set delegate that handles resize
    let delegate = create_window_delegate_with_layout(window);
    msg_send_void_id(
        window.as_ptr(),
        Sel::get("setDelegate:").as_ptr(),
        delegate.as_ptr(),
    );
}

/// Global references to UI elements (used for layout updates)
/// Stores: (toolbar, scroll_view, status_bar)
static mut UI_ELEMENTS: Option<(ObjCObject, ObjCObject, ObjCObject)> = None;

/// Global reference to text view (used to set loaded file content)
static mut TEXT_VIEW: Option<ObjCObject> = None;

/// Global reference to app delegate (for menu actions)
static mut APP_DELEGATE: Option<ObjCObject> = None;

/// Global reference to open button (for setting target)
static mut OPEN_BUTTON: Option<ObjCObject> = None;

/// Global reference to URL button (for setting target)
static mut URL_BUTTON: Option<ObjCObject> = None;

/// Global reference to window (for updating title)
static mut WINDOW: Option<ObjCObject> = None;

/// Global references to status bar labels (for updating status text)
static mut STATUS_FILE_LABEL: Option<ObjCObject> = None;
static mut STATUS_BYTE_LABEL: Option<ObjCObject> = None;
static mut STATUS_LINE_LABEL: Option<ObjCObject> = None;

/// Global text view font size for pinch zoom
static mut TEXT_VIEW_FONT_SIZE: f64 = 12.0;

/// Create the UI elements (toolbar, text view, status bar)
fn create_ui_elements(content_view: &ObjCObject) -> (ObjCObject, ObjCObject, ObjCObject) {
    let ns_view_class = ObjCClass::get("NSView").expect("Failed to get NSView class");
    let ns_color_class = ObjCClass::get("NSColor").expect("Failed to get NSColor class");

    // Create toolbar
    let toolbar = msg_send_id(ns_view_class.as_ptr(), Sel::get("alloc").as_ptr());
    let toolbar = msg_send_id_rect(
        toolbar,
        Sel::get("initWithFrame:").as_ptr(),
        NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size: NSSize {
                width: 100.0,
                height: 44.0,
            },
        },
    );
    let toolbar = ObjCObject::from_ptr(toolbar);
    let light_gray = msg_send_id(ns_color_class.as_ptr(), Sel::get("lightGrayColor").as_ptr());
    msg_send_void_id(
        toolbar.as_ptr(),
        Sel::get("setBackgroundColor:").as_ptr(),
        light_gray,
    );
    msg_send_void_id(
        content_view.as_ptr(),
        Sel::get("addSubview:").as_ptr(),
        toolbar.as_ptr(),
    );

    // Add Open button to toolbar
    let ns_button_class = ObjCClass::get("NSButton").expect("Failed to get NSButton class");
    let open_button = msg_send_id(ns_button_class.as_ptr(), Sel::get("alloc").as_ptr());
    let open_button = msg_send_id_rect(
        open_button,
        Sel::get("initWithFrame:").as_ptr(),
        NSRect {
            origin: NSPoint { x: 10.0, y: 10.0 },
            size: NSSize {
                width: 80.0,
                height: 24.0,
            },
        },
    );
    let open_button = ObjCObject::from_ptr(open_button);

    // Set button title
    let open_btn_title = create_nsstring("Open");
    msg_send_void_id(
        open_button.as_ptr(),
        Sel::get("setTitle:").as_ptr(),
        open_btn_title,
    );

    // Set button action
    msg_send_void_id(
        open_button.as_ptr(),
        Sel::get("setAction:").as_ptr(),
        Sel::get("open_file:").as_ptr(),
    );

    // Store button globally so we can set target later
    unsafe {
        OPEN_BUTTON = Some(open_button.clone());
    }

    // Add button to toolbar
    msg_send_void_id(
        toolbar.as_ptr(),
        Sel::get("addSubview:").as_ptr(),
        open_button.as_ptr(),
    );

    // Add URL button to toolbar
    let url_button = msg_send_id(ns_button_class.as_ptr(), Sel::get("alloc").as_ptr());
    let url_button = msg_send_id_rect(
        url_button,
        Sel::get("initWithFrame:").as_ptr(),
        NSRect {
            origin: NSPoint { x: 100.0, y: 10.0 },
            size: NSSize {
                width: 100.0,
                height: 24.0,
            },
        },
    );
    let url_button = ObjCObject::from_ptr(url_button);

    // Set button title
    let url_btn_title = create_nsstring("Open URL");
    msg_send_void_id(
        url_button.as_ptr(),
        Sel::get("setTitle:").as_ptr(),
        url_btn_title,
    );

    // Set button action
    msg_send_void_id(
        url_button.as_ptr(),
        Sel::get("setAction:").as_ptr(),
        Sel::get("open_url:").as_ptr(),
    );

    // Store button globally so we can set target later
    unsafe {
        URL_BUTTON = Some(url_button.clone());
    }

    // Add button to toolbar
    msg_send_void_id(
        toolbar.as_ptr(),
        Sel::get("addSubview:").as_ptr(),
        url_button.as_ptr(),
    );

    // Create scroll view containing text view
    let ns_scroll_view_class =
        ObjCClass::get("NSScrollView").expect("Failed to get NSScrollView class");
    let scroll_view = msg_send_id(ns_scroll_view_class.as_ptr(), Sel::get("alloc").as_ptr());
    let scroll_view = msg_send_id_rect(
        scroll_view,
        Sel::get("initWithFrame:").as_ptr(),
        NSRect {
            origin: NSPoint { x: 0.0, y: 20.0 },
            size: NSSize {
                width: 100.0,
                height: 100.0,
            },
        },
    );
    let scroll_view = ObjCObject::from_ptr(scroll_view);

    // Configure scroll view
    msg_send_void_bool(
        scroll_view.as_ptr(),
        Sel::get("setHasVerticalScroller:").as_ptr(),
        true,
    );

    // Create custom text view class that supports pinch zoom
    let ns_text_view_class = create_custom_text_view_class();
    let text_view = msg_send_id(ns_text_view_class, Sel::get("alloc").as_ptr());
    let text_view = msg_send_id_rect(
        text_view,
        Sel::get("initWithFrame:").as_ptr(),
        NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size: NSSize {
                width: 100.0,
                height: 100.0,
            },
        },
    );
    let text_view = ObjCObject::from_ptr(text_view);
    msg_send_void_bool(text_view.as_ptr(), Sel::get("setEditable:").as_ptr(), false);

    // Allow text view to be wider than scroll view for horizontal scrolling
    msg_send_void_bool(
        text_view.as_ptr(),
        Sel::get("setHorizontallyResizable:").as_ptr(),
        true,
    );
    msg_send_void_bool(
        text_view.as_ptr(),
        Sel::get("setVerticallyResizable:").as_ptr(),
        true,
    );

    // Get text container and configure it
    let text_container = msg_send_id(text_view.as_ptr(), Sel::get("textContainer").as_ptr());
    if !text_container.is_null() {
        let text_container = ObjCObject::from_ptr(text_container);
        // Set container size to a very large width but let height be unlimited
        type MsgSendVoidSize = extern "C" fn(*mut c_void, *mut c_void, NSSize);
        unsafe {
            let f: MsgSendVoidSize = transmute(objc_msgSend as *const ());
            let huge_size = NSSize {
                width: 100000.0,
                height: 100000.0,
            };
            f(
                text_container.as_ptr(),
                Sel::get("setContainerSize:").as_ptr(),
                huge_size,
            );
        }
    }

    // Add sample text
    let sample_text = match CString::new(
        "Sample Text View\n\nThis is a read-only text view with vertical scrolling support.",
    ) {
        Ok(s) => s,
        Err(_) => return (toolbar, scroll_view, text_view),
    };
    let text_obj = {
        let ns_string_class = match ObjCClass::get("NSString") {
            Some(c) => c,
            None => return (toolbar, scroll_view, text_view),
        };
        type MsgSendIdCStr = extern "C" fn(*mut c_void, *mut c_void, *const c_char) -> *mut c_void;
        unsafe {
            let f: MsgSendIdCStr = transmute(objc_msgSend as *const ());
            f(
                ns_string_class.as_ptr(),
                Sel::get("stringWithUTF8String:").as_ptr(),
                sample_text.as_ptr(),
            )
        }
    };
    msg_send_void_id(
        text_view.as_ptr(),
        Sel::get("setString:").as_ptr(),
        text_obj,
    );

    // Add text view to scroll view
    msg_send_void_id(
        scroll_view.as_ptr(),
        Sel::get("setDocumentView:").as_ptr(),
        text_view.as_ptr(),
    );

    // Add magnification gesture recognizer for pinch zoom
    // The custom text view class overrides magnifyWithEvent: which will be called by the system
    // when pinch gestures are detected on the view, so no explicit gesture recognizer needed

    // Add scroll view to content view
    msg_send_void_id(
        content_view.as_ptr(),
        Sel::get("addSubview:").as_ptr(),
        scroll_view.as_ptr(),
    );

    // Create status bar
    let status_bar = msg_send_id(ns_view_class.as_ptr(), Sel::get("alloc").as_ptr());
    let status_bar = msg_send_id_rect(
        status_bar,
        Sel::get("initWithFrame:").as_ptr(),
        NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size: NSSize {
                width: 100.0,
                height: 20.0,
            },
        },
    );
    let status_bar = ObjCObject::from_ptr(status_bar);
    let dark_gray = msg_send_id(ns_color_class.as_ptr(), Sel::get("darkGrayColor").as_ptr());
    msg_send_void_id(
        status_bar.as_ptr(),
        Sel::get("setBackgroundColor:").as_ptr(),
        dark_gray,
    );
    msg_send_void_id(
        content_view.as_ptr(),
        Sel::get("addSubview:").as_ptr(),
        status_bar.as_ptr(),
    );

    // Create status bar labels
    let ns_label_class = ObjCClass::get("NSTextField").expect("Failed to get NSTextField class");
    let white_color = msg_send_id(ns_color_class.as_ptr(), Sel::get("whiteColor").as_ptr());

    // File status label (left)
    let file_label = msg_send_id(ns_label_class.as_ptr(), Sel::get("alloc").as_ptr());
    let file_label = msg_send_id_rect(
        file_label,
        Sel::get("initWithFrame:").as_ptr(),
        NSRect {
            origin: NSPoint { x: 5.0, y: 2.0 },
            size: NSSize {
                width: 150.0,
                height: 16.0,
            },
        },
    );
    let file_label = ObjCObject::from_ptr(file_label);
    let file_text = create_nsstring("No file loaded");
    msg_send_void_id(
        file_label.as_ptr(),
        Sel::get("setStringValue:").as_ptr(),
        file_text,
    );
    msg_send_void_id(
        file_label.as_ptr(),
        Sel::get("setTextColor:").as_ptr(),
        white_color,
    );
    msg_send_void_id(
        file_label.as_ptr(),
        Sel::get("setBordered:").as_ptr(),
        null_mut(),
    );
    msg_send_void_id(
        file_label.as_ptr(),
        Sel::get("setDrawsBackground:").as_ptr(),
        null_mut(),
    );
    msg_send_void_bool(
        file_label.as_ptr(),
        Sel::get("setEditable:").as_ptr(),
        false,
    );
    msg_send_void_bool(
        file_label.as_ptr(),
        Sel::get("setSelectable:").as_ptr(),
        false,
    );
    msg_send_void_id(
        status_bar.as_ptr(),
        Sel::get("addSubview:").as_ptr(),
        file_label.as_ptr(),
    );
    unsafe {
        STATUS_FILE_LABEL = Some(file_label.clone());
    }

    // Byte count label (middle)
    let byte_label = msg_send_id(ns_label_class.as_ptr(), Sel::get("alloc").as_ptr());
    let byte_label = msg_send_id_rect(
        byte_label,
        Sel::get("initWithFrame:").as_ptr(),
        NSRect {
            origin: NSPoint { x: 160.0, y: 2.0 },
            size: NSSize {
                width: 120.0,
                height: 16.0,
            },
        },
    );
    let byte_label = ObjCObject::from_ptr(byte_label);
    let byte_text = create_nsstring("Bytes: 0");
    msg_send_void_id(
        byte_label.as_ptr(),
        Sel::get("setStringValue:").as_ptr(),
        byte_text,
    );
    msg_send_void_id(
        byte_label.as_ptr(),
        Sel::get("setTextColor:").as_ptr(),
        white_color,
    );
    msg_send_void_id(
        byte_label.as_ptr(),
        Sel::get("setBordered:").as_ptr(),
        null_mut(),
    );
    msg_send_void_id(
        byte_label.as_ptr(),
        Sel::get("setDrawsBackground:").as_ptr(),
        null_mut(),
    );
    msg_send_void_bool(
        byte_label.as_ptr(),
        Sel::get("setEditable:").as_ptr(),
        false,
    );
    msg_send_void_bool(
        byte_label.as_ptr(),
        Sel::get("setSelectable:").as_ptr(),
        false,
    );
    msg_send_void_id(
        status_bar.as_ptr(),
        Sel::get("addSubview:").as_ptr(),
        byte_label.as_ptr(),
    );
    unsafe {
        STATUS_BYTE_LABEL = Some(byte_label.clone());
    }

    // Line count label (right)
    let line_label = msg_send_id(ns_label_class.as_ptr(), Sel::get("alloc").as_ptr());
    let line_label = msg_send_id_rect(
        line_label,
        Sel::get("initWithFrame:").as_ptr(),
        NSRect {
            origin: NSPoint { x: 285.0, y: 2.0 },
            size: NSSize {
                width: 120.0,
                height: 16.0,
            },
        },
    );
    let line_label = ObjCObject::from_ptr(line_label);
    let line_text = create_nsstring("Lines: 0");
    msg_send_void_id(
        line_label.as_ptr(),
        Sel::get("setStringValue:").as_ptr(),
        line_text,
    );
    msg_send_void_id(
        line_label.as_ptr(),
        Sel::get("setTextColor:").as_ptr(),
        white_color,
    );
    msg_send_void_id(
        line_label.as_ptr(),
        Sel::get("setBordered:").as_ptr(),
        null_mut(),
    );
    msg_send_void_id(
        line_label.as_ptr(),
        Sel::get("setDrawsBackground:").as_ptr(),
        null_mut(),
    );
    msg_send_void_bool(
        line_label.as_ptr(),
        Sel::get("setEditable:").as_ptr(),
        false,
    );
    msg_send_void_bool(
        line_label.as_ptr(),
        Sel::get("setSelectable:").as_ptr(),
        false,
    );
    msg_send_void_id(
        status_bar.as_ptr(),
        Sel::get("addSubview:").as_ptr(),
        line_label.as_ptr(),
    );
    unsafe {
        STATUS_LINE_LABEL = Some(line_label.clone());
    }

    // Store references globally for layout updates and file loading
    unsafe {
        UI_ELEMENTS = Some((toolbar, scroll_view, status_bar));
        TEXT_VIEW = Some(text_view.clone());
    }

    // Setup gesture recognizers for pinch zoom (without needing text selection)
    setup_text_view_gestures(&text_view);

    (toolbar, scroll_view, status_bar)
}

/// Layout UI elements based on window size
fn layout_ui_elements(window: &ObjCObject) {
    let content_view = msg_send_id(window.as_ptr(), Sel::get("contentView").as_ptr());
    let content_view = ObjCObject::from_ptr(content_view);

    // Use contentView's bounds, not window's frame
    let content_bounds = msg_send_rect(content_view.as_ptr(), Sel::get("bounds").as_ptr());
    let content_height = content_bounds.size.height;
    let content_width = content_bounds.size.width;

    let toolbar_height = 44.0;
    let status_bar_height = 20.0;
    let text_view_height = content_height - toolbar_height - status_bar_height;

    // Get stored references
    unsafe {
        if let Some((toolbar, text_view, status_bar)) = UI_ELEMENTS {
            // macOS uses flipped coordinates for window contentView (y increases downward)
            // Toolbar at top (y = content_height - toolbar_height)
            let toolbar_frame = NSRect {
                origin: NSPoint {
                    x: 0.0,
                    y: content_height - toolbar_height,
                },
                size: NSSize {
                    width: content_width,
                    height: toolbar_height,
                },
            };
            set_view_frame(toolbar, toolbar_frame);

            // Status bar at bottom (y = 0)
            let status_bar_frame = NSRect {
                origin: NSPoint { x: 0.0, y: 0.0 },
                size: NSSize {
                    width: content_width,
                    height: status_bar_height,
                },
            };
            set_view_frame(status_bar, status_bar_frame);

            // Text view in middle (y = status_bar_height, height = rest)
            let text_view_frame = NSRect {
                origin: NSPoint {
                    x: 0.0,
                    y: status_bar_height,
                },
                size: NSSize {
                    width: content_width,
                    height: text_view_height,
                },
            };
            set_view_frame(text_view, text_view_frame);

            // Update text container width to match scroll view (for proper text wrapping on resize)
            if let Some(text_view_obj) = TEXT_VIEW {
                let text_container =
                    msg_send_id(text_view_obj.as_ptr(), Sel::get("textContainer").as_ptr());
                if !text_container.is_null() {
                    let text_container = ObjCObject::from_ptr(text_container);
                    type MsgSendVoidSize = extern "C" fn(*mut c_void, *mut c_void, NSSize);
                    let f: MsgSendVoidSize = transmute(objc_msgSend as *const ());
                    // Set width to current scroll view width, height unlimited
                    let new_size = NSSize {
                        width: content_width,
                        height: 100000.0,
                    };
                    f(
                        text_container.as_ptr(),
                        Sel::get("setContainerSize:").as_ptr(),
                        new_size,
                    );
                }
            }
        }
    }
}

/// Set frame for a view
fn set_view_frame(view: ObjCObject, frame: NSRect) {
    unsafe {
        let f: MsgSendVoidRect = transmute(objc_msgSend as *const ());
        f(view.as_ptr(), Sel::get("setFrame:").as_ptr(), frame);
    }
}

/// Menu action: Open file dialog and load file
extern "C" fn open_file(_self: *mut c_void, _sel: *mut c_void, _sender: *mut c_void) {
    unsafe {
        let ns_open_panel_class = match ObjCClass::get("NSOpenPanel") {
            Some(c) => c,
            None => return,
        };
        let open_panel = msg_send_id(ns_open_panel_class.as_ptr(), Sel::get("openPanel").as_ptr());
        let open_panel = ObjCObject::from_ptr(open_panel);

        // Set allowed file types - text-like formats: txt, md, html, xml, css, json, rtf, pdf, etc.
        let extensions = vec![
            "txt", "md", "markdown", "html", "htm", "xml", "css", "json", "js", "ts", "rs", "py",
            "rb", "go", "c", "h", "cpp", "java", "rtf", "pdf",
        ];

        let allowed_types = {
            let ns_array_class = match ObjCClass::get("NSArray") {
                Some(c) => c,
                None => return,
            };
            let ns_array_ptr = ns_array_class.as_ptr();

            // Create array with all extensions
            type MsgSendIdIdArray =
                extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void;
            let f_array: MsgSendIdIdArray = transmute(objc_msgSend as *const ());

            let mut array = msg_send_id(ns_array_ptr, Sel::get("array").as_ptr());

            // Add each extension to the array
            type MsgSendIdIdIdId =
                extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void;
            let f_add: MsgSendIdIdIdId = transmute(objc_msgSend as *const ());

            for ext in extensions {
                let ext_str = create_nsstring(ext);
                array = f_add(array, Sel::get("arrayByAddingObject:").as_ptr(), ext_str);
            }

            array
        };
        msg_send_void_id(
            open_panel.as_ptr(),
            Sel::get("setAllowedFileTypes:").as_ptr(),
            allowed_types,
        );

        // Show the dialog
        type MsgSendInt = extern "C" fn(*mut c_void, *mut c_void) -> i64;
        let f: MsgSendInt = transmute(objc_msgSend as *const ());
        let result = f(open_panel.as_ptr(), Sel::get("runModal").as_ptr());

        if result == 1 {
            // NSOKButton
            // Get selected file URL
            let url = msg_send_id(open_panel.as_ptr(), Sel::get("URL").as_ptr());
            if !url.is_null() {
                // Get path from URL
                let path = msg_send_id(url, Sel::get("path").as_ptr());
                if !path.is_null() {
                    // Convert NSString path to Rust string
                    let path_cstr: *const c_char = unsafe {
                        let f: extern "C" fn(*mut c_void, *mut c_void) -> *const c_char =
                            transmute(objc_msgSend as *const ());
                        f(path, Sel::get("UTF8String").as_ptr())
                    };

                    // Read file contents
                    if !path_cstr.is_null() {
                        let file_path = CStr::from_ptr(path_cstr).to_string_lossy();
                        if let Ok(contents) = read_to_string(file_path.as_ref()) {
                            // Set text view content
                            if let Some(text_view) = TEXT_VIEW {
                                let content_str = create_nsstring(&contents);
                                msg_send_void_id(
                                    text_view.as_ptr(),
                                    Sel::get("setString:").as_ptr(),
                                    content_str,
                                );

                                // Re-sync text container after loading to ensure proper layout
                                if let Some(window) = WINDOW {
                                    let content_view = msg_send_id(
                                        window.as_ptr(),
                                        Sel::get("contentView").as_ptr(),
                                    );
                                    let content_bounds =
                                        msg_send_rect(content_view, Sel::get("bounds").as_ptr());
                                    let scroll_view_width = content_bounds.size.width;

                                    let text_container = msg_send_id(
                                        text_view.as_ptr(),
                                        Sel::get("textContainer").as_ptr(),
                                    );
                                    if !text_container.is_null() {
                                        let text_container = ObjCObject::from_ptr(text_container);
                                        type MsgSendVoidSize =
                                            extern "C" fn(*mut c_void, *mut c_void, NSSize);
                                        let f: MsgSendVoidSize =
                                            transmute(objc_msgSend as *const ());
                                        let new_size = NSSize {
                                            width: scroll_view_width,
                                            height: 100000.0,
                                        };
                                        f(
                                            text_container.as_ptr(),
                                            Sel::get("setContainerSize:").as_ptr(),
                                            new_size,
                                        );
                                    }
                                }
                            }

                            // Update window title with filename
                            if let Some(window) = WINDOW {
                                // Extract filename from full path
                                let path_str = file_path.as_ref();
                                let filename = path_str.split('/').last().unwrap_or(path_str);
                                let title_str = create_nsstring(filename);
                                msg_send_void_id(
                                    window.as_ptr(),
                                    Sel::get("setTitle:").as_ptr(),
                                    title_str,
                                );
                            }

                            // Update status bar labels
                            let byte_count = contents.len();
                            let line_count = contents.lines().count();

                            if let Some(file_label) = STATUS_FILE_LABEL {
                                let file_text = {
                                    let path_str = file_path.as_ref();
                                    let filename = path_str.split('/').last().unwrap_or(path_str);
                                    create_nsstring(filename)
                                };
                                msg_send_void_id(
                                    file_label.as_ptr(),
                                    Sel::get("setStringValue:").as_ptr(),
                                    file_text,
                                );
                            }

                            if let Some(byte_label) = STATUS_BYTE_LABEL {
                                let byte_text =
                                    format!("Bytes: {}", format_with_thousands(byte_count));
                                let byte_str = create_nsstring(&byte_text);
                                msg_send_void_id(
                                    byte_label.as_ptr(),
                                    Sel::get("setStringValue:").as_ptr(),
                                    byte_str,
                                );
                            }

                            if let Some(line_label) = STATUS_LINE_LABEL {
                                let line_text =
                                    format!("Lines: {}", format_with_thousands(line_count));
                                let line_str = create_nsstring(&line_text);
                                msg_send_void_id(
                                    line_label.as_ptr(),
                                    Sel::get("setStringValue:").as_ptr(),
                                    line_str,
                                );
                            }

                            // Apply orange highlighting with underline to every 5th word
                            if let Some(tv) = TEXT_VIEW {
                                apply_random_underlines(&tv);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Menu action: Open URL input dialog and load content from URL
extern "C" fn open_url(_self: *mut c_void, _sel: *mut c_void, _sender: *mut c_void) {
    unsafe {
        // Create an NSAlert for simple text input
        let ns_alert_class = match ObjCClass::get("NSAlert") {
            Some(c) => c,
            None => return,
        };
        let alert = msg_send_id(ns_alert_class.as_ptr(), Sel::get("alloc").as_ptr());
        let alert = msg_send_id(alert, Sel::get("init").as_ptr());
        let alert = ObjCObject::from_ptr(alert);

        let title = create_nsstring("Load from URL");
        msg_send_void_id(alert.as_ptr(), Sel::get("setMessageText:").as_ptr(), title);

        let info = create_nsstring("Enter a URL to load:");
        msg_send_void_id(
            alert.as_ptr(),
            Sel::get("setInformativeText:").as_ptr(),
            info,
        );

        // Add OK and Cancel buttons
        let ok_title = create_nsstring("OK");
        msg_send_void_id(
            alert.as_ptr(),
            Sel::get("addButtonWithTitle:").as_ptr(),
            ok_title,
        );

        let cancel_title = create_nsstring("Cancel");
        msg_send_void_id(
            alert.as_ptr(),
            Sel::get("addButtonWithTitle:").as_ptr(),
            cancel_title,
        );

        // Create text field for URL input
        let ns_text_field_class = match ObjCClass::get("NSTextField") {
            Some(c) => c,
            None => return,
        };
        let text_field = msg_send_id(ns_text_field_class.as_ptr(), Sel::get("alloc").as_ptr());
        let text_field = msg_send_id_rect(
            text_field,
            Sel::get("initWithFrame:").as_ptr(),
            NSRect {
                origin: NSPoint { x: 0.0, y: 0.0 },
                size: NSSize {
                    width: 300.0,
                    height: 24.0,
                },
            },
        );
        let text_field = ObjCObject::from_ptr(text_field);

        let placeholder = create_nsstring("https://example.com/file.txt");
        msg_send_void_id(
            text_field.as_ptr(),
            Sel::get("setPlaceholderString:").as_ptr(),
            placeholder,
        );

        msg_send_void_id(
            alert.as_ptr(),
            Sel::get("setAccessoryView:").as_ptr(),
            text_field.as_ptr(),
        );

        // Show the alert
        type MsgSendInt = extern "C" fn(*mut c_void, *mut c_void) -> i64;
        let f: MsgSendInt = transmute(objc_msgSend as *const ());
        let result = f(alert.as_ptr(), Sel::get("runModal").as_ptr());

        if result == 1000 {
            // NSAlertFirstButtonReturn
            // Get text field content
            let url_str = msg_send_id(text_field.as_ptr(), Sel::get("stringValue").as_ptr());
            if !url_str.is_null() {
                let url_cstr: *const c_char = {
                    let f: extern "C" fn(*mut c_void, *mut c_void) -> *const c_char =
                        transmute(objc_msgSend as *const ());
                    f(url_str, Sel::get("UTF8String").as_ptr())
                };

                if !url_cstr.is_null() {
                    let url = CStr::from_ptr(url_cstr).to_string_lossy().to_string();

                    // Spawn blocking task to load URL content
                    spawn(move || {
                        load_url_content(&url);
                    });
                }
            }
        }
    }
}

/// Blocking function to fetch content from a URL
/// This runs on a background thread - just fetch, don't update UI
fn load_url_content(url: &str) {
    // Validate URL format
    if !url.starts_with("http://") && !url.starts_with("https://") {
        eprintln!("Invalid URL: {}", url);
        return;
    }

    eprintln!("Loading URL: {}", url);

    match ureq::get(url).call() {
        Ok(response) => {
            match response.into_string() {
                Ok(content) => {
                    eprintln!("Successfully loaded {} bytes from {}", content.len(), url);
                    // URL loading is complete - user will see content loaded
                    // (In a real app, would dispatch to main thread to update UI)
                }
                Err(e) => {
                    eprintln!("Failed to read response body: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch URL: {}", e);
        }
    }
}

/// Show a status alert dialog (must be called from main thread or use thread-safe approach)
fn show_status_alert(title: &str, message: &str) {
    unsafe {
        let ns_alert_class = match ObjCClass::get("NSAlert") {
            Some(c) => c,
            None => return,
        };
        let alert = msg_send_id(ns_alert_class.as_ptr(), Sel::get("alloc").as_ptr());
        let alert = msg_send_id(alert, Sel::get("init").as_ptr());
        let alert = ObjCObject::from_ptr(alert);

        let title_str = create_nsstring(title);
        msg_send_void_id(
            alert.as_ptr(),
            Sel::get("setMessageText:").as_ptr(),
            title_str,
        );

        let message_str = create_nsstring(message);
        msg_send_void_id(
            alert.as_ptr(),
            Sel::get("setInformativeText:").as_ptr(),
            message_str,
        );

        let ok = create_nsstring("OK");
        msg_send_void_id(alert.as_ptr(), Sel::get("addButtonWithTitle:").as_ptr(), ok);

        type MsgSendInt = extern "C" fn(*mut c_void, *mut c_void) -> i64;
        let f: MsgSendInt = transmute(objc_msgSend as *const ());
        let _ = f(alert.as_ptr(), Sel::get("runModal").as_ptr());
    }
}

/// App delegate callback - called when app finishes launching
extern "C" fn app_did_finish_launching(
    _self: *mut c_void,
    _sel: *mut c_void,
    _notification: *mut c_void,
) {
    unsafe {
        let app_class = match ObjCClass::get("NSApplication") {
            Some(c) => c,
            None => {
                return;
            }
        };
        let app = msg_send_id(app_class.as_ptr(), Sel::get("sharedApplication").as_ptr());
        if app.is_null() {
            return;
        }
        let app = ObjCObject::from_ptr(app);

        // Create the window
        let window = create_window();

        // Store window globally for title updates
        WINDOW = Some(window.clone());

        // Add UI elements to the window (this creates the views but doesn't layout yet)
        setup_window_ui(&window);

        // Set button targets to delegate (now that delegate exists)
        if let Some(button) = OPEN_BUTTON {
            if let Some(delegate) = APP_DELEGATE {
                msg_send_void_id(
                    button.as_ptr(),
                    Sel::get("setTarget:").as_ptr(),
                    delegate.as_ptr(),
                );
            }
        }

        if let Some(button) = URL_BUTTON {
            if let Some(delegate) = APP_DELEGATE {
                msg_send_void_id(
                    button.as_ptr(),
                    Sel::get("setTarget:").as_ptr(),
                    delegate.as_ptr(),
                );
            }
        }

        // Show the window
        msg_send_void_id(
            window.as_ptr(),
            Sel::get("makeKeyAndOrderFront:").as_ptr(),
            null_mut(),
        );

        // Activate the app after window is shown
        msg_send_void_int(
            app.as_ptr(),
            Sel::get("activateIgnoringOtherApps:").as_ptr(),
            1,
        );

        // Now layout the UI after window is shown (so bounds are correct)
        layout_ui_elements(&window);
    }
}

/// Create app delegate class with applicationDidFinishLaunching: callback
/// Create custom NSTextView subclass that handles pinch zoom
fn create_custom_text_view_class() -> *mut c_void {
    unsafe {
        // Check if class already exists
        let class_name = CString::new("CustomTextView").unwrap();
        let existing_class = objc_getClass(class_name.as_ptr());

        if !existing_class.is_null() {
            return existing_class;
        }

        // Create new class inheriting from NSTextView
        let ns_text_view = match ObjCClass::get("NSTextView") {
            Some(c) => c,
            None => {
                // Fallback to standard NSTextView
                return ObjCClass::get("NSTextView")
                    .map(|c| c.as_ptr())
                    .unwrap_or(null_mut());
            }
        };

        let text_view_class = objc_allocateClassPair(ns_text_view.as_ptr(), class_name.as_ptr(), 0);

        if text_view_class.is_null() {
            return ns_text_view.as_ptr();
        }

        // Add magnifyWithEvent: method
        let method_types = CString::new("v@:@").unwrap();
        let imp = magnify_with_event as *mut c_void;
        class_addMethod(
            text_view_class,
            Sel::get("magnifyWithEvent:").as_ptr() as *mut c_void,
            imp,
            method_types.as_ptr(),
        );

        // Add mouseEntered: method for hover detection
        let entered_types = CString::new("v@:@").unwrap();
        let entered_imp = mouse_entered as *mut c_void;
        class_addMethod(
            text_view_class,
            Sel::get("mouseEntered:").as_ptr() as *mut c_void,
            entered_imp,
            entered_types.as_ptr(),
        );

        // Add mouseExited: method for hover detection
        let exited_types = CString::new("v@:@").unwrap();
        let exited_imp = mouse_exited as *mut c_void;
        class_addMethod(
            text_view_class,
            Sel::get("mouseExited:").as_ptr() as *mut c_void,
            exited_imp,
            exited_types.as_ptr(),
        );

        objc_registerClassPair(text_view_class);
        text_view_class
    }
}

fn create_app_delegate(_app: &ObjCObject) -> ObjCObject {
    unsafe {
        // Try to get existing class first
        let class_name = CString::new("AppDelegate").unwrap();
        let existing_class = objc_getClass(class_name.as_ptr());

        let delegate_class = if !existing_class.is_null() {
            // Class already exists, reuse it
            existing_class
        } else {
            // Create delegate class dynamically
            let ns_object = ObjCClass::get("NSObject").unwrap();
            let delegate_class = objc_allocateClassPair(ns_object.as_ptr(), class_name.as_ptr(), 0);

            if delegate_class.is_null() {
                panic!("Failed to allocate AppDelegate class pair");
            }

            // Add applicationDidFinishLaunching: method
            let method_types = CString::new("v@:@").unwrap();
            let imp = app_did_finish_launching as *mut c_void;
            class_addMethod(
                delegate_class,
                Sel::get("applicationDidFinishLaunching:").as_ptr(),
                imp,
                method_types.as_ptr(),
            );

            // Add open_file: method
            let open_file_types = CString::new("v@:@").unwrap();
            let open_file_imp = open_file as *mut c_void;
            class_addMethod(
                delegate_class,
                Sel::get("open_file:").as_ptr(),
                open_file_imp,
                open_file_types.as_ptr(),
            );

            // Add open_url: method
            let open_url_types = CString::new("v@:@").unwrap();
            let open_url_imp = open_url as *mut c_void;
            class_addMethod(
                delegate_class,
                Sel::get("open_url:").as_ptr(),
                open_url_imp,
                open_url_types.as_ptr(),
            );

            objc_registerClassPair(delegate_class);
            delegate_class
        };

        // Create instance
        let alloc = msg_send_id(delegate_class as *mut c_void, Sel::get("alloc").as_ptr());
        let delegate = msg_send_id(alloc, Sel::get("init").as_ptr());
        ObjCObject::from_ptr(delegate)
    }
}

/// Create a window delegate that handles resize events
fn create_window_delegate_with_layout(_window: &ObjCObject) -> ObjCObject {
    unsafe {
        // Try to get existing class first
        let class_name = CString::new("WindowDelegate").unwrap();
        let existing_class = objc_getClass(class_name.as_ptr());

        let delegate_class = if !existing_class.is_null() {
            // Class already exists, reuse it
            existing_class
        } else {
            // Create delegate class dynamically
            let ns_object = ObjCClass::get("NSObject").unwrap();
            let delegate_class = objc_allocateClassPair(ns_object.as_ptr(), class_name.as_ptr(), 0);

            if delegate_class.is_null() {
                panic!("Failed to allocate WindowDelegate class pair");
            }

            // Add windowShouldClose: method
            let method_types = CString::new("I@:@").unwrap();
            let imp = window_should_close as *mut c_void;
            class_addMethod(
                delegate_class,
                Sel::get("windowShouldClose:").as_ptr(),
                imp,
                method_types.as_ptr(),
            );

            // Add windowDidResize: method
            let method_types_resize = CString::new("v@:@").unwrap();
            let imp_resize = window_did_resize as *mut c_void;
            class_addMethod(
                delegate_class,
                Sel::get("windowDidResize:").as_ptr(),
                imp_resize,
                method_types_resize.as_ptr(),
            );

            objc_registerClassPair(delegate_class);
            delegate_class
        };

        // Create instance
        let alloc = msg_send_id(delegate_class as *mut c_void, Sel::get("alloc").as_ptr());
        let delegate = msg_send_id(alloc, Sel::get("init").as_ptr());
        ObjCObject::from_ptr(delegate)
    }
}

/// Callback for windowShouldClose: - called when window close button is clicked
extern "C" fn window_should_close(
    _self: *mut c_void,
    _sel: *mut c_void,
    _sender: *mut c_void,
) -> u32 {
    unsafe {
        let app_class = ObjCClass::get("NSApplication").unwrap();
        let app = msg_send_id(app_class.as_ptr(), Sel::get("sharedApplication").as_ptr());
        msg_send_void_id(app, Sel::get("terminate:").as_ptr(), null_mut());
    }
    1 // Return YES
}

/// Callback for windowDidResize: - called when window is resized
extern "C" fn window_did_resize(_self: *mut c_void, _sel: *mut c_void, _notification: *mut c_void) {
    unsafe {
        // Get the window from the notification
        let window = {
            type MsgSendId = extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void;
            let f: MsgSendId = transmute(objc_msgSend as *const ());
            f(_notification, Sel::get("object").as_ptr())
        };
        let window = ObjCObject::from_ptr(window);
        layout_ui_elements(&window);
    }
}

/// Callback for magnification gesture (pinch zoom)
/// Must be careful with panics since this crosses the FFI boundary
extern "C" fn magnify_with_event(_self: *mut c_void, _sel: *mut c_void, event: *mut c_void) {
    // Wrap entire callback in catch_unwind to prevent unwinding across FFI
    let _ = std::panic::catch_unwind(|| {
        unsafe {
            if event.is_null() {
                return;
            }

            let event = ObjCObject::from_ptr(event);

            // Get magnification value from event
            let magnification = {
                type MsgSendDouble = extern "C" fn(*mut c_void, *mut c_void) -> f64;
                let f_mag: MsgSendDouble = transmute(objc_msgSend as *const ());
                let sel = Sel::get("magnification");
                if sel.is_null() {
                    return;
                }
                f_mag(event.as_ptr(), sel.as_ptr())
            };

            // Update font size based on magnification
            let new_size = TEXT_VIEW_FONT_SIZE * (1.0 + magnification);
            TEXT_VIEW_FONT_SIZE = if new_size < 8.0 {
                8.0
            } else if new_size > 48.0 {
                48.0
            } else {
                new_size
            };

            // Apply the new size to the text view
            if let Some(text_view) = TEXT_VIEW {
                set_text_view_font_size(&text_view, TEXT_VIEW_FONT_SIZE);
            }
        }
    });
}

/// Setup gesture recognizers for the text view (pinch zoom, etc.)
fn setup_text_view_gestures(text_view: &ObjCObject) {
    unsafe {
        // Create NSMagnificationGestureRecognizer
        let recognizer_class = match ObjCClass::get("NSMagnificationGestureRecognizer") {
            Some(c) => c,
            None => return,
        };

        // Alloc and init: [NSMagnificationGestureRecognizer alloc]
        let recognizer = msg_send_id(recognizer_class.as_ptr(), Sel::get("alloc").as_ptr());

        // Set target and action before init
        // We need to init with a target/action, so use initWithTarget:action:
        type MsgSendIdIdSelector =
            extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void) -> *mut c_void;
        let f: MsgSendIdIdSelector = transmute(objc_msgSend as *const ());

        if let Some(text_view_obj) = TEXT_VIEW {
            let sel = Sel::get("magnifyWithEvent:");
            let recognizer = f(
                recognizer,
                Sel::get("initWithTarget:action:").as_ptr(),
                text_view_obj.as_ptr(),
                sel.as_ptr(),
            );

            if !recognizer.is_null() {
                // Add the recognizer to the text view: [textView addGestureRecognizer:recognizer]
                msg_send_void_id(
                    text_view.as_ptr(),
                    Sel::get("addGestureRecognizer:").as_ptr(),
                    recognizer,
                );
            }
        }
    }
}
