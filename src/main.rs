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

fn main() {
    let ns_app_class = ObjCClass::get("NSApplication")
        .expect("Failed to get NSApplication class");
    
    let shared_app = msg_send_id(ns_app_class.as_ptr(), Sel::get("sharedApplication").as_ptr());
    let shared_app = ObjCObject::from_ptr(shared_app);
    
    // Set activation policy to Regular (makes app appear in Dock and be frontmost)
    msg_send_void_int(shared_app.as_ptr(), Sel::get("setActivationPolicy:").as_ptr(), 0);
    
    // CRITICAL: finishLaunching must be called before creating windows
    unsafe {
        let f: MsgSendId = std::mem::transmute(ffi::objc_msgSend as *const ());
        f(shared_app.as_ptr(), Sel::get("finishLaunching").as_ptr());
    }
    
    // Create the window
    let window = create_window();
    
    // Add UI elements to the window
    setup_window_ui(&window);
    
    // Show the window
    msg_send_void_id(window.as_ptr(), Sel::get("makeKeyAndOrderFront:").as_ptr(), std::ptr::null_mut());
    
    // Activate the app
    msg_send_void_int(shared_app.as_ptr(), Sel::get("activateIgnoringOtherApps:").as_ptr(), 1);
    
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
    
    // Create and set delegate to handle close button
    let delegate = create_window_delegate();
    msg_send_void_id(window.as_ptr(), Sel::get("setDelegate:").as_ptr(), delegate.as_ptr());
    
    window
}

/// Create a window delegate that quits the app when window closes
fn create_window_delegate() -> ObjCObject {
    unsafe {
        // Create delegate class dynamically
        let ns_object = ObjCClass::get("NSObject").unwrap();
        let class_name = std::ffi::CString::new("WindowDelegate").unwrap();
        
        let delegate_class = ffi::objc_allocateClassPair(ns_object.as_ptr(), class_name.as_ptr(), 0);
        
        // Add windowShouldClose: method
        let _method_name = std::ffi::CString::new("windowShouldClose:").unwrap();
        let method_types = std::ffi::CString::new("I@:@").unwrap(); // I=unsigned int, @=id, :=SEL
        
        let imp = window_should_close as *mut std::ffi::c_void;
        ffi::class_addMethod(delegate_class, Sel::get("windowShouldClose:").as_ptr(), imp, method_types.as_ptr());
        
        ffi::objc_registerClassPair(delegate_class);
        
        // Create instance
        let alloc = msg_send_id(delegate_class as *mut std::ffi::c_void, Sel::get("alloc").as_ptr());
        let delegate = msg_send_id(alloc, Sel::get("init").as_ptr());
        ObjCObject::from_ptr(delegate)
    }
}

/// Setup toolbar, text view, and status bar for the window
fn setup_window_ui(window: &ObjCObject) {
    // Get window content view
    let content_view = msg_send_id(window.as_ptr(), Sel::get("contentView").as_ptr());
    let content_view = ObjCObject::from_ptr(content_view);
    
    // Create UI elements
    let (_toolbar, _text_view, _status_bar) = create_ui_elements(&content_view);
    
    // Layout the UI
    layout_ui_elements(window);
    
    // Create and set delegate that handles resize
    let delegate = create_window_delegate_with_layout(window);
    msg_send_void_id(window.as_ptr(), Sel::get("setDelegate:").as_ptr(), delegate.as_ptr());
}

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
    
    // Create text view
    let text_view = msg_send_id(ObjCClass::get("NSTextView").unwrap().as_ptr(), Sel::get("alloc").as_ptr());
    let text_view = msg_send_id_rect(text_view, Sel::get("initWithFrame:").as_ptr(), NSRect {
        origin: NSPoint { x: 0.0, y: 20.0 },
        size: NSSize { width: 100.0, height: 100.0 },
    });
    let text_view = ObjCObject::from_ptr(text_view);
    msg_send_void_bool(text_view.as_ptr(), Sel::get("setEditable:").as_ptr(), false);
    
    // Add sample text
    let sample_text = std::ffi::CString::new("Sample Text View\n\nThis is a read-only text view with vertical scrolling support.").unwrap();
    let text_obj = {
        type MsgSendIdCStr = extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *const std::ffi::c_char) -> *mut std::ffi::c_void;
        unsafe {
            let f: MsgSendIdCStr = std::mem::transmute(ffi::objc_msgSend as *const ());
            f(ObjCClass::get("NSString").unwrap().as_ptr(), Sel::get("stringWithUTF8String:").as_ptr(), sample_text.as_ptr())
        }
    };
    msg_send_void_id(text_view.as_ptr(), Sel::get("setString:").as_ptr(), text_obj);
    msg_send_void_id(content_view.as_ptr(), Sel::get("addSubview:").as_ptr(), text_view.as_ptr());
    
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
    
    (toolbar, text_view, status_bar)
}

