// Minimal macOS ObjC FFI boundary
// This module contains only the bare essentials for calling Objective-C from Rust.

use std::ffi::{c_void, c_char};

// ============================================================================
// ObjC Runtime Types
// ============================================================================

/// Opaque pointer to an Objective-C object
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct ObjCObject(*mut c_void);

/// Opaque pointer to an Objective-C class
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct ObjCClass(*mut c_void);

/// Opaque pointer to an Objective-C selector
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Sel(*mut c_void);

unsafe impl Send for ObjCObject {}
unsafe impl Sync for ObjCObject {}
unsafe impl Send for ObjCClass {}
unsafe impl Sync for ObjCClass {}

// ============================================================================
// ObjC Runtime Functions (direct C bindings)
// ============================================================================

extern "C" {
    /// Get a class by name
    pub fn objc_getClass(name: *const c_char) -> *mut c_void;
    
    /// Get a selector by name
    pub fn sel_getUid(name: *const c_char) -> *mut c_void;
    
    /// Send a message - raw variadic function (use with casting for correct ABI)
    #[link_name = "objc_msgSend"]
    pub fn objc_msgSend(obj: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
    
    /// Allocate a new class pair
    pub fn objc_allocateClassPair(
        superclass: *mut c_void,
        name: *const c_char,
        extraBytes: usize,
    ) -> *mut c_void;
    
    /// Register a newly created class
    pub fn objc_registerClassPair(cls: *mut c_void);
    
    /// Add a method to a class
    pub fn class_addMethod(
        cls: *mut c_void,
        name: *mut c_void,
        imp: *mut c_void,
        types: *const c_char,
    ) -> bool;
}

/// Represents an NSRect/CGRect structure  
/// Used for frame positioning and sizing
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NSRect {
    pub origin: NSPoint,
    pub size: NSSize,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NSPoint {
    pub x: f64,
    pub y: f64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NSSize {
    pub width: f64,
    pub height: f64,
}

// ============================================================================
// Safe wrappers for basic operations
// ============================================================================

/// Type-safe function pointers for common objc_msgSend signatures
pub mod msg_send_signatures {
    use super::*;
    
    /// id objc_msgSend(id obj, SEL sel)
    pub type MsgSendId = extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void;
    
    /// void objc_msgSend(id obj, SEL sel, id arg)
    pub type MsgSendVoidId = extern "C" fn(*mut c_void, *mut c_void, *mut c_void);
    
    /// void objc_msgSend(id obj, SEL sel, int arg)
    pub type MsgSendVoidInt = extern "C" fn(*mut c_void, *mut c_void, i32);
    
    /// id objc_msgSend(id obj, SEL sel, NSRect frame)
    pub type MsgSendIdRect = extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void;
    
    /// id objc_msgSend(id obj, SEL sel, NSRect frame, int styleMask, int backing, int defer)
    pub type MsgSendIdRectIntIntInt = extern "C" fn(*mut c_void, *mut c_void, NSRect, i32, i32, i32) -> *mut c_void;
    
    /// void objc_msgSend(id obj, SEL sel, bool arg)
    pub type MsgSendVoidBool = extern "C" fn(*mut c_void, *mut c_void, bool);
    
    /// void objc_msgSend(id obj, SEL sel, NSRect frame)
    pub type MsgSendVoidRect = extern "C" fn(*mut c_void, *mut c_void, NSRect);
    
    /// id objc_msgSend(id obj, SEL sel, int index)
    pub type MsgSendIdInt = extern "C" fn(*mut c_void, *mut c_void, i32) -> *mut c_void;
}

impl ObjCClass {
    /// Get a class by name (e.g., "NSApplication")
    pub fn get(name: &str) -> Option<Self> {
        let c_name = std::ffi::CString::new(name).ok()?;
        let ptr = unsafe { objc_getClass(c_name.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(ObjCClass(ptr))
        }
    }
    
    /// Get the raw pointer
    pub fn as_ptr(&self) -> *mut c_void {
        self.0
    }
}

impl Sel {
    /// Get a selector by name (e.g., "alloc")
    pub fn get(name: &str) -> Self {
        let c_name = std::ffi::CString::new(name).unwrap();
        let ptr = unsafe { sel_getUid(c_name.as_ptr()) };
        Sel(ptr)
    }
    
    /// Get the raw pointer
    pub fn as_ptr(&self) -> *mut c_void {
        self.0
    }
}

impl ObjCObject {
    /// Create from raw pointer
    pub fn from_ptr(ptr: *mut c_void) -> Self {
        ObjCObject(ptr)
    }
    
    /// Get the raw pointer (if you need it for direct FFI calls)
    pub fn as_ptr(&self) -> *mut c_void {
        self.0
    }
    
    /// Send a message that returns void
    pub unsafe fn send_msg(&self, sel: Sel) {
        objc_msgSend(self.0, sel.0);
    }
    
    /// Send a message that returns an ObjCObject
    pub unsafe fn send_msg_object(&self, sel: Sel) -> Option<ObjCObject> {
        let result = objc_msgSend(self.0, sel.0);
        if result.is_null() {
            None
        } else {
            Some(ObjCObject(result))
        }
    }
    
    /// Send a message with one argument that returns an ObjCObject
    pub unsafe fn send_msg_object_arg(&self, sel: Sel, arg: ObjCObject) -> Option<ObjCObject> {
        let result = objc_msgSend(self.0, sel.0, arg.0);
        if result.is_null() {
            None
        } else {
            Some(ObjCObject(result))
        }
    }
}
