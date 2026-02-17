use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

static BOOL_STATE: LazyLock<Mutex<HashMap<String, bool>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static TEXT_STATE: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static OPT_TEXT_STATE: LazyLock<Mutex<HashMap<String, Option<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static LIST_STATE: LazyLock<Mutex<HashMap<String, Vec<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
pub const FOCUSED_SLOT: &str = "focused";

fn key(id: &str, slot: &str) -> String {
    format!("{id}::{slot}")
}

pub fn bool_state(id: &str, slot: &str, controlled: Option<bool>, default: bool) -> bool {
    if let Some(value) = controlled {
        return value;
    }

    let composed = key(id, slot);
    if let Ok(mut state) = BOOL_STATE.lock() {
        return *state.entry(composed).or_insert(default);
    }
    default
}

pub fn set_bool_state(id: &str, slot: &str, value: bool) {
    let composed = key(id, slot);
    if let Ok(mut state) = BOOL_STATE.lock() {
        state.insert(composed, value);
    }
}

pub fn focused_state(id: &str, controlled: Option<bool>, default: bool) -> bool {
    bool_state(id, FOCUSED_SLOT, controlled, default)
}

pub fn set_focused_state(id: &str, value: bool) {
    set_bool_state(id, FOCUSED_SLOT, value);
}

pub fn is_activation_key(key: &str) -> bool {
    key == "space" || key == "enter"
}

pub fn text_state(id: &str, slot: &str, controlled: Option<String>, default: String) -> String {
    if let Some(value) = controlled {
        return value;
    }

    let composed = key(id, slot);
    if let Ok(mut state) = TEXT_STATE.lock() {
        return state.entry(composed).or_insert(default).clone();
    }
    default
}

pub fn set_text_state(id: &str, slot: &str, value: String) {
    let composed = key(id, slot);
    if let Ok(mut state) = TEXT_STATE.lock() {
        state.insert(composed, value);
    }
}

pub fn optional_text_state(
    id: &str,
    slot: &str,
    controlled: Option<Option<String>>,
    default: Option<String>,
) -> Option<String> {
    if let Some(value) = controlled {
        return value;
    }

    let composed = key(id, slot);
    if let Ok(mut state) = OPT_TEXT_STATE.lock() {
        return state.entry(composed).or_insert(default).clone();
    }
    default
}

pub fn set_optional_text_state(id: &str, slot: &str, value: Option<String>) {
    let composed = key(id, slot);
    if let Ok(mut state) = OPT_TEXT_STATE.lock() {
        state.insert(composed, value);
    }
}

pub fn list_state(
    id: &str,
    slot: &str,
    controlled: Option<Vec<String>>,
    default: Vec<String>,
) -> Vec<String> {
    if let Some(value) = controlled {
        return value;
    }

    let composed = key(id, slot);
    if let Ok(mut state) = LIST_STATE.lock() {
        return state.entry(composed).or_insert(default).clone();
    }
    default
}

pub fn set_list_state(id: &str, slot: &str, value: Vec<String>) {
    let composed = key(id, slot);
    if let Ok(mut state) = LIST_STATE.lock() {
        state.insert(composed, value);
    }
}
