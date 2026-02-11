use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::motion::MotionConfig;
use gpui::SharedString;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ModalId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Layer {
    Base,
    Dropdown,
    Popover,
    Modal,
    Toast,
    Tooltip,
    DragPreview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModalKind {
    Alert,
    Confirm,
    Prompt,
    Custom,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModalEntry {
    pub id: Option<ModalId>,
    pub kind: ModalKind,
    pub title: SharedString,
    pub body: SharedString,
    pub close_on_escape: bool,
    pub close_on_click_outside: bool,
    pub layer: Layer,
    pub motion: MotionConfig,
}

impl ModalEntry {
    pub fn new(
        kind: ModalKind,
        title: impl Into<SharedString>,
        body: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: None,
            kind,
            title: title.into(),
            body: body.into(),
            close_on_escape: true,
            close_on_click_outside: true,
            layer: Layer::Modal,
            motion: MotionConfig::default(),
        }
    }

    pub fn close_on_escape(mut self, value: bool) -> Self {
        self.close_on_escape = value;
        self
    }

    pub fn close_on_click_outside(mut self, value: bool) -> Self {
        self.close_on_click_outside = value;
        self
    }

    pub fn layer(mut self, value: Layer) -> Self {
        self.layer = value;
        self
    }

    pub fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

#[derive(Default)]
struct ModalState {
    stack: Vec<ModalEntry>,
}

#[derive(Clone, Default)]
pub struct ModalManager {
    next_id: Arc<AtomicU64>,
    state: Arc<RwLock<ModalState>>,
}

impl ModalManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&self, mut entry: ModalEntry) -> ModalId {
        let id = ModalId(self.next_id.fetch_add(1, Ordering::SeqCst) + 1);
        entry.id = Some(id);
        self.state
            .write()
            .expect("modal state poisoned")
            .stack
            .push(entry);
        id
    }

    pub fn update(&self, id: ModalId, mut entry: ModalEntry) -> bool {
        let mut state = self.state.write().expect("modal state poisoned");
        if let Some(current) = state
            .stack
            .iter_mut()
            .find(|candidate| candidate.id == Some(id))
        {
            entry.id = Some(id);
            *current = entry;
            true
        } else {
            false
        }
    }

    pub fn close(&self, id: ModalId) -> bool {
        let mut state = self.state.write().expect("modal state poisoned");
        if let Some(index) = state.stack.iter().position(|entry| entry.id == Some(id)) {
            state.stack.remove(index);
            true
        } else {
            false
        }
    }

    pub fn close_top(&self) -> Option<ModalEntry> {
        self.state
            .write()
            .expect("modal state poisoned")
            .stack
            .pop()
    }

    pub fn close_all(&self) {
        self.state
            .write()
            .expect("modal state poisoned")
            .stack
            .clear();
    }

    pub fn list(&self) -> Vec<ModalEntry> {
        self.state
            .read()
            .expect("modal state poisoned")
            .stack
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modal_manager_uses_stack_order() {
        let manager = ModalManager::new();
        let first_id = manager.open(ModalEntry::new(ModalKind::Alert, "A", "one"));
        let second_id = manager.open(ModalEntry::new(ModalKind::Alert, "B", "two"));
        let stack = manager.list();
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].id, Some(first_id));
        assert_eq!(stack[1].id, Some(second_id));
        let top = manager.close_top().expect("top modal should exist");
        assert_eq!(top.id, Some(second_id));
    }
}
