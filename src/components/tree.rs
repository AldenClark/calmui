use std::{collections::BTreeSet, rc::Rc};

use gpui::{
    AnyElement, ClickEvent, Component, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::{MotionAware, VariantConfigurable, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::Stack;
use super::control;
use super::icon::Icon;
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type SelectHandler = Rc<dyn Fn(Option<SharedString>, &mut Window, &mut gpui::App)>;
type ExpandedChangeHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut gpui::App)>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeNode {
    pub value: SharedString,
    pub label: SharedString,
    pub children: Vec<TreeNode>,
    pub disabled: bool,
    pub default_expanded: bool,
}

impl TreeNode {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            children: Vec::new(),
            disabled: false,
            default_expanded: false,
        }
    }

    pub fn child(mut self, node: TreeNode) -> Self {
        self.children.push(node);
        self
    }

    pub fn children(mut self, nodes: impl IntoIterator<Item = TreeNode>) -> Self {
        self.children.extend(nodes);
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn default_expanded(mut self, value: bool) -> Self {
        self.default_expanded = value;
        self
    }
}

pub struct Tree {
    id: String,
    nodes: Vec<TreeNode>,
    value: Option<SharedString>,
    value_controlled: bool,
    default_value: Option<SharedString>,
    expanded_values: Vec<SharedString>,
    expanded_controlled: bool,
    default_expanded_values: Vec<SharedString>,
    show_lines: bool,
    toggle_position: TreeTogglePosition,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_select: Option<SelectHandler>,
    on_expanded_change: Option<ExpandedChangeHandler>,
}

