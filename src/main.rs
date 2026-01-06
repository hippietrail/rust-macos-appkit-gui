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
        extern "C" {
            fn objc_msgSend() -> NSRect;
        }
        let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> NSRect = 
            std::mem::transmute(ffi::objc_msgSend as *const ());
        f(obj, sel)
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
        extern "C" {
            fn objc_msgSend(obj: *mut std::ffi::c_void, sel: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        }
        objc_msgSend(shared_app.as_ptr(), Sel::get("finishLaunching").as_ptr());
    }
    
    // Create the window
    let window = create_window();
    
    // Show the window
    msg_send_void_id(window.as_ptr(), Sel::get("makeKeyAndOrderFront:").as_ptr(), std::ptr::null_mut());
    
    // Activate the app
    msg_send_void_int(shared_app.as_ptr(), Sel::get("activateIgnoringOtherApps:").as_ptr(), 1);
    
    // Run the event loop (no arguments needed)
    unsafe {
        extern "C" {
            fn objc_msgSend(obj: *mut std::ffi::c_void, sel: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        }
        objc_msgSend(shared_app.as_ptr(), Sel::get("run").as_ptr());
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
        let method_name = std::ffi::CString::new("windowShouldClose:").unwrap();
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

/// Callback for windowShouldClose: - called when window close button is clicked
extern "C" fn window_should_close(_self: *mut std::ffi::c_void, _sel: *mut std::ffi::c_void, _sender: *mut std::ffi::c_void) -> u32 {
    unsafe {
        let app_class = ObjCClass::get("NSApplication").unwrap();
        let app = msg_send_id(app_class.as_ptr(), Sel::get("sharedApplication").as_ptr());
        msg_send_void_id(app, Sel::get("terminate:").as_ptr(), std::ptr::null_mut());
    }
    1 // Return YES
}
