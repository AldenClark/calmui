use std::rc::Rc;

use gpui::{
    ClickEvent, Component, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use crate::contracts::{MotionAware, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::Size;

use super::primitives::h_stack;
use super::transition::TransitionExt;
use super::utils::resolve_hsla;

type ItemClickHandler = Rc<dyn Fn(usize, SharedString, &mut Window, &mut gpui::App)>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreadcrumbItem {
    pub label: SharedString,
    pub disabled: bool,
}

impl BreadcrumbItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
}

enum CrumbNode {
    Item(usize, BreadcrumbItem),
    Ellipsis,
}

pub struct Breadcrumbs {
    id: String,
    items: Vec<BreadcrumbItem>,
    separator: SharedString,
    max_items: Option<usize>,
    size: Size,
    theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    on_item_click: Option<ItemClickHandler>,
}

impl Breadcrumbs {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("breadcrumbs"),
            items: Vec::new(),
            separator: "/".into(),
            max_items: None,
            size: Size::Md,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::default(),
            on_item_click: None,
        }
    }

    pub fn item(mut self, item: BreadcrumbItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = BreadcrumbItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn separator(mut self, value: impl Into<SharedString>) -> Self {
        self.separator = value.into();
        self
    }

    pub fn max_items(mut self, value: usize) -> Self {
        self.max_items = Some(value.max(2));
        self
    }

    pub fn size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    pub fn on_item_click(
        mut self,
        handler: impl Fn(usize, SharedString, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_item_click = Some(Rc::new(handler));
        self
    }

    fn apply_item_size<T: Styled>(&self, node: T) -> T {
        match self.size {
            Size::Xs => node.text_xs(),
            Size::Sm => node.text_sm(),
            Size::Md => node.text_base(),
            Size::Lg => node.text_lg(),
            Size::Xl => node.text_xl(),
        }
    }

    fn nodes(&self) -> Vec<CrumbNode> {
        let total = self.items.len();
        if total == 0 {
            return Vec::new();
        }

        let Some(max_items) = self.max_items else {
            return self
                .items
                .iter()
                .cloned()
                .enumerate()
                .map(|(index, item)| CrumbNode::Item(index, item))
                .collect();
        };

        if total <= max_items {
            return self
                .items
                .iter()
                .cloned()
                .enumerate()
                .map(|(index, item)| CrumbNode::Item(index, item))
                .collect();
        }

        let tail_count = max_items.saturating_sub(1);
        let tail_start = total.saturating_sub(tail_count);

        let mut nodes = Vec::with_capacity(max_items + 1);
        nodes.push(CrumbNode::Item(0, self.items[0].clone()));
        nodes.push(CrumbNode::Ellipsis);
        for index in tail_start..total {
            nodes.push(CrumbNode::Item(index, self.items[index].clone()));
        }
        nodes
    }
}

impl WithId for Breadcrumbs {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl MotionAware for Breadcrumbs {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Breadcrumbs {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = self.theme.components.breadcrumbs.clone();
        let nodes = self.nodes();
        let total_nodes = nodes.len();

        let mut children = Vec::with_capacity(total_nodes.saturating_mul(2).max(1));
        for (position, node) in nodes.into_iter().enumerate() {
            match node {
                CrumbNode::Item(index, item) => {
                    let is_current = position == total_nodes.saturating_sub(1);
                    let mut crumb = div()
                        .id(format!("{}-item-{index}", self.id))
                        .text_color(if is_current {
                            resolve_hsla(&self.theme, &tokens.item_current_fg)
                        } else {
                            resolve_hsla(&self.theme, &tokens.item_fg)
                        })
                        .child(item.label.clone());
                    crumb = self.apply_item_size(crumb);

                    if !is_current && !item.disabled {
                        if let Some(handler) = self.on_item_click.clone() {
                            let label = item.label.clone();
                            let hover_bg = resolve_hsla(&self.theme, &tokens.item_hover_bg);
                            crumb = crumb
                                .cursor_pointer()
                                .px_1()
                                .rounded_sm()
                                .hover(move |style| style.bg(hover_bg))
                                .on_click(move |_: &ClickEvent, window, cx| {
                                    (handler)(index, label.clone(), window, cx);
                                });
                        }
                    } else if item.disabled {
                        crumb = crumb.opacity(0.5).cursor_default();
                    }

                    children.push(crumb.into_any_element());
                }
                CrumbNode::Ellipsis => {
                    let mut ellipsis = div()
                        .id(format!("{}-ellipsis-{position}", self.id))
                        .text_color(resolve_hsla(&self.theme, &tokens.separator))
                        .child("...");
                    ellipsis = self.apply_item_size(ellipsis);
                    children.push(ellipsis.into_any_element());
                }
            }

            if position < total_nodes.saturating_sub(1) {
                let mut separator = div()
                    .id(format!("{}-sep-{position}", self.id))
                    .text_color(resolve_hsla(&self.theme, &tokens.separator))
                    .child(self.separator.clone());
                separator = self.apply_item_size(separator);
                children.push(separator.into_any_element());
            }
        }

        h_stack()
            .id(self.id.clone())
            .items_center()
            .gap_1()
            .children(children)
            .with_enter_transition(format!("{}-enter", self.id), self.motion)
    }
}

impl IntoElement for Breadcrumbs {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemePatchable for Breadcrumbs {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}
