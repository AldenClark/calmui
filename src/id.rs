use std::sync::atomic::{AtomicU64, Ordering};

use gpui::{ElementId, SharedString};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ComponentId {
    root: ElementId,
    key: String,
}

impl ComponentId {
    pub fn stable(prefix: &str) -> Self {
        Self::new(stable_auto_id(prefix))
    }

    pub fn unique(prefix: &str) -> Self {
        static AUTO_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        let next = AUTO_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::new(format!("{prefix}-{next:016x}"))
    }

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

    pub fn scoped(&self, name: impl Into<String>) -> Self {
        Self::new(self.slot(name))
    }

    pub fn scoped_index(&self, slot: &str, key: impl Into<String>) -> Self {
        Self::new(self.slot_index(slot, key))
    }
}

impl Default for ComponentId {
    #[track_caller]
    fn default() -> Self {
        // Keep callsite-stable default for uncontrolled-state continuity.
        // For repeated dynamic instances, prefer explicit with_id(...) or ComponentId::unique(...).
        Self::stable("component")
    }
}

#[track_caller]
pub fn stable_auto_id(prefix: &str) -> String {
    let location = std::panic::Location::caller();
    let seed = format!(
        "{prefix}:{}:{}:{}",
        location.file(),
        location.line(),
        location.column()
    );
    format!("{prefix}-{:016x}", fnv1a64(seed.as_bytes()))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x00000100000001b3;

    let mut hash = OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
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

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn call_once() -> String {
        stable_auto_id("button")
    }

    #[test]
    fn id_is_stable_for_same_callsite() {
        let ids = (0..3).map(|_| call_once()).collect::<Vec<_>>();
        assert!(ids.windows(2).all(|pair| pair[0] == pair[1]));
    }

    #[test]
    fn id_differs_for_different_callsites() {
        let first = call_once();
        let second = {
            // Different callsite by design.
            stable_auto_id("button")
        };
        assert_ne!(first, second);
    }

    #[track_caller]
    fn component_id_once() -> String {
        ComponentId::default().to_string()
    }

    #[test]
    fn component_id_default_is_stable_for_same_callsite() {
        let ids = (0..3).map(|_| component_id_once()).collect::<Vec<_>>();
        assert!(ids.windows(2).all(|pair| pair[0] == pair[1]));
    }
}
