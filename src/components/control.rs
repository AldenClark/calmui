use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

#[derive(Default)]
struct ControlStore {
    bools: HashMap<String, bool>,
    texts: HashMap<String, String>,
    optional_texts: HashMap<String, Option<String>>,
    lists: HashMap<String, Vec<String>>,
}

static CONTROL_STORE: LazyLock<Mutex<ControlStore>> =
    LazyLock::new(|| Mutex::new(ControlStore::default()));

pub const FOCUSED_SLOT: &str = "focused";

fn key(id: &str, slot: &str) -> String {
    format!("{id}::{slot}")
}

fn with_store<R>(default: R, f: impl FnOnce(&mut ControlStore) -> R) -> R {
    CONTROL_STORE
        .lock()
        .map(|mut store| f(&mut store))
        .unwrap_or(default)
}

fn bool_state_raw(id: &str, slot: &str, controlled: Option<bool>, default: bool) -> bool {
    if let Some(value) = controlled {
        return value;
    }

    let composed = key(id, slot);
    with_store(default, |store| {
        *store.bools.entry(composed).or_insert(default)
    })
}

fn text_state_raw(id: &str, slot: &str, controlled: Option<String>, default: String) -> String {
    if let Some(value) = controlled {
        return value;
    }

    let composed = key(id, slot);
    with_store(default.clone(), |store| {
        store.texts.entry(composed).or_insert(default).clone()
    })
}

fn optional_text_state_raw(
    id: &str,
    slot: &str,
    controlled: Option<Option<String>>,
    default: Option<String>,
) -> Option<String> {
    if let Some(value) = controlled {
        return value;
    }

    let composed = key(id, slot);
    with_store(default.clone(), |store| {
        store
            .optional_texts
            .entry(composed)
            .or_insert(default)
            .clone()
    })
}

fn list_state_raw(
    id: &str,
    slot: &str,
    controlled: Option<Vec<String>>,
    default: Vec<String>,
) -> Vec<String> {
    if let Some(value) = controlled {
        return value;
    }

    let composed = key(id, slot);
    with_store(default.clone(), |store| {
        store.lists.entry(composed).or_insert(default).clone()
    })
}

pub struct StateScope<'a> {
    id: &'a str,
}

impl<'a> StateScope<'a> {
    pub fn bool(&self, slot: &str, controlled: Option<bool>, default: bool) -> bool {
        bool_state_raw(self.id, slot, controlled, default)
    }

    pub fn set_bool(&self, slot: &str, value: bool) {
        let composed = key(self.id, slot);
        let _ = with_store((), |store| {
            store.bools.insert(composed, value);
        });
    }

    pub fn text(
        &self,
        slot: &str,
        controlled: Option<String>,
        default: impl Into<String>,
    ) -> String {
        text_state_raw(self.id, slot, controlled, default.into())
    }

    pub fn set_text(&self, slot: &str, value: impl Into<String>) {
        let composed = key(self.id, slot);
        let _ = with_store((), |store| {
            store.texts.insert(composed, value.into());
        });
    }

    pub fn optional_text(
        &self,
        slot: &str,
        controlled: Option<Option<String>>,
        default: Option<String>,
    ) -> Option<String> {
        optional_text_state_raw(self.id, slot, controlled, default)
    }

    pub fn set_optional_text(&self, slot: &str, value: Option<String>) {
        let composed = key(self.id, slot);
        let _ = with_store((), |store| {
            store.optional_texts.insert(composed, value);
        });
    }

    pub fn list(
        &self,
        slot: &str,
        controlled: Option<Vec<String>>,
        default: Vec<String>,
    ) -> Vec<String> {
        list_state_raw(self.id, slot, controlled, default)
    }

    pub fn set_list(&self, slot: &str, value: Vec<String>) {
        let composed = key(self.id, slot);
        let _ = with_store((), |store| {
            store.lists.insert(composed, value);
        });
    }

    pub fn f32(&self, slot: &str, controlled: Option<f32>, default: f32) -> f32 {
        if let Some(value) = controlled {
            return value;
        }

        text_state_raw(self.id, slot, None, default.to_string())
            .parse::<f32>()
            .ok()
            .unwrap_or(default)
    }

    pub fn set_f32(&self, slot: &str, value: f32) {
        self.set_text(slot, value.to_string());
    }

    pub fn usize(&self, slot: &str, controlled: Option<usize>, default: usize) -> usize {
        if let Some(value) = controlled {
            return value;
        }

        text_state_raw(self.id, slot, None, default.to_string())
            .parse::<usize>()
            .ok()
            .unwrap_or(default)
    }

    pub fn set_usize(&self, slot: &str, value: usize) {
        self.set_text(slot, value.to_string());
    }

    pub fn optional_f32(
        &self,
        slot: &str,
        controlled: Option<Option<f32>>,
        default: Option<f32>,
    ) -> Option<f32> {
        if let Some(value) = controlled {
            return value;
        }

        self.optional_text(slot, None, default.map(|value| value.to_string()))
            .and_then(|value| value.parse::<f32>().ok())
    }

