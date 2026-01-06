mod ffi;

use ffi::{ObjCClass, ObjCObject, Sel, NSRect, NSPoint, NSSize};
use ffi::msg_send_signatures::{self, *};

/// Cast objc_msgSend to the right type and call it
/// This ensures the compiler uses the correct calling convention
fn msg_send_id(obj: *mut std::ffi::c_void, sel: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    unsafe {
        let f: MsgSendId = std::mem::transmute(ffi::objc_msgSend as *const ());
        f(obj, sel)
    }
}

fn msg_send_void_id(obj: *mut std::ffi::c_void, sel: *mut std::ffi::c_void, arg: *mut std::ffi::c_void) {
    unsafe {
        let f: MsgSendVoidId = std::mem::transmute(ffi::objc_msgSend as *const ());
        f(obj, sel, arg)
    }
}

fn msg_send_void_int(obj: *mut std::ffi::c_void, sel: *mut std::ffi::c_void, arg: i32) {
    unsafe {
        let f: MsgSendVoidInt = std::mem::transmute(ffi::objc_msgSend as *const ());
        f(obj, sel, arg)
    }
}

fn msg_send_id_rect(obj: *mut std::ffi::c_void, sel: *mut std::ffi::c_void, rect: NSRect) -> *mut std::ffi::c_void {
    unsafe {
        let f: MsgSendIdRect = std::mem::transmute(ffi::objc_msgSend as *const ());
        f(obj, sel, rect)
    }
}

fn msg_send_id_rect_int_int_int(
    obj: *mut std::ffi::c_void,
    sel: *mut std::ffi::c_void,
    rect: NSRect,
    style: i32,
    backing: i32,
    defer: i32,
) -> *mut std::ffi::c_void {
    unsafe {
        let f: msg_send_signatures::MsgSendIdRectIntIntInt = std::mem::transmute(ffi::objc_msgSend as *const ());
        f(obj, sel, rect, style, backing, defer)
    }
}

/// Get screen frame (NSRect)
fn msg_send_rect(obj: *mut std::ffi::c_void, sel: *mut std::ffi::c_void) -> NSRect {
    unsafe {
        let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> NSRect = 
            std::mem::transmute(ffi::objc_msgSend as *const ());
        f(obj, sel)
    }
}

fn msg_send_void_bool(obj: *mut std::ffi::c_void, sel: *mut std::ffi::c_void, arg: bool) {
    unsafe {
        let f: MsgSendVoidBool = std::mem::transmute(ffi::objc_msgSend as *const ());
        f(obj, sel, arg)
    }
}

