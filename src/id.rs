use std::sync::atomic::{AtomicUsize, Ordering};

use gpui::{ElementId, SharedString};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ComponentId {
    root: ElementId,
    key: String,
}

impl ComponentId {
    pub fn new(root: impl Into<ElementId>) -> Self {
        let root = root.into();
        let key = root.to_string();
        Self { root, key }
    }

    pub fn id(&self) -> &ElementId {
        &self.root
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn slot(&self, name: impl Into<String>) -> ElementId {
        (self.root.clone(), name.into()).into()
    }

    pub fn slot_index(&self, slot: &str, key: impl Into<String>) -> ElementId {
        (self.slot(slot.to_owned()), key.into()).into()
    }
}

impl Default for ComponentId {
    fn default() -> Self {
        let sequence = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self::new(format!("component-{sequence}"))
    }
}

impl std::fmt::Display for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.key.fmt(f)
    }
}

impl std::ops::Deref for ComponentId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.key()
    }
}

impl AsRef<str> for ComponentId {
    fn as_ref(&self) -> &str {
        self.key()
    }
}

impl From<ComponentId> for ElementId {
    fn from(value: ComponentId) -> Self {
        value.root
    }
}

impl From<&ComponentId> for ElementId {
    fn from(value: &ComponentId) -> Self {
        value.root.clone()
    }
}

impl From<ElementId> for ComponentId {
    fn from(value: ElementId) -> Self {
        Self::new(value)
    }
}

impl From<&ElementId> for ComponentId {
    fn from(value: &ElementId) -> Self {
        Self::new(value.clone())
    }
}

impl From<String> for ComponentId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for ComponentId {
    fn from(value: &str) -> Self {
        Self::new(value.to_owned())
    }
}

impl From<ComponentId> for String {
    fn from(value: ComponentId) -> Self {
        value.key
    }
}

impl From<&ComponentId> for String {
    fn from(value: &ComponentId) -> Self {
        value.key.clone()
    }
}

impl From<ComponentId> for SharedString {
    fn from(value: ComponentId) -> Self {
        value.key.into()
    }
}

impl From<&ComponentId> for SharedString {
    fn from(value: &ComponentId) -> Self {
        value.key.clone().into()
    }
}
