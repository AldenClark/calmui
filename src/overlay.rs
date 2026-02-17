use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::components::Modal;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ModalId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModalKind {
    Custom,
    Info,
    Success,
    Warning,
    Error,
    Confirm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModalCloseReason {
    Programmatic,
    OverlayClick,
    CloseButton,
    EscapeKey,
    ConfirmAction,
    CancelAction,
    CompleteAction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModalStateChange {
    Opened,
    Confirmed,
    Canceled,
    Completed,
    Closed(ModalCloseReason),
}

#[derive(Clone)]
pub struct ManagedModal {
    id: ModalId,
    modal: Arc<Modal>,
}

impl ManagedModal {
    pub fn id(&self) -> ModalId {
        self.id
    }

    pub fn modal(&self) -> &Modal {
        self.modal.as_ref()
    }

    pub fn modal_arc(&self) -> Arc<Modal> {
        self.modal.clone()
    }
}

#[derive(Default)]
struct ModalState {
    stack: Vec<ManagedModal>,
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

    pub fn open(&self, modal: Modal) -> ModalId {
        let id = ModalId(self.next_id.fetch_add(1, Ordering::SeqCst) + 1);
        let modal = Arc::new(modal);

        self.state
            .write()
            .expect("modal state poisoned")
            .stack
            .push(ManagedModal {
                id,
                modal: modal.clone(),
            });

        modal.emit_opened();
        id
    }

    pub fn open_confirm(
        &self,
        title: impl Into<gpui::SharedString>,
        body: impl Into<gpui::SharedString>,
    ) -> ModalId {
        self.open(Modal::confirm(title, body))
    }

    pub fn open_info(
        &self,
        title: impl Into<gpui::SharedString>,
        body: impl Into<gpui::SharedString>,
    ) -> ModalId {
        self.open(Modal::info(title, body))
    }

    pub fn open_success(
        &self,
        title: impl Into<gpui::SharedString>,
        body: impl Into<gpui::SharedString>,
    ) -> ModalId {
        self.open(Modal::success(title, body))
    }

    pub fn open_warning(
        &self,
        title: impl Into<gpui::SharedString>,
        body: impl Into<gpui::SharedString>,
    ) -> ModalId {
        self.open(Modal::warning(title, body))
    }

    pub fn open_error(
        &self,
        title: impl Into<gpui::SharedString>,
        body: impl Into<gpui::SharedString>,
    ) -> ModalId {
        self.open(Modal::error(title, body))
    }

    pub fn update(&self, id: ModalId, modal: Modal) -> bool {
        let mut state = self.state.write().expect("modal state poisoned");
        if let Some(current) = state.stack.iter_mut().find(|entry| entry.id == id) {
            current.modal = Arc::new(modal);
            true
        } else {
            false
        }
    }

    pub fn close(&self, id: ModalId) -> bool {
        self.close_with_reason(id, ModalCloseReason::Programmatic)
    }

    pub fn close_with_reason(&self, id: ModalId, reason: ModalCloseReason) -> bool {
        let closed = self.remove(id);
        if let Some(entry) = closed {
            entry.modal.emit_closed(reason);
            true
        } else {
            false
        }
    }

    pub fn confirm(&self, id: ModalId) -> bool {
        let closed = self.remove(id);
        if let Some(entry) = closed {
            entry.modal.emit_confirmed();
            entry.modal.emit_closed(ModalCloseReason::ConfirmAction);
            true
        } else {
            false
        }
    }

    pub fn cancel(&self, id: ModalId) -> bool {
        let closed = self.remove(id);
        if let Some(entry) = closed {
            entry.modal.emit_canceled();
            entry.modal.emit_closed(ModalCloseReason::CancelAction);
            true
        } else {
            false
        }
    }

    pub fn complete(&self, id: ModalId) -> bool {
        let closed = self.remove(id);
        if let Some(entry) = closed {
            entry.modal.emit_completed();
            entry.modal.emit_closed(ModalCloseReason::CompleteAction);
            true
        } else {
            false
        }
    }

    pub fn close_top(&self) -> Option<ModalId> {
        let closed = self
            .state
            .write()
            .expect("modal state poisoned")
            .stack
            .pop();

        closed.map(|entry| {
            entry.modal.emit_closed(ModalCloseReason::Programmatic);
            entry.id
        })
    }

    pub fn close_all(&self) {
        let closed = {
            let mut state = self.state.write().expect("modal state poisoned");
            state.stack.drain(..).collect::<Vec<_>>()
        };

        for entry in closed {
            entry.modal.emit_closed(ModalCloseReason::Programmatic);
        }
    }

    pub fn list(&self) -> Vec<ManagedModal> {
        self.state
            .read()
            .expect("modal state poisoned")
            .stack
            .clone()
    }

    pub fn top(&self) -> Option<ManagedModal> {
        self.state
            .read()
            .expect("modal state poisoned")
            .stack
            .last()
            .cloned()
    }

    fn remove(&self, id: ModalId) -> Option<ManagedModal> {
        let mut state = self.state.write().expect("modal state poisoned");
        let index = state.stack.iter().position(|entry| entry.id == id)?;
        Some(state.stack.remove(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn modal_manager_uses_stack_order() {
        let manager = ModalManager::new();
        let first_id = manager.open(Modal::new("A"));
        let second_id = manager.open(Modal::new("B"));

        let stack = manager.list();
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].id(), first_id);
        assert_eq!(stack[1].id(), second_id);
        assert_eq!(manager.top().expect("top should exist").id(), second_id);

        let closed = manager.close_top().expect("top modal should close");
        assert_eq!(closed, second_id);
    }

    #[test]
    fn modal_manager_fires_confirm_and_close_callbacks() {
        let manager = ModalManager::new();
        let confirmed = Arc::new(AtomicUsize::new(0));
        let closed = Arc::new(AtomicUsize::new(0));

        let confirmed_for_modal = confirmed.clone();
        let closed_for_modal = closed.clone();
        let id = manager.open(
            Modal::confirm("Delete", "Are you sure?")
                .on_confirm(move || {
                    confirmed_for_modal.fetch_add(1, Ordering::SeqCst);
                })
                .on_close(move |reason| {
                    if reason == ModalCloseReason::ConfirmAction {
                        closed_for_modal.fetch_add(1, Ordering::SeqCst);
                    }
                }),
        );

        assert!(manager.confirm(id));
        assert_eq!(confirmed.load(Ordering::SeqCst), 1);
        assert_eq!(closed.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn modal_manager_fires_complete_and_close_callbacks() {
        let manager = ModalManager::new();
        let completed = Arc::new(AtomicUsize::new(0));
        let closed = Arc::new(AtomicUsize::new(0));

        let completed_for_modal = completed.clone();
        let closed_for_modal = closed.clone();
        let id = manager.open(
            Modal::success("Done", "Operation finished")
                .on_complete(move || {
                    completed_for_modal.fetch_add(1, Ordering::SeqCst);
                })
                .on_close(move |reason| {
                    if reason == ModalCloseReason::CompleteAction {
                        closed_for_modal.fetch_add(1, Ordering::SeqCst);
                    }
                }),
        );

        assert!(manager.complete(id));
        assert_eq!(completed.load(Ordering::SeqCst), 1);
        assert_eq!(closed.load(Ordering::SeqCst), 1);
    }
}