/// Helper to create NSString from Rust string
fn create_nsstring(s: &str) -> *mut std::ffi::c_void {
    let c_str = match std::ffi::CString::new(s) {
        Ok(c) => c,
        Err(_) => return std::ptr::null_mut(),
    };
    let ns_string_class = match ObjCClass::get("NSString") {
        Some(c) => c,
        None => return std::ptr::null_mut(),
    };
    let f: msg_send_signatures::MsgSendIdCStr = unsafe { 
        std::mem::transmute(ffi::objc_msgSend as *const ())
    };
    f(ns_string_class.as_ptr(), Sel::get("stringWithUTF8String:").as_ptr(), c_str.as_ptr())
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
    type MsgSendIdIdIdId = extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    let quit_item = unsafe {
        let f: MsgSendIdIdIdId = std::mem::transmute(ffi::objc_msgSend as *const ());
        f(quit_item, Sel::get("initWithTitle:action:keyEquivalent:").as_ptr(), quit_title_obj, Sel::get("terminate:").as_ptr(), q_str)
    };
    let quit_item = ObjCObject::from_ptr(quit_item);
    
    // Add quit item to app submenu
    msg_send_void_id(app_submenu.as_ptr(), Sel::get("addItem:").as_ptr(), quit_item.as_ptr());
    
    // Create app menu item (with no title initially)
    let app_menu_item = msg_send_id(ns_menu_item_class.as_ptr(), Sel::get("alloc").as_ptr());
    let app_menu_item = msg_send_id(app_menu_item, Sel::get("init").as_ptr());
    let app_menu_item = ObjCObject::from_ptr(app_menu_item);
    
    // Set submenu on app menu item
    msg_send_void_id(app_menu_item.as_ptr(), Sel::get("setSubmenu:").as_ptr(), app_submenu.as_ptr());
    
    // Add app menu item to main menu
    msg_send_void_id(main_menu.as_ptr(), Sel::get("addItem:").as_ptr(), app_menu_item.as_ptr());
    
    // Create File menu
    let file_submenu = msg_send_id(ns_menu_class.as_ptr(), Sel::get("alloc").as_ptr());
    let file_submenu = msg_send_id(file_submenu, Sel::get("init").as_ptr());
    let file_submenu = ObjCObject::from_ptr(file_submenu);
    
    // Create Open menu item
    let open_title_obj = create_nsstring("Open");
    let open_item = msg_send_id(ns_menu_item_class.as_ptr(), Sel::get("alloc").as_ptr());
    let o_str = create_nsstring("o");
    
    let open_item = unsafe {
        let f: MsgSendIdIdIdId = std::mem::transmute(ffi::objc_msgSend as *const ());
        f(open_item, Sel::get("initWithTitle:action:keyEquivalent:").as_ptr(), open_title_obj, Sel::get("open_file:").as_ptr(), o_str)
    };
    let open_item = ObjCObject::from_ptr(open_item);
    
    // Set target for the Open menu item
    msg_send_void_id(open_item.as_ptr(), Sel::get("setTarget:").as_ptr(), app_delegate.as_ptr());
    
    // Add open item to file submenu
    msg_send_void_id(file_submenu.as_ptr(), Sel::get("addItem:").as_ptr(), open_item.as_ptr());
    
    // Create File menu item
    let file_menu_item = msg_send_id(ns_menu_item_class.as_ptr(), Sel::get("alloc").as_ptr());
    let file_menu_item = msg_send_id(file_menu_item, Sel::get("init").as_ptr());
    let file_menu_item = ObjCObject::from_ptr(file_menu_item);
    
    // Set submenu on file menu item
    let file_title = create_nsstring("File");
    msg_send_void_id(file_menu_item.as_ptr(), Sel::get("setTitle:").as_ptr(), file_title);
    msg_send_void_id(file_menu_item.as_ptr(), Sel::get("setSubmenu:").as_ptr(), file_submenu.as_ptr());
    
    // Add file menu item to main menu
    msg_send_void_id(main_menu.as_ptr(), Sel::get("addItem:").as_ptr(), file_menu_item.as_ptr());
    
    // Set main menu on app
    msg_send_void_id(app.as_ptr(), Sel::get("setMainMenu:").as_ptr(), main_menu.as_ptr());
}

fn main() {
    let ns_app_class = ObjCClass::get("NSApplication")
        .expect("Failed to get NSApplication class");
    
    let shared_app = msg_send_id(ns_app_class.as_ptr(), Sel::get("sharedApplication").as_ptr());
    let shared_app = ObjCObject::from_ptr(shared_app);
    
    // Set activation policy to Regular (makes app appear in Dock and be frontmost)
    msg_send_void_int(shared_app.as_ptr(), Sel::get("setActivationPolicy:").as_ptr(), 0);
    
    // Create and set app delegate BEFORE finishLaunching so callback gets triggered
    let app_delegate = create_app_delegate(&shared_app);
    msg_send_void_id(shared_app.as_ptr(), Sel::get("setDelegate:").as_ptr(), app_delegate.as_ptr());
    
    // Store app delegate globally for menu actions
    unsafe {
        APP_DELEGATE = Some(app_delegate.clone());
    }
    
    // Setup main menu with Quit option and Open action BEFORE finishLaunching
    setup_menu(&shared_app, &app_delegate);
    
    // finishLaunching triggers applicationDidFinishLaunching: callback which creates window
    unsafe {
        let f: MsgSendId = std::mem::transmute(ffi::objc_msgSend as *const ());
        f(shared_app.as_ptr(), Sel::get("finishLaunching").as_ptr());
    }
    
    // Run the event loop (no arguments needed)
    unsafe {
        let f: MsgSendId = std::mem::transmute(ffi::objc_msgSend as *const ());
        f(shared_app.as_ptr(), Sel::get("run").as_ptr());
    }
}

