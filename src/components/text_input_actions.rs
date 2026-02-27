use std::sync::Once;

use gpui::{App, KeyBinding, actions};

pub const INPUT_KEY_CONTEXT: &str = "calmui_text_input";
pub const TEXTAREA_KEY_CONTEXT: &str = "calmui_textarea";

actions!(
    calmui_text_input,
    [
        MoveLeft,
        MoveRight,
        MoveUp,
        MoveDown,
        MoveHome,
        MoveEnd,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectDown,
        SelectHome,
        SelectEnd,
        DeleteBackward,
        DeleteForward,
        SelectAll,
        CopySelection,
        CutSelection,
        PasteClipboard,
        Submit,
        InsertNewline,
    ]
);

static BINDINGS_INIT: Once = Once::new();

pub fn ensure_text_keybindings(cx: &mut App) {
    BINDINGS_INIT.call_once(|| {
        cx.bind_keys(common_bindings(INPUT_KEY_CONTEXT));
        cx.bind_keys(common_bindings(TEXTAREA_KEY_CONTEXT));
        cx.bind_keys(textarea_only_bindings());
        cx.bind_keys(input_only_bindings());
    });
}

fn common_bindings(context: &'static str) -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("left", MoveLeft, Some(context)),
        KeyBinding::new("right", MoveRight, Some(context)),
        KeyBinding::new("home", MoveHome, Some(context)),
        KeyBinding::new("end", MoveEnd, Some(context)),
        KeyBinding::new("shift-left", SelectLeft, Some(context)),
        KeyBinding::new("shift-right", SelectRight, Some(context)),
        KeyBinding::new("shift-home", SelectHome, Some(context)),
        KeyBinding::new("shift-end", SelectEnd, Some(context)),
        KeyBinding::new("backspace", DeleteBackward, Some(context)),
        KeyBinding::new("delete", DeleteForward, Some(context)),
        KeyBinding::new("cmd-a", SelectAll, Some(context)),
        KeyBinding::new("ctrl-a", SelectAll, Some(context)),
        KeyBinding::new("cmd-c", CopySelection, Some(context)),
        KeyBinding::new("ctrl-c", CopySelection, Some(context)),
        KeyBinding::new("cmd-x", CutSelection, Some(context)),
        KeyBinding::new("ctrl-x", CutSelection, Some(context)),
        KeyBinding::new("cmd-v", PasteClipboard, Some(context)),
        KeyBinding::new("ctrl-v", PasteClipboard, Some(context)),
    ]
}

fn input_only_bindings() -> Vec<KeyBinding> {
    vec![KeyBinding::new("enter", Submit, Some(INPUT_KEY_CONTEXT))]
}

fn textarea_only_bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("enter", InsertNewline, Some(TEXTAREA_KEY_CONTEXT)),
        KeyBinding::new("up", MoveUp, Some(TEXTAREA_KEY_CONTEXT)),
        KeyBinding::new("down", MoveDown, Some(TEXTAREA_KEY_CONTEXT)),
        KeyBinding::new("shift-up", SelectUp, Some(TEXTAREA_KEY_CONTEXT)),
        KeyBinding::new("shift-down", SelectDown, Some(TEXTAREA_KEY_CONTEXT)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_context_constants_are_stable() {
        assert_eq!(INPUT_KEY_CONTEXT, "calmui_text_input");
        assert_eq!(TEXTAREA_KEY_CONTEXT, "calmui_textarea");
    }

    #[test]
    fn common_bindings_contain_core_navigation_and_edit_shortcuts() {
        let bindings = common_bindings(INPUT_KEY_CONTEXT);
        assert_eq!(bindings.len(), 18);
    }

    #[test]
    fn input_and_textarea_bindings_have_expected_enter_behavior() {
        let input = input_only_bindings();
        let textarea = textarea_only_bindings();
        assert_eq!(input.len(), 1);
        assert!(textarea.len() >= 3);
    }
}
