use std::sync::atomic::{AtomicU64, Ordering};

use gpui::{ElementId, SharedString};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ComponentId {
    root: ElementId,
    key: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct IdCtx {
    base: ComponentId,
}

impl ComponentId {
    #[track_caller]
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

    pub fn ctx(&self) -> IdCtx {
        IdCtx::from(self)
    }
}

impl IdCtx {
    pub fn new(base: impl Into<ComponentId>) -> Self {
        Self { base: base.into() }
    }

    pub fn base(&self) -> &ComponentId {
        &self.base
    }

    pub fn root<T>(&self, node: T) -> T
    where
        T: crate::contracts::WithId,
    {
        crate::contracts::WithId::with_id(node, self.base.clone())
    }

    pub fn child<T>(&self, slot: impl Into<String>, node: T) -> T
    where
        T: crate::contracts::WithId,
    {
        crate::contracts::WithId::with_id(node, self.base.slot(slot))
    }

    pub fn child_index<T>(&self, slot: &str, key: impl Into<String>, node: T) -> T
    where
        T: crate::contracts::WithId,
    {
        crate::contracts::WithId::with_id(node, self.base.slot_index(slot, key))
    }

    pub fn scoped(&self, slot: impl Into<String>) -> Self {
        Self::new(self.base.scoped(slot))
    }

    pub fn scoped_index(&self, slot: &str, key: impl Into<String>) -> Self {
        Self::new(self.base.scoped_index(slot, key))
    }
}

impl From<ComponentId> for IdCtx {
    fn from(value: ComponentId) -> Self {
        Self { base: value }
    }
}

impl From<&ComponentId> for IdCtx {
    fn from(value: &ComponentId) -> Self {
        Self {
            base: value.clone(),
        }
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
    let crate_name = env!("CARGO_PKG_NAME");
    format!(
        "{prefix}|crate={crate_name}|file={}|line={}|col={}",
        location.file(),
        location.line(),
        location.column()
    )
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
    use crate::contracts::WithId;

    #[derive(Clone)]
    struct DummyNode {
        id: ComponentId,
    }

    impl DummyNode {
        fn new() -> Self {
            Self {
                id: ComponentId::stable("dummy-node"),
            }
        }
    }

    impl WithId for DummyNode {
        fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
            self.id = id.into();
            self
        }
    }

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

    #[test]
    fn stable_id_contains_enhanced_seed_fields() {
        let id = call_once();
        assert!(id.contains("button|crate="));
        assert!(id.contains("|file="));
        assert!(id.contains("|line="));
        assert!(id.contains("|col="));
    }

    #[track_caller]
    fn component_id_once() -> String {
        ComponentId::default().to_string()
    }

    #[track_caller]
    fn stable_component_id_once() -> String {
        ComponentId::stable("component").to_string()
    }

    #[test]
    fn component_id_default_is_stable_for_same_callsite() {
        let ids = (0..3).map(|_| component_id_once()).collect::<Vec<_>>();
        assert!(ids.windows(2).all(|pair| pair[0] == pair[1]));
    }

    #[test]
    fn component_id_default_differs_for_different_callsites() {
        let first = component_id_once();
        let second = { ComponentId::default().to_string() };
        assert_ne!(first, second);
    }

    #[test]
    fn component_id_stable_differs_for_different_callsites() {
        let first = stable_component_id_once();
        let second = { ComponentId::stable("component").to_string() };
        assert_ne!(first, second);
    }

    #[test]
    fn id_ctx_assigns_root_and_child_ids() {
        let base = ComponentId::stable("ctx-root");
        let ctx = base.ctx();

        let rooted = ctx.root(DummyNode::new());
        assert_eq!(rooted.id.to_string(), base.to_string());

        let child = ctx.child("icon", DummyNode::new());
        assert_eq!(child.id.to_string(), base.slot("icon").to_string());

        let indexed = ctx.child_index("row", "3", DummyNode::new());
        assert_eq!(
            indexed.id.to_string(),
            base.slot_index("row", "3").to_string()
        );
    }
}
