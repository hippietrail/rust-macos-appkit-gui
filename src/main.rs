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
    let frame = NSRect {
        origin: NSPoint { x: 100.0, y: 100.0 },
        size: NSSize { width: 320.0, height: 420.0 },
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
    
    ObjCObject::from_ptr(window)
}