fn create_window() -> ObjCObject {
    // Get main screen and its frame
    let ns_screen_class = ObjCClass::get("NSScreen")
        .expect("Failed to get NSScreen class");
    let main_screen = msg_send_id(ns_screen_class.as_ptr(), Sel::get("mainScreen").as_ptr());
    let screen_frame = msg_send_rect(main_screen, Sel::get("frame").as_ptr());
    
    // Calculate window size as 3/4 of screen
    let frame = NSRect {
        origin: NSPoint { x: screen_frame.size.width * 0.125, y: screen_frame.size.height * 0.125 },
        size: NSSize {
            width: screen_frame.size.width * 0.75,
            height: screen_frame.size.height * 0.75,
        },
    };
    
    let ns_window_class = ObjCClass::get("NSWindow")
        .expect("Failed to get NSWindow class");
    
    let alloc_window = msg_send_id(ns_window_class.as_ptr(), Sel::get("alloc").as_ptr());
    
    let window = msg_send_id_rect_int_int_int(
        alloc_window,
        Sel::get("initWithContentRect:styleMask:backing:defer:").as_ptr(),
        frame,
        15,  // styleMask: Titled | Closable | Miniaturizable | Resizable
        2,   // backing: Buffered
        0,   // defer: NO
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
    msg_send_void_id(window.as_ptr(), Sel::get("setDelegate:").as_ptr(), delegate.as_ptr());
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

/// Global reference to window (for updating title)
static mut WINDOW: Option<ObjCObject> = None;

/// Create the UI elements (toolbar, text view, status bar)
fn create_ui_elements(content_view: &ObjCObject) -> (ObjCObject, ObjCObject, ObjCObject) {
    let ns_view_class = ObjCClass::get("NSView").expect("Failed to get NSView class");
    let ns_color_class = ObjCClass::get("NSColor").expect("Failed to get NSColor class");
    
    // Create toolbar
    let toolbar = msg_send_id(ns_view_class.as_ptr(), Sel::get("alloc").as_ptr());
    let toolbar = msg_send_id_rect(toolbar, Sel::get("initWithFrame:").as_ptr(), NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size: NSSize { width: 100.0, height: 44.0 },
    });
    let toolbar = ObjCObject::from_ptr(toolbar);
    let light_gray = msg_send_id(ns_color_class.as_ptr(), Sel::get("lightGrayColor").as_ptr());
    msg_send_void_id(toolbar.as_ptr(), Sel::get("setBackgroundColor:").as_ptr(), light_gray);
    msg_send_void_id(content_view.as_ptr(), Sel::get("addSubview:").as_ptr(), toolbar.as_ptr());
    
    // Add Open button to toolbar
    let ns_button_class = ObjCClass::get("NSButton").expect("Failed to get NSButton class");
    let open_button = msg_send_id(ns_button_class.as_ptr(), Sel::get("alloc").as_ptr());
    let open_button = msg_send_id_rect(open_button, Sel::get("initWithFrame:").as_ptr(), NSRect {
        origin: NSPoint { x: 10.0, y: 10.0 },
        size: NSSize { width: 80.0, height: 24.0 },
    });
    let open_button = ObjCObject::from_ptr(open_button);
    
    // Set button title
    let open_btn_title = create_nsstring("Open");
    msg_send_void_id(open_button.as_ptr(), Sel::get("setTitle:").as_ptr(), open_btn_title);
    
    // Set button action
    msg_send_void_id(open_button.as_ptr(), Sel::get("setAction:").as_ptr(), Sel::get("open_file:").as_ptr());
    
    // Store button globally so we can set target later
    unsafe {
        OPEN_BUTTON = Some(open_button.clone());
    }
    
    // Add button to toolbar
    msg_send_void_id(toolbar.as_ptr(), Sel::get("addSubview:").as_ptr(), open_button.as_ptr());
    
    // Create scroll view containing text view
    let ns_scroll_view_class = ObjCClass::get("NSScrollView").expect("Failed to get NSScrollView class");
    let scroll_view = msg_send_id(ns_scroll_view_class.as_ptr(), Sel::get("alloc").as_ptr());
    let scroll_view = msg_send_id_rect(scroll_view, Sel::get("initWithFrame:").as_ptr(), NSRect {
        origin: NSPoint { x: 0.0, y: 20.0 },
        size: NSSize { width: 100.0, height: 100.0 },
    });
    let scroll_view = ObjCObject::from_ptr(scroll_view);
    
    // Configure scroll view
    msg_send_void_bool(scroll_view.as_ptr(), Sel::get("setHasVerticalScroller:").as_ptr(), true);
    
    // Create text view
    let ns_text_view_class = match ObjCClass::get("NSTextView") {
        Some(c) => c,
        None => return (toolbar, scroll_view, ObjCObject::from_ptr(std::ptr::null_mut())),
    };
    let text_view = msg_send_id(ns_text_view_class.as_ptr(), Sel::get("alloc").as_ptr());
    let text_view = msg_send_id_rect(text_view, Sel::get("initWithFrame:").as_ptr(), NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size: NSSize { width: 100.0, height: 100.0 },
    });
    let text_view = ObjCObject::from_ptr(text_view);
    msg_send_void_bool(text_view.as_ptr(), Sel::get("setEditable:").as_ptr(), false);
    
    // Allow text view to be wider than scroll view for horizontal scrolling
    msg_send_void_bool(text_view.as_ptr(), Sel::get("setHorizontallyResizable:").as_ptr(), true);
    msg_send_void_bool(text_view.as_ptr(), Sel::get("setVerticallyResizable:").as_ptr(), true);
    
    // Get text container and configure it
    let text_container = msg_send_id(text_view.as_ptr(), Sel::get("textContainer").as_ptr());
    if !text_container.is_null() {
        let text_container = ObjCObject::from_ptr(text_container);
        // Set container size to a very large width but let height be unlimited
        type MsgSendVoidSize = extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, NSSize);
        unsafe {
            let f: MsgSendVoidSize = std::mem::transmute(ffi::objc_msgSend as *const ());
            let huge_size = NSSize { width: 100000.0, height: 100000.0 };
            f(text_container.as_ptr(), Sel::get("setContainerSize:").as_ptr(), huge_size);
        }
    }
    
    // Add sample text
    let sample_text = match std::ffi::CString::new("Sample Text View\n\nThis is a read-only text view with vertical scrolling support.") {
        Ok(s) => s,
        Err(_) => return (toolbar, scroll_view, text_view),
    };
    let text_obj = {
        let ns_string_class = match ObjCClass::get("NSString") {
            Some(c) => c,
            None => return (toolbar, scroll_view, text_view),
        };
        type MsgSendIdCStr = extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *const std::ffi::c_char) -> *mut std::ffi::c_void;
        unsafe {
            let f: MsgSendIdCStr = std::mem::transmute(ffi::objc_msgSend as *const ());
            f(ns_string_class.as_ptr(), Sel::get("stringWithUTF8String:").as_ptr(), sample_text.as_ptr())
        }
    };
    msg_send_void_id(text_view.as_ptr(), Sel::get("setString:").as_ptr(), text_obj);
    
    // Add text view to scroll view
    msg_send_void_id(scroll_view.as_ptr(), Sel::get("setDocumentView:").as_ptr(), text_view.as_ptr());
    
    // Add scroll view to content view
    msg_send_void_id(content_view.as_ptr(), Sel::get("addSubview:").as_ptr(), scroll_view.as_ptr());
    
    // Create status bar
    let status_bar = msg_send_id(ns_view_class.as_ptr(), Sel::get("alloc").as_ptr());
    let status_bar = msg_send_id_rect(status_bar, Sel::get("initWithFrame:").as_ptr(), NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size: NSSize { width: 100.0, height: 20.0 },
    });
    let status_bar = ObjCObject::from_ptr(status_bar);
    let dark_gray = msg_send_id(ns_color_class.as_ptr(), Sel::get("darkGrayColor").as_ptr());
    msg_send_void_id(status_bar.as_ptr(), Sel::get("setBackgroundColor:").as_ptr(), dark_gray);
    msg_send_void_id(content_view.as_ptr(), Sel::get("addSubview:").as_ptr(), status_bar.as_ptr());
    
    // Store references globally for layout updates and file loading
    unsafe {
        UI_ELEMENTS = Some((toolbar, scroll_view, status_bar));
        TEXT_VIEW = Some(text_view.clone());
    }
    
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
                origin: NSPoint { x: 0.0, y: content_height - toolbar_height },
                size: NSSize { width: content_width, height: toolbar_height },
            };
            set_view_frame(toolbar, toolbar_frame);
            
            // Status bar at bottom (y = 0)
            let status_bar_frame = NSRect {
                origin: NSPoint { x: 0.0, y: 0.0 },
                size: NSSize { width: content_width, height: status_bar_height },
            };
            set_view_frame(status_bar, status_bar_frame);
            
            // Text view in middle (y = status_bar_height, height = rest)
            let text_view_frame = NSRect {
                origin: NSPoint { x: 0.0, y: status_bar_height },
                size: NSSize { width: content_width, height: text_view_height },
            };
            set_view_frame(text_view, text_view_frame);
            
            // Update text container width to match scroll view (for proper text wrapping on resize)
            if let Some(text_view_obj) = TEXT_VIEW {
                let text_container = msg_send_id(text_view_obj.as_ptr(), Sel::get("textContainer").as_ptr());
                if !text_container.is_null() {
                    let text_container = ObjCObject::from_ptr(text_container);
                    type MsgSendVoidSize = extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, NSSize);
                    let f: MsgSendVoidSize = std::mem::transmute(ffi::objc_msgSend as *const ());
                    // Set width to current scroll view width, height unlimited
                    let new_size = NSSize { width: content_width, height: 100000.0 };
                    f(text_container.as_ptr(), Sel::get("setContainerSize:").as_ptr(), new_size);
                }
            }
        }
    }
}

