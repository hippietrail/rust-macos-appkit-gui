I'll investigate the repository to check for a rich-text capable textview, Harper grammar checker usage, and pinch/zoom gesture handling.Let me try with a different approach to explore the repository structure:Based on my review of the code:

## Rich-Text TextView

**No, this does NOT use a rich-text capable TextView.** The code uses `NSTextView` (lines 733, 1538-1545) which is created as a basic text view. The text view is set to read-only (line 747) and there's no configuration for rich-text attributes like font styling, colors beyond spell-check simulation, or formatting controls. The "highlighting" you see (lines 155-309) is just simulated spell-check using yellow backgrounds and red text on every 5th word—not genuine rich-text editing capabilities.

## Harper Grammar Checker

**Harper is included but NOT used yet.** It's declared as a dependency in `Cargo.toml` (line 7: `harper-core = "1.3.0"`) but I found **zero references** to `harper` anywhere in the Rust code. The highlighting system (lines 155-309) is completely custom, using hardcoded logic to highlight every 5th word—not actual grammar checking.

## Pinch/Zoom Gesture Issues

**There are several issues with the pinch/zoom handling:**

1. **Missing gesture recognizer initialization** (lines 1768-1804): The `setup_text_view_gestures()` function creates an `NSMagnificationGestureRecognizer` but:
   - It tries to use `initWithTarget:action:` (lines 1786-1792) but never calls it—just allocates and then tries to add it
   - The actual initialization appears incomplete

2. **No gesture state tracking** (lines 1728-1765): The `magnify_with_event` callback doesn't distinguish between `.began`, `.changed`, and `.ended` states. It applies font size changes on every event, which could cause:
   - Flickering if events fire rapidly
   - No smooth progressive zooming
   - Difficult to control zoom precision

3. **Gesture recognizer not properly added** (lines 1795-1800): The recognizer is added but the initialization order is questionable—should init the recognizer *before* adding it

4. **No max/min enforcement during gesture** (lines 1750-1757): Clamping happens after, but the gesture state isn't validated

**Quick fix suggestions:**
- Complete the gesture recognizer initialization: call `init` before adding to view
- Add gesture state handling to only respond to `.changed` state
- Validate that the magnification event properties are being read correctly (the callback should check if `magnification` is being retrieved properly)
- Consider multiplying magnification incrementally rather than on cumulative values if the gesture fires multiple times

----

Looking at the code and the screenshot, here are the likely culprits for why highlighting isn't visible:

## Primary Issues

1. **`apply_random_underlines()` is only called on file load** (line 1285)
   - It's only invoked in the `open_file` callback after loading a file
   - The sample text shown on startup (lines 782-807) never gets highlighting applied
   - Try opening a file to see if highlighting appears there

2. **Text view is read-only** (line 747)
   - `setEditable:` is set to `false`
   - This shouldn't prevent highlighting, but combined with other issues it might cause attribute rendering problems

3. **Highlighting uses attributed string attributes that may not render**
   - Lines 272-291 apply `NSBackgroundColorAttributeName` and `NSForegroundColorAttributeName`
   - These *should* work on `NSTextStorage`, but there might be a rendering issue
   - The text view needs to actually display these attributes

4. **`setNeedsDisplay:` may not trigger a redraw** (lines 302-306)
   - Calling `setNeedsDisplay:` on the text view itself might not be sufficient
   - The scroll view or its document view might need to be marked as needing display

5. **UTF-16 conversion could be silently failing**
   - The mapping between UTF-8 and UTF-16 (lines 193-204) is complex
   - If byte position mapping is off, ranges might be invalid and silently skipped

## Debugging Suggestions

Try adding this modified version to diagnose:

```rust
/// Apply random red highlighting to words in the text view (DEBUGGING VERSION)
fn apply_random_underlines(text_view: &ObjCObject) {
    unsafe {
        let text_storage = msg_send_id(text_view.as_ptr(), Sel::get("textStorage").as_ptr());
        if text_storage.is_null() {
            eprintln!("❌ HIGHLIGHT: text_storage is null!");
            return;
        }
        let text_storage = ObjCObject::from_ptr(text_storage);

        let text_str = msg_send_id(text_storage.as_ptr(), Sel::get("string").as_ptr());
        if text_str.is_null() {
            eprintln!("❌ HIGHLIGHT: text_str is null!");
            return;
        }

        type MsgSendUsize = extern "C" fn(*mut c_void, *mut c_void) -> usize;
        let f_len: MsgSendUsize = transmute(objc_msgSend as *const ());
        let utf16_len = f_len(text_str, Sel::get("length").as_ptr());
        eprintln!("📝 HIGHLIGHT: UTF-16 length = {}", utf16_len);

        if utf16_len == 0 {
            eprintln!("❌ HIGHLIGHT: String is empty!");
            return;
        }

        // ... rest of code ...

        eprintln!("✅ HIGHLIGHT: Applied to {} words out of {}", applied_count, words.len());
        
        // Try marking scroll view for redraw instead
        if let Some((_, scroll_view, _)) = UI_ELEMENTS {
            msg_send_void_bool(
                scroll_view.as_ptr(),
                Sel::get("setNeedsDisplay:").as_ptr(),
                true,
            );
            eprintln!("✅ HIGHLIGHT: Marked scroll view for redraw");
        }
    }
}
```

## Most Likely Fix

Replace the `setNeedsDisplay:` call (lines 302-306) with:

```rust
// Force redraw of the scroll view instead of just the text view
unsafe {
    if let Some((_, scroll_view, _)) = UI_ELEMENTS {
        msg_send_void_bool(
            scroll_view.as_ptr(),
            Sel::get("setNeedsDisplay:").as_ptr(),
            true,
        );
    }
}
```

Or add this to invalidate the text layout:

```rust
// Invalidate layout
let layout_manager = msg_send_id(
    text_storage.as_ptr(),
    Sel::get("layoutManagers").as_ptr()
);
```

**Test with a loaded file first** — open the markdown file in your screenshot to see if highlighting works on loaded content. If it does, the issue is that `apply_random_underlines()` isn't being called on the sample text.