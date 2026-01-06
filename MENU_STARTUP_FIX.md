# Menu Bar Not Showing on Startup - Fix & Key Findings

## The Problem
Menu bar was invisible when the app launched. It only appeared after switching focus to another app and back.

## What Didn't Work
- Moving menu creation before `finishLaunching`
- Calling `activateIgnoringOtherApps:` immediately after `finishLaunching`
- Calling `activateIgnoringOtherApps:` after `makeKeyAndOrderFront:`

These were all correct individually, but the timing of window creation was wrong.

## The Critical Fix

**Window creation and activation must happen INSIDE the `applicationDidFinishLaunching:` delegate callback, not in `main()` after `finishLaunching` returns.**

### Correct Sequence

```rust
fn main() {
    // 1. Get app instance
    let shared_app = get_shared_application();
    
    // 2. Set activation policy early
    msg_send_void_int(shared_app.as_ptr(), Sel::get("setActivationPolicy:").as_ptr(), 0);
    
    // 3. Create menu BEFORE finishLaunching
    setup_menu(&shared_app);
    
    // 4. Create and SET app delegate BEFORE finishLaunching
    let app_delegate = create_app_delegate(&shared_app);
    msg_send_void_id(shared_app.as_ptr(), Sel::get("setDelegate:").as_ptr(), app_delegate.as_ptr());
    
    // 5. Call finishLaunching - triggers applicationDidFinishLaunching: callback
    call_finish_launching(&shared_app);
    
    // 6. Run event loop
    call_run(&shared_app);
}

// 7. INSIDE the delegate callback:
extern "C" fn app_did_finish_launching(...) {
    // Create window
    let window = create_window();
    
    // Setup UI
    setup_window_ui(&window);
    
    // Show window
    msg_send_void_id(window.as_ptr(), Sel::get("makeKeyAndOrderFront:").as_ptr(), ...);
    
    // Activate app AFTER window is shown
    msg_send_void_int(app.as_ptr(), Sel::get("activateIgnoringOtherApps:").as_ptr(), 1);
    
    // Layout UI
    layout_ui_elements(&window);
}
```

## Why This Works

1. **`setActivationPolicy`** tells macOS the app should appear in the menu bar
2. **Menu created before finishLaunching** ensures menu structure exists
3. **Delegate set before finishLaunching** means callback is registered
4. **finishLaunching triggers the callback** which is the right time for UI setup
5. **Window shown, then app activated** ensures proper rendering order
6. **Event loop starts after callback returns** with everything ready

The key insight: macOS expects app UI setup (windows, activation) to happen during the finishLaunching lifecycle event, not in arbitrary code after it returns.

## Reference
- Working C example: `/Users/hippietrail/rust-macos-gui/menu-without-bundle.c` (from StackOverflow answer)
- Rust implementation: `/Users/hippietrail/rust-macos-gui/src/main.rs`