    pub fn set_optional_f32(&self, slot: &str, value: Option<f32>) {
        self.set_optional_text(slot, value.map(|value| value.to_string()));
    }

    pub fn optional_usize(
        &self,
        slot: &str,
        controlled: Option<Option<usize>>,
        default: Option<usize>,
    ) -> Option<usize> {
        if let Some(value) = controlled {
            return value;
        }

        self.optional_text(slot, None, default.map(|value| value.to_string()))
            .and_then(|value| value.parse::<usize>().ok())
    }

    pub fn set_optional_usize(&self, slot: &str, value: Option<usize>) {
        self.set_optional_text(slot, value.map(|value| value.to_string()));
    }
}

pub fn scope(id: &str) -> StateScope<'_> {
    StateScope { id }
}

pub fn bool_state(id: &str, slot: &str, controlled: Option<bool>, default: bool) -> bool {
    scope(id).bool(slot, controlled, default)
}

pub fn set_bool_state(id: &str, slot: &str, value: bool) {
    scope(id).set_bool(slot, value);
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

pub fn is_plain_keystroke(event: &gpui::KeyDownEvent) -> bool {
    !event.keystroke.modifiers.control
        && !event.keystroke.modifiers.platform
        && !event.keystroke.modifiers.function
        && !event.keystroke.modifiers.alt
}

pub fn is_activation_keystroke(event: &gpui::KeyDownEvent) -> bool {
    is_plain_keystroke(event) && is_activation_key(event.keystroke.key.as_str())
}

pub fn is_escape_keystroke(event: &gpui::KeyDownEvent) -> bool {
    is_plain_keystroke(event) && event.keystroke.key == "escape"
}

pub fn step_direction_from_vertical_key(event: &gpui::KeyDownEvent) -> Option<f64> {
    if !is_plain_keystroke(event) {
        return None;
    }

    let key = event.keystroke.key.as_str();
    if key == "up" {
        Some(1.0)
    } else if key == "down" {
        Some(-1.0)
    } else {
        None
    }
}

pub fn text_state(id: &str, slot: &str, controlled: Option<String>, default: String) -> String {
    scope(id).text(slot, controlled, default)
}

pub fn set_text_state(id: &str, slot: &str, value: String) {
    scope(id).set_text(slot, value);
}

pub fn optional_text_state(
    id: &str,
    slot: &str,
    controlled: Option<Option<String>>,
    default: Option<String>,
) -> Option<String> {
    scope(id).optional_text(slot, controlled, default)
}

pub fn set_optional_text_state(id: &str, slot: &str, value: Option<String>) {
    scope(id).set_optional_text(slot, value);
}

pub fn list_state(
    id: &str,
    slot: &str,
    controlled: Option<Vec<String>>,
    default: Vec<String>,
) -> Vec<String> {
    scope(id).list(slot, controlled, default)
}

pub fn set_list_state(id: &str, slot: &str, value: Vec<String>) {
    scope(id).set_list(slot, value);
}

pub fn f32_state(id: &str, slot: &str, controlled: Option<f32>, default: f32) -> f32 {
    scope(id).f32(slot, controlled, default)
}

pub fn set_f32_state(id: &str, slot: &str, value: f32) {
    scope(id).set_f32(slot, value);
}

pub fn usize_state(id: &str, slot: &str, controlled: Option<usize>, default: usize) -> usize {
    scope(id).usize(slot, controlled, default)
}

pub fn set_usize_state(id: &str, slot: &str, value: usize) {
    scope(id).set_usize(slot, value);
}

pub fn optional_f32_state(
    id: &str,
    slot: &str,
    controlled: Option<Option<f32>>,
    default: Option<f32>,
) -> Option<f32> {
    scope(id).optional_f32(slot, controlled, default)
}

pub fn set_optional_f32_state(id: &str, slot: &str, value: Option<f32>) {
    scope(id).set_optional_f32(slot, value);
}

pub fn optional_usize_state(
    id: &str,
    slot: &str,
    controlled: Option<Option<usize>>,
    default: Option<usize>,
) -> Option<usize> {
    scope(id).optional_usize(slot, controlled, default)
}

pub fn set_optional_usize_state(id: &str, slot: &str, value: Option<usize>) {
    scope(id).set_optional_usize(slot, value);
}

#[allow(dead_code)]
pub fn clear_slot(id: &str, slot: &str) {
    let composed = key(id, slot);
    let _ = with_store((), |store| {
        store.bools.remove(&composed);
        store.texts.remove(&composed);
        store.optional_texts.remove(&composed);
        store.lists.remove(&composed);
    });
}

#[allow(dead_code)]
pub fn clear_component(id: &str) {
    let prefix = format!("{id}::");
    let _ = with_store((), |store| {
        store.bools.retain(|key, _| !key.starts_with(&prefix));
        store.texts.retain(|key, _| !key.starts_with(&prefix));
        store
            .optional_texts
            .retain(|key, _| !key.starts_with(&prefix));
        store.lists.retain(|key, _| !key.starts_with(&prefix));
    });
}

#[allow(dead_code)]
pub fn clear_all() {
    let _ = with_store((), |store| {
        *store = ControlStore::default();
    });
}