/// Layout UI elements based on window size
fn layout_ui_elements(window: &ObjCObject) {
    let window_frame = msg_send_rect(window.as_ptr(), Sel::get("frame").as_ptr());
    let content_height = window_frame.size.height;
    let content_width = window_frame.size.width;
    
    let content_view = msg_send_id(window.as_ptr(), Sel::get("contentView").as_ptr());
    let content_view = ObjCObject::from_ptr(content_view);
    
    let toolbar_height = 44.0;
    let status_bar_height = 20.0;
    let text_view_height = content_height - toolbar_height - status_bar_height;
    
    // Get subviews
    let subviews = msg_send_id(content_view.as_ptr(), Sel::get("subviews").as_ptr());
    
    // Layout toolbar (top)
    let toolbar_frame = NSRect {
        origin: NSPoint { x: 0.0, y: content_height - toolbar_height },
        size: NSSize { width: content_width, height: toolbar_height },
    };
    layout_view_at_index(subviews, 0, toolbar_frame);
    
    // Layout status bar (bottom)
    let status_bar_frame = NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size: NSSize { width: content_width, height: status_bar_height },
    };
    layout_view_at_index(subviews, 2, status_bar_frame);
    
    // Layout text view (middle)
    let text_view_frame = NSRect {
        origin: NSPoint { x: 0.0, y: status_bar_height },
        size: NSSize { width: content_width, height: text_view_height },
    };
    layout_view_at_index(subviews, 1, text_view_frame);
}

/// Set frame for a view at a specific index in subviews array
fn layout_view_at_index(subviews: *mut std::ffi::c_void, index: i32, frame: NSRect) {
    // Get the view at index
    let view = unsafe {
        let f: msg_send_signatures::MsgSendIdInt = std::mem::transmute(ffi::objc_msgSend as *const ());
        f(subviews, Sel::get("objectAtIndex:").as_ptr(), index)
    };
    let view = ObjCObject::from_ptr(view);
    
    // Set its frame
    unsafe {
        let f: msg_send_signatures::MsgSendVoidRect = std::mem::transmute(ffi::objc_msgSend as *const ());
        f(view.as_ptr(), Sel::get("setFrame:").as_ptr(), frame);
    }
}

/// Create a window delegate that handles resize events
fn create_window_delegate_with_layout(window: &ObjCObject) -> ObjCObject {
    unsafe {
        // Create delegate class dynamically
        let ns_object = ObjCClass::get("NSObject").unwrap();
        let class_name = std::ffi::CString::new("WindowDelegate").unwrap();
        
        let delegate_class = ffi::objc_allocateClassPair(ns_object.as_ptr(), class_name.as_ptr(), 0);
        
        // Add windowShouldClose: method
        let _method_name = std::ffi::CString::new("windowShouldClose:").unwrap();
        let method_types = std::ffi::CString::new("I@:@").unwrap();
        let imp = window_should_close as *mut std::ffi::c_void;
        ffi::class_addMethod(delegate_class, Sel::get("windowShouldClose:").as_ptr(), imp, method_types.as_ptr());
        
        // Add windowDidResize: method
        let method_types_resize = std::ffi::CString::new("v@:@").unwrap(); // v=void, @=id, :=SEL, @=id
        let imp_resize = window_did_resize as *mut std::ffi::c_void;
        ffi::class_addMethod(delegate_class, Sel::get("windowDidResize:").as_ptr(), imp_resize, method_types_resize.as_ptr());
        
        ffi::objc_registerClassPair(delegate_class);
        
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