/// Set frame for a view
fn set_view_frame(view: ObjCObject, frame: NSRect) {
    unsafe {
        let f: msg_send_signatures::MsgSendVoidRect = std::mem::transmute(ffi::objc_msgSend as *const ());
        f(view.as_ptr(), Sel::get("setFrame:").as_ptr(), frame);
    }
}

/// Menu action: Open file dialog and load file
extern "C" fn open_file(_self: *mut std::ffi::c_void, _sel: *mut std::ffi::c_void, _sender: *mut std::ffi::c_void) {
    unsafe {
        let ns_open_panel_class = match ObjCClass::get("NSOpenPanel") {
            Some(c) => c,
            None => return,
        };
        let open_panel = msg_send_id(ns_open_panel_class.as_ptr(), Sel::get("openPanel").as_ptr());
        let open_panel = ObjCObject::from_ptr(open_panel);
        
        // Set allowed file types - text-like formats: txt, md, html, xml, css, json, rtf, pdf, etc.
        let extensions = vec!["txt", "md", "markdown", "html", "htm", "xml", "css", "json", "js", "ts", "rs", "py", "rb", "go", "c", "h", "cpp", "java", "rtf", "pdf"];
        
        let allowed_types = {
            let ns_array_class = match ObjCClass::get("NSArray") {
                Some(c) => c,
                None => return,
            };
            let ns_array_ptr = ns_array_class.as_ptr();
            
            // Create array with all extensions
            type MsgSendIdIdArray = extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void;
            let f_array: MsgSendIdIdArray = std::mem::transmute(ffi::objc_msgSend as *const ());
            
            let mut array = msg_send_id(ns_array_ptr, Sel::get("array").as_ptr());
            
            // Add each extension to the array
            type MsgSendIdIdIdId = extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void;
            let f_add: MsgSendIdIdIdId = std::mem::transmute(ffi::objc_msgSend as *const ());
            
            for ext in extensions {
                let ext_str = create_nsstring(ext);
                array = f_add(array, Sel::get("arrayByAddingObject:").as_ptr(), ext_str);
            }
            
            array
        };
        msg_send_void_id(open_panel.as_ptr(), Sel::get("setAllowedFileTypes:").as_ptr(), allowed_types);
        
        // Show the dialog
        type MsgSendInt = extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> i64;
        let f: MsgSendInt = std::mem::transmute(ffi::objc_msgSend as *const ());
        let result = f(open_panel.as_ptr(), Sel::get("runModal").as_ptr());
        
        if result == 1 { // NSOKButton
            // Get selected file URL
            let url = msg_send_id(open_panel.as_ptr(), Sel::get("URL").as_ptr());
            if !url.is_null() {
                // Get path from URL
                let path = msg_send_id(url, Sel::get("path").as_ptr());
                if !path.is_null() {
                    // Convert NSString path to Rust string
                    let path_cstr: *const std::ffi::c_char = unsafe {
                        let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *const std::ffi::c_char = 
                            std::mem::transmute(ffi::objc_msgSend as *const ());
                        f(path, Sel::get("UTF8String").as_ptr())
                    };
                    
                    // Read file contents
                    if !path_cstr.is_null() {
                        let file_path = std::ffi::CStr::from_ptr(path_cstr).to_string_lossy();
                        if let Ok(contents) = std::fs::read_to_string(file_path.as_ref()) {
                            // Set text view content
                            if let Some(text_view) = TEXT_VIEW {
                                let content_str = create_nsstring(&contents);
                                msg_send_void_id(text_view.as_ptr(), Sel::get("setString:").as_ptr(), content_str);
                                
                                // Re-sync text container after loading to ensure proper layout
                                if let Some(window) = WINDOW {
                                    let content_view = msg_send_id(window.as_ptr(), Sel::get("contentView").as_ptr());
                                    let content_bounds = msg_send_rect(content_view, Sel::get("bounds").as_ptr());
                                    let scroll_view_width = content_bounds.size.width;
                                    
                                    let text_container = msg_send_id(text_view.as_ptr(), Sel::get("textContainer").as_ptr());
                                    if !text_container.is_null() {
                                        let text_container = ObjCObject::from_ptr(text_container);
                                        type MsgSendVoidSize = extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, NSSize);
                                        let f: MsgSendVoidSize = std::mem::transmute(ffi::objc_msgSend as *const ());
                                        let new_size = NSSize { width: scroll_view_width, height: 100000.0 };
                                        f(text_container.as_ptr(), Sel::get("setContainerSize:").as_ptr(), new_size);
                                    }
                                }
                            }
                            
                            // Update window title with filename
                            if let Some(window) = WINDOW {
                                // Extract filename from full path
                                let path_str = file_path.as_ref();
                                let filename = path_str.split('/').last().unwrap_or(path_str);
                                let title_str = create_nsstring(filename);
                                msg_send_void_id(window.as_ptr(), Sel::get("setTitle:").as_ptr(), title_str);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// App delegate callback - called when app finishes launching
extern "C" fn app_did_finish_launching(_self: *mut std::ffi::c_void, _sel: *mut std::ffi::c_void, _notification: *mut std::ffi::c_void) {
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
        
        // Set button target to delegate (now that delegate exists)
        if let Some(button) = OPEN_BUTTON {
            if let Some(delegate) = APP_DELEGATE {
                msg_send_void_id(button.as_ptr(), Sel::get("setTarget:").as_ptr(), delegate.as_ptr());
            }
        }
        
        // Show the window
        msg_send_void_id(window.as_ptr(), Sel::get("makeKeyAndOrderFront:").as_ptr(), std::ptr::null_mut());
        
        // Activate the app after window is shown
        msg_send_void_int(app.as_ptr(), Sel::get("activateIgnoringOtherApps:").as_ptr(), 1);
        
        // Now layout the UI after window is shown (so bounds are correct)
        layout_ui_elements(&window);
    }
}

/// Create app delegate class with applicationDidFinishLaunching: callback
fn create_app_delegate(_app: &ObjCObject) -> ObjCObject {
    unsafe {
        // Try to get existing class first
        let class_name = std::ffi::CString::new("AppDelegate").unwrap();
        let existing_class = ffi::objc_getClass(class_name.as_ptr());
        
        let delegate_class = if !existing_class.is_null() {
            // Class already exists, reuse it
            existing_class
        } else {
            // Create delegate class dynamically
            let ns_object = ObjCClass::get("NSObject").unwrap();
            let delegate_class = ffi::objc_allocateClassPair(ns_object.as_ptr(), class_name.as_ptr(), 0);
            
            if delegate_class.is_null() {
                panic!("Failed to allocate AppDelegate class pair");
            }
            
            // Add applicationDidFinishLaunching: method
            let method_types = std::ffi::CString::new("v@:@").unwrap();
            let imp = app_did_finish_launching as *mut std::ffi::c_void;
            ffi::class_addMethod(delegate_class, Sel::get("applicationDidFinishLaunching:").as_ptr(), imp, method_types.as_ptr());
            
            // Add open_file: method
            let open_file_types = std::ffi::CString::new("v@:@").unwrap();
            let open_file_imp = open_file as *mut std::ffi::c_void;
            ffi::class_addMethod(delegate_class, Sel::get("open_file:").as_ptr(), open_file_imp, open_file_types.as_ptr());
            
            ffi::objc_registerClassPair(delegate_class);
            delegate_class
        };
        
        // Create instance
        let alloc = msg_send_id(delegate_class as *mut std::ffi::c_void, Sel::get("alloc").as_ptr());
        let delegate = msg_send_id(alloc, Sel::get("init").as_ptr());
        ObjCObject::from_ptr(delegate)
    }
}

/// Create a window delegate that handles resize events
fn create_window_delegate_with_layout(window: &ObjCObject) -> ObjCObject {
    unsafe {
        // Try to get existing class first
        let class_name = std::ffi::CString::new("WindowDelegate").unwrap();
        let existing_class = ffi::objc_getClass(class_name.as_ptr());
        
        let delegate_class = if !existing_class.is_null() {
            // Class already exists, reuse it
            existing_class
        } else {
            // Create delegate class dynamically
            let ns_object = ObjCClass::get("NSObject").unwrap();
            let delegate_class = ffi::objc_allocateClassPair(ns_object.as_ptr(), class_name.as_ptr(), 0);
            
            if delegate_class.is_null() {
                panic!("Failed to allocate WindowDelegate class pair");
            }
            
            // Add windowShouldClose: method
            let method_types = std::ffi::CString::new("I@:@").unwrap();
            let imp = window_should_close as *mut std::ffi::c_void;
            ffi::class_addMethod(delegate_class, Sel::get("windowShouldClose:").as_ptr(), imp, method_types.as_ptr());
            
            // Add windowDidResize: method
            let method_types_resize = std::ffi::CString::new("v@:@").unwrap();
            let imp_resize = window_did_resize as *mut std::ffi::c_void;
            ffi::class_addMethod(delegate_class, Sel::get("windowDidResize:").as_ptr(), imp_resize, method_types_resize.as_ptr());
            
            ffi::objc_registerClassPair(delegate_class);
            delegate_class
        };
        
        // Create instance
        let alloc = msg_send_id(delegate_class as *mut std::ffi::c_void, Sel::get("alloc").as_ptr());
        let delegate = msg_send_id(alloc, Sel::get("init").as_ptr());
        ObjCObject::from_ptr(delegate)
    }
}

/// Callback for windowShouldClose: - called when window close button is clicked
extern "C" fn window_should_close(_self: *mut std::ffi::c_void, _sel: *mut std::ffi::c_void, _sender: *mut std::ffi::c_void) -> u32 {
    unsafe {
        let app_class = ObjCClass::get("NSApplication").unwrap();
        let app = msg_send_id(app_class.as_ptr(), Sel::get("sharedApplication").as_ptr());
        msg_send_void_id(app, Sel::get("terminate:").as_ptr(), std::ptr::null_mut());
    }
    1 // Return YES
}

/// Callback for windowDidResize: - called when window is resized
extern "C" fn window_did_resize(_self: *mut std::ffi::c_void, _sel: *mut std::ffi::c_void, _notification: *mut std::ffi::c_void) {
    unsafe {
        // Get the window from the notification
        let window = {
            type MsgSendId = extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void;
            let f: MsgSendId = std::mem::transmute(ffi::objc_msgSend as *const ());
            f(_notification, Sel::get("object").as_ptr())
        };
        let window = ObjCObject::from_ptr(window);
        layout_ui_elements(&window);
    }
}
