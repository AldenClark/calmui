use std::rc::Rc;

use gpui::{
    InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};

use crate::contracts::MotionAware;
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::Size;

use super::Stack;
use super::interaction_adapter::{ActivateHandler, PressAdapter, bind_press_adapter};
use super::transition::TransitionExt;
use super::utils::{InteractionStyles, apply_interaction_styles, interaction_style, resolve_hsla};

type ItemClickHandler = Rc<dyn Fn(usize, SharedString, &mut Window, &mut gpui::App)>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreadcrumbItem {
    pub label: Option<SharedString>,
    pub disabled: bool,
}

impl BreadcrumbItem {
    pub fn new() -> Self {
        Self {
            label: None,
            disabled: false,
        }
    }

    pub fn labeled(label: impl Into<SharedString>) -> Self {
        Self::new().label(label)
    }

    pub fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
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

#[derive(IntoElement)]
pub struct Breadcrumbs {
    id: ComponentId,
    items: Vec<BreadcrumbItem>,
    separator: SharedString,
    max_items: Option<usize>,
    size: Size,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_item_click: Option<ItemClickHandler>,
}

impl Breadcrumbs {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            items: Vec::new(),
            separator: "/".into(),
            max_items: None,
            size: Size::Md,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
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

    pub fn with_size(mut self, value: Size) -> Self {
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

    fn apply_item_size<T: Styled>(
        &self,
        node: T,
        preset: crate::theme::BreadcrumbsSizePreset,
    ) -> T {
        node.text_size(preset.font_size)
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

impl Breadcrumbs {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
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
        let size_preset = tokens.sizes.for_size(self.size);
        let nodes = self.nodes();
        let total_nodes = nodes.len();

        let mut children = Vec::with_capacity(total_nodes.saturating_mul(2).max(1));
        for (position, node) in nodes.into_iter().enumerate() {
            match node {
                CrumbNode::Item(index, item) => {
                    let is_current = position == total_nodes.saturating_sub(1);
                    let mut crumb = div()
                        .id(self.id.slot_index("item", index.to_string()))
                        .text_color(if is_current {
                            resolve_hsla(&self.theme, &tokens.item_current_fg)
                        } else {
                            resolve_hsla(&self.theme, &tokens.item_fg)
                        });
                    if let Some(label) = item.label.clone() {
                        crumb = crumb.child(label);
                    }
                    crumb = self.apply_item_size(crumb, size_preset);

                    if !is_current && !item.disabled {
                        if let Some(handler) = self.on_item_click.clone() {
                            let label = item.label.clone().unwrap_or_default();
                            let hover_bg = resolve_hsla(&self.theme, &tokens.item_hover_bg);
                            let press_bg = hover_bg.blend(gpui::black().opacity(0.08));
                            let activate_handler: ActivateHandler = Rc::new(move |window, cx| {
                                (handler)(index, label.clone(), window, cx);
                            });
                            crumb = crumb
                                .px(size_preset.item_padding_x)
                                .py(size_preset.item_padding_y)
                                .rounded(size_preset.item_radius)
                                .cursor_pointer();
                            crumb = apply_interaction_styles(
                                crumb,
                                InteractionStyles::new()
                                    .hover(interaction_style(move |style| style.bg(hover_bg)))
                                    .active(interaction_style(move |style| style.bg(press_bg)))
                                    .focus(interaction_style(move |style| style.bg(hover_bg))),
                            );
                            crumb = bind_press_adapter(
                                crumb,
                                PressAdapter::new(self.id.slot_index("item", index.to_string()))
                                    .on_activate(Some(activate_handler)),
                            );
                        }
                    } else if item.disabled {
                        crumb = crumb.opacity(0.5).cursor_default();
                    }

                    children.push(crumb);
                }
                CrumbNode::Ellipsis => {
                    let mut ellipsis = div()
                        .id(self.id.slot_index("ellipsis", position.to_string()))
                        .text_color(resolve_hsla(&self.theme, &tokens.separator))
                        .child("...");
                    ellipsis = self.apply_item_size(ellipsis, size_preset);
                    children.push(ellipsis);
                }
            }

            if position < total_nodes.saturating_sub(1) {
                let mut separator = div()
                    .id(self.id.slot_index("sep", position.to_string()))
                    .text_color(resolve_hsla(&self.theme, &tokens.separator))
                    .child(self.separator.clone());
                separator = self.apply_item_size(separator, size_preset);
                children.push(separator);
            }
        }

        Stack::horizontal()
            .id(self.id.clone())
            .items_center()
            .gap(tokens.root_gap)
            .children(children)
            .with_enter_transition(self.id.slot("enter"), self.motion)
    }
}

impl crate::contracts::ComponentThemeOverridable for Breadcrumbs {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_sized_via_method!(Breadcrumbs);

crate::impl_disableable!(BreadcrumbItem);

impl gpui::Styled for Breadcrumbs {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