impl Tree {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("tree"),
            nodes: Vec::new(),
            value: None,
            value_controlled: false,
            default_value: None,
            expanded_values: Vec::new(),
            expanded_controlled: false,
            default_expanded_values: Vec::new(),
            show_lines: true,
            toggle_position: TreeTogglePosition::Left,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Sm,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            on_select: None,
            on_expanded_change: None,
        }
    }

    pub fn node(mut self, node: TreeNode) -> Self {
        self.nodes.push(node);
        self
    }

    pub fn nodes(mut self, nodes: impl IntoIterator<Item = TreeNode>) -> Self {
        self.nodes.extend(nodes);
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self.value_controlled = true;
        self
    }

    pub fn clear_value(mut self) -> Self {
        self.value = None;
        self.value_controlled = true;
        self
    }

    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    pub fn expanded_values(mut self, values: impl IntoIterator<Item = SharedString>) -> Self {
        self.expanded_values = values.into_iter().collect();
        self.expanded_controlled = true;
        self
    }

    pub fn default_expanded_values(
        mut self,
        values: impl IntoIterator<Item = SharedString>,
    ) -> Self {
        self.default_expanded_values = values.into_iter().collect();
        self
    }

    pub fn show_lines(mut self, value: bool) -> Self {
        self.show_lines = value;
        self
    }

    pub fn toggle_position(mut self, value: TreeTogglePosition) -> Self {
        self.toggle_position = value;
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(Option<SharedString>, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    pub fn on_expanded_change(
        mut self,
        handler: impl Fn(Vec<SharedString>, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_expanded_change = Some(Rc::new(handler));
        self
    }

    fn resolved_value(&self) -> Option<SharedString> {
        control::optional_text_state(
            &self.id,
            "value",
            self.value_controlled
                .then_some(self.value.as_ref().map(|value| value.to_string())),
            self.default_value.as_ref().map(|value| value.to_string()),
        )
        .map(SharedString::from)
    }

    fn collect_default_expanded(nodes: &[TreeNode], output: &mut Vec<SharedString>) {
        for node in nodes {
            if node.default_expanded {
                output.push(node.value.clone());
            }
            if !node.children.is_empty() {
                Self::collect_default_expanded(&node.children, output);
            }
        }
    }

    fn resolved_expanded(&self) -> Vec<SharedString> {
        let controlled = if self.expanded_controlled {
            Some(
                self.expanded_values
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        };

        let default = if self.default_expanded_values.is_empty() {
            let mut values = Vec::new();
            Self::collect_default_expanded(&self.nodes, &mut values);
            values
        } else {
            self.default_expanded_values.clone()
        }
        .into_iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();

        control::list_state(&self.id, "expanded", controlled, default)
            .into_iter()
            .map(SharedString::from)
            .collect()
    }

    fn indent_px(&self) -> f32 {
        match self.size {
            Size::Xs => 14.0,
            Size::Sm => 16.0,
            Size::Md => 18.0,
            Size::Lg => 20.0,
            Size::Xl => 22.0,
        }
    }

    fn selected_bg(&self) -> gpui::Hsla {
        let base = resolve_hsla(&self.theme, &self.theme.components.tree.row_selected_bg);
        match self.variant {
            Variant::Filled | Variant::Default => base,
            Variant::Light => base.alpha(0.82),
            Variant::Subtle => base.alpha(0.72),
            Variant::Outline => base.alpha(0.9),
            Variant::Ghost => base.alpha(0.64),
        }
    }
}

impl WithId for Tree {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantConfigurable for Tree {
    fn variant(mut self, value: Variant) -> Self {
        self.variant = value;
        self
    }

    fn size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    fn radius(mut self, value: Radius) -> Self {
        self.radius = value;
        self
    }
}

impl MotionAware for Tree {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeTogglePosition {
    Left,
    Right,
}

#[derive(Clone)]
struct TreeRenderCtx {
    tree_id: String,
    theme: crate::theme::LocalTheme,
    tokens: crate::theme::TreeTokens,
    selected: Option<SharedString>,
    expanded: BTreeSet<String>,
    expanded_values: Vec<String>,
    selected_controlled: bool,
    expanded_controlled: bool,
    show_lines: bool,
    toggle_position: TreeTogglePosition,
    indent_px: f32,
    radius: Radius,
    selected_bg: gpui::Hsla,
    on_select: Option<SelectHandler>,
    on_expanded_change: Option<ExpandedChangeHandler>,
}

impl TreeRenderCtx {
    fn toggled_values(mut current: Vec<String>, value: &str) -> Vec<String> {
        if let Some(index) = current.iter().position(|item| item == value) {
            current.remove(index);
        } else {
            current.push(value.to_string());
        }
        current
    }

    fn render_node(
        &self,
        window: &gpui::Window,
        node: TreeNode,
        depth: usize,
        path: String,
    ) -> AnyElement {
        let value_key = node.value.to_string();
        let has_children = !node.children.is_empty();
        let is_expanded = self.expanded.contains(value_key.as_str());
        let is_selected = self
            .selected
            .as_ref()
            .is_some_and(|selected| selected.as_ref() == node.value.as_ref());

        let mut row = div()
            .id(format!("{}-row-{path}", self.tree_id))
            .w_full()
            .flex()
            .items_center()
            .gap_1()
            .pl(px(depth as f32 * self.indent_px))
            .pr(px(6.0))
            .py(px(4.0))
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(if is_selected {
                resolve_hsla(&self.theme, &self.tokens.row_selected_fg)
            } else {
                resolve_hsla(&self.theme, &gpui::transparent_black())
            })
            .text_color(if node.disabled {
                resolve_hsla(&self.theme, &self.tokens.row_disabled_fg)
            } else if is_selected {
                resolve_hsla(&self.theme, &self.tokens.row_selected_fg)
            } else {
                resolve_hsla(&self.theme, &self.tokens.row_fg)
            })
            .bg(if is_selected {
                self.selected_bg
            } else {
                resolve_hsla(&self.theme, &gpui::transparent_black())
            });
        row = apply_radius(&self.theme, row, self.radius);

        let mut toggle = div()
            .id(format!("{}-toggle-{path}", self.tree_id))
            .w(px(16.0))
            .h(px(16.0))
            .flex()
            .items_center()
            .justify_center();
        if has_children {
            toggle = toggle.child(
                Icon::named(if is_expanded {
                    "chevron-down"
                } else {
                    "chevron-right"
                })
                .with_id(format!("{}-chevron-{path}", self.tree_id))
                .size(13.0),
            );
            if !node.disabled {
                let tree_id = self.tree_id.clone();
                let value = node.value.clone();
                let controlled = self.expanded_controlled;
                let on_expanded_change = self.on_expanded_change.clone();
                let expanded_snapshot = self.expanded_values.clone();
                toggle = toggle
                    .cursor_pointer()
                    .on_click(move |_: &ClickEvent, window, cx| {
                        let current = if controlled {
                            expanded_snapshot.clone()
                        } else {
                            control::list_state(
                                &tree_id,
                                "expanded",
                                None,
                                expanded_snapshot.clone(),
                            )
                        };
                        let next = Self::toggled_values(current, value.as_ref());
                        if !controlled {
                            control::set_list_state(&tree_id, "expanded", next.clone());
                            window.refresh();
                        }
                        if let Some(handler) = on_expanded_change.as_ref() {
                            (handler)(
                                next.into_iter().map(SharedString::from).collect(),
                                window,
                                cx,
                            );
                        }
                    });
            }
        }

        let connector = if self.show_lines && depth > 0 {
            Some(
                div()
                    .id(format!("{}-line-h-{path}", self.tree_id))
                    .w(px(8.0))
                    .h(super::utils::hairline_px(window))
                    .bg(resolve_hsla(&self.theme, &self.tokens.line))
                    .into_any_element(),
            )
        } else {
            None
        };

        let label = div()
            .id(format!("{}-label-{path}", self.tree_id))
            .flex_1()
            .min_w_0()
            .truncate()
            .child(node.label.clone());

        if let Some(connector) = connector {
            row = row.child(connector);
        }
        row = match self.toggle_position {
            TreeTogglePosition::Left => row.child(toggle).child(label),
            TreeTogglePosition::Right => row.child(label).child(toggle),
        };

        if !node.disabled {
            let hover_bg = resolve_hsla(&self.theme, &self.tokens.row_hover_bg);
            row = row.hover(move |style| style.bg(hover_bg));
            let tree_id = self.tree_id.clone();
            let value = node.value.clone();
            let controlled = self.selected_controlled;
            let on_select = self.on_select.clone();
            row = row
                .cursor_pointer()
                .on_click(move |_: &ClickEvent, window, cx| {
                    if !controlled {
                        control::set_optional_text_state(
                            &tree_id,
                            "value",
                            Some(value.to_string()),
                        );
                        window.refresh();
                    }
                    if let Some(handler) = on_select.as_ref() {
                        (handler)(Some(value.clone()), window, cx);
                    }
                });
        } else {
            row = row.opacity(0.55).cursor_default();
        }

        let mut wrapper = Stack::vertical()
            .id(format!("{}-node-{path}", self.tree_id))
            .w_full()
            .gap_0()
            .child(row);

        if has_children && is_expanded {
            let mut child_list = Stack::vertical()
                .id(format!("{}-children-{path}", self.tree_id))
                .w_full()
                .gap_0();
            if self.show_lines {
                child_list = child_list
                    .relative()
                    .ml(px((depth as f32 * self.indent_px) + 8.0))
                    .pl(px(10.0))
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .top_0()
                            .h_full()
                            .w(super::utils::hairline_px(window))
                            .bg(resolve_hsla(&self.theme, &self.tokens.line)),
                    );
            }
            for (index, child) in node.children.into_iter().enumerate() {
                child_list = child_list.child(self.render_node(
                    window,
                    child,
                    depth + 1,
                    format!("{path}-{index}"),
                ));
            }
            wrapper = wrapper.child(child_list);
        }

        wrapper.into_any_element()
    }
}

impl RenderOnce for Tree {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let expanded_values = self
            .resolved_expanded()
            .into_iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>();
        let ctx = TreeRenderCtx {
            tree_id: self.id.clone(),
            theme: self.theme.clone(),
            tokens: self.theme.components.tree.clone(),
            selected: self.resolved_value(),
            expanded: expanded_values.iter().cloned().collect(),
            expanded_values,
            selected_controlled: self.value_controlled,
            expanded_controlled: self.expanded_controlled,
            show_lines: self.show_lines,
            toggle_position: self.toggle_position,
            indent_px: self.indent_px(),
            radius: self.radius,
            selected_bg: self.selected_bg(),
            on_select: self.on_select.clone(),
            on_expanded_change: self.on_expanded_change.clone(),
        };

        let mut root = Stack::vertical().id(self.id.clone()).w_full().gap_0p5();
        for (index, node) in self.nodes.into_iter().enumerate() {
            root = root.child(ctx.render_node(window, node, 0, index.to_string()));
        }

        gpui::Refineable::refine(gpui::Styled::style(&mut root), &self.style);
        root.with_enter_transition(format!("{}-enter", self.id), self.motion)
    }
}

impl IntoElement for Tree {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemeOverridable for Tree {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(TreeNode);

impl gpui::Styled for Tree {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
