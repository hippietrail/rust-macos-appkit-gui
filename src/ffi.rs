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
    
    /// Send a message to an object (returns void/generic value)
    /// This is the core variadic function - callers handle return value casting
    #[link_name = "objc_msgSend"]
    pub fn objc_msgSend(obj: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
    
    /// Send a message to a class (void return)
    pub fn objc_msgSend_stret(obj: *mut c_void, sel: *mut c_void, ...);
    
    /// Allocate memory for a new instance
    pub fn objc_allocateClassPair(
        superclass: *mut c_void,
        name: *const c_char,
        extraBytes: usize,
    ) -> *mut c_void;
    
    /// Register a newly created class
    pub fn objc_registerClassPair(cls: *mut c_void);
}

// ============================================================================
// Safe wrappers for basic operations
// ============================================================================

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
