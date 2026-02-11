use std::collections::{BTreeMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::icon::IconSource;
use crate::motion::MotionConfig;
use gpui::SharedString;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ToastId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
    Loading,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ToastPosition {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToastEntry {
    pub id: Option<ToastId>,
    pub title: SharedString,
    pub message: SharedString,
    pub icon: Option<IconSource>,
    pub kind: ToastKind,
    pub position: ToastPosition,
    pub auto_close_ms: Option<u32>,
    pub closable: bool,
    pub motion: MotionConfig,
}

impl ToastEntry {
    pub fn new(title: impl Into<SharedString>, message: impl Into<SharedString>) -> Self {
        Self {
            id: None,
            title: title.into(),
            message: message.into(),
            icon: None,
            kind: ToastKind::Info,
            position: ToastPosition::TopRight,
            auto_close_ms: Some(4_000),
            closable: true,
            motion: MotionConfig::default(),
        }
    }

    pub fn kind(mut self, value: ToastKind) -> Self {
        self.kind = value;
        self
    }

    pub fn icon(mut self, value: impl Into<SharedString>) -> Self {
        self.icon = Some(IconSource::named(value.into().to_string()));
        self
    }

    pub fn icon_source(mut self, source: IconSource) -> Self {
        self.icon = Some(source);
        self
    }

    pub fn position(mut self, value: ToastPosition) -> Self {
        self.position = value;
        self
    }

    pub fn auto_close_ms(mut self, value: Option<u32>) -> Self {
        self.auto_close_ms = value;
        self
    }

    pub fn closable(mut self, value: bool) -> Self {
        self.closable = value;
        self
    }

    pub fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToastViewport {
    pub position: ToastPosition,
    pub max_visible: usize,
}

impl ToastViewport {
    pub fn new(position: ToastPosition) -> Self {
        Self {
            position,
            max_visible: 5,
        }
    }

    pub fn max_visible(mut self, value: usize) -> Self {
        self.max_visible = value.max(1);
        self
    }
}

#[derive(Default)]
struct ToastState {
    queues: BTreeMap<ToastPosition, VecDeque<ToastEntry>>,
    max_visible: BTreeMap<ToastPosition, usize>,
}

#[derive(Clone, Default)]
pub struct ToastManager {
    next_id: Arc<AtomicU64>,
    state: Arc<RwLock<ToastState>>,
}

impl ToastManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn configure_viewport(&self, viewport: ToastViewport) {
        self.state
            .write()
            .expect("toast state poisoned")
            .max_visible
            .insert(viewport.position, viewport.max_visible);
    }

    pub fn show(&self, mut entry: ToastEntry) -> ToastId {
        let id = ToastId(self.next_id.fetch_add(1, Ordering::SeqCst) + 1);
        entry.id = Some(id);

        let mut state = self.state.write().expect("toast state poisoned");
        let limit = *state.max_visible.get(&entry.position).unwrap_or(&5);
        let queue = state.queues.entry(entry.position).or_default();
        queue.push_back(entry);

        while queue.len() > limit {
            queue.pop_front();
        }
        id
    }

    pub fn update(&self, id: ToastId, mut entry: ToastEntry) -> bool {
        let mut state = self.state.write().expect("toast state poisoned");
        for queue in state.queues.values_mut() {
            if let Some(current) = queue.iter_mut().find(|candidate| candidate.id == Some(id)) {
                entry.id = Some(id);
                *current = entry;
                return true;
            }
        }
        false
    }

    pub fn dismiss(&self, id: ToastId) -> bool {
        let mut state = self.state.write().expect("toast state poisoned");
        for queue in state.queues.values_mut() {
            if let Some(index) = queue.iter().position(|entry| entry.id == Some(id)) {
                queue.remove(index);
                return true;
            }
        }
        false
    }

    pub fn dismiss_all(&self) {
        for queue in self
            .state
            .write()
            .expect("toast state poisoned")
            .queues
            .values_mut()
        {
            queue.clear();
        }
    }

    pub fn list(&self, position: ToastPosition) -> Vec<ToastEntry> {
        self.state
            .read()
            .expect("toast state poisoned")
            .queues
            .get(&position)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_manager_enforces_position_limit() {
        let manager = ToastManager::new();
        manager.configure_viewport(ToastViewport::new(ToastPosition::TopRight).max_visible(2));
        manager.show(ToastEntry::new("a", "1"));
        manager.show(ToastEntry::new("b", "2"));
        manager.show(ToastEntry::new("c", "3"));

        let top_right = manager.list(ToastPosition::TopRight);
        assert_eq!(top_right.len(), 2);
        assert_eq!(top_right[0].title.to_string(), "b");
        assert_eq!(top_right[1].title.to_string(), "c");
    }
}
