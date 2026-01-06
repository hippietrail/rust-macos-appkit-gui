mod ffi;

use ffi::{ObjCClass, ObjCObject, Sel};

/// Send a message and return an ObjCObject
macro_rules! msg_send_id {
    ($obj:expr, $sel:expr) => {{
        let sel = Sel::get($sel);
        let result = unsafe { ffi::objc_msgSend($obj.as_ptr(), sel.as_ptr()) };
        ObjCObject::from_ptr(result)
    }};
    ($obj:expr, $sel:expr, $arg1:expr) => {{
        let sel = Sel::get($sel);
        let result = unsafe { ffi::objc_msgSend($obj.as_ptr(), sel.as_ptr(), $arg1) };
        ObjCObject::from_ptr(result)
    }};
}

/// Send a message that returns void
macro_rules! msg_send {
    ($obj:expr, $sel:expr) => {{
        let sel = Sel::get($sel);
        unsafe { ffi::objc_msgSend($obj.as_ptr(), sel.as_ptr()); }
    }};
    ($obj:expr, $sel:expr, $arg1:expr) => {{
        let sel = Sel::get($sel);
        unsafe { ffi::objc_msgSend($obj.as_ptr(), sel.as_ptr(), $arg1); }
    }};
}

fn main() {
    println!("Initializing macOS GUI app...");
    
    let ns_app_class = ObjCClass::get("NSApplication")
        .expect("Failed to get NSApplication class");
    
    let shared_app = msg_send_id!(ns_app_class, "sharedApplication");
    println!("Got NSApplication singleton");
    
    // Activate the app (brings it to foreground)
    msg_send!(shared_app, "activateIgnoringOtherApps:", 1);
    println!("Activated app");
    
    // Run the event loop
    println!("Starting event loop...");
    msg_send!(shared_app, "run");
    
    println!("Exited event loop");
}
