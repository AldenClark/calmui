use std::{collections::BTreeSet, rc::Rc};

use gpui::{
    AnyElement, ClickEvent, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::{MotionAware, VariantConfigurable};
use crate::id::ComponentId;
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

#[derive(IntoElement)]
pub struct Tree {
    id: ComponentId,
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
            id: ComponentId::default(),
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

    fn collect_visible_nodes(
        nodes: &[TreeNode],
        expanded: &BTreeSet<String>,
        parent: Option<&str>,
        output: &mut Vec<TreeVisibleNode>,
    ) {
        for node in nodes {
            let value = node.value.to_string();
            let is_expanded = expanded.contains(value.as_str());
            let first_child = node.children.first().map(|child| child.value.to_string());
            output.push(TreeVisibleNode {
                value: value.clone(),
                parent: parent.map(ToOwned::to_owned),
                disabled: node.disabled,
                has_children: !node.children.is_empty(),
                expanded: is_expanded,
                first_child,
            });
            if is_expanded && !node.children.is_empty() {
                Self::collect_visible_nodes(&node.children, expanded, Some(value.as_str()), output);
            }
        }
    }
}

impl Tree {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for Tree {
    fn with_variant(mut self, value: Variant) -> Self {
        self.variant = value;
        self
    }

    fn with_size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    fn with_radius(mut self, value: Radius) -> Self {
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

#[derive(Clone, Debug)]
struct TreeVisibleNode {
    value: String,
    parent: Option<String>,
    disabled: bool,
    has_children: bool,
    expanded: bool,
    first_child: Option<String>,
}

#[derive(Clone)]
struct TreeRenderCtx {
    tree_id: ComponentId,
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
            .id(self.tree_id.slot_index("row", path.clone()))
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
            .id(self.tree_id.slot_index("toggle", path.clone()))
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
                .with_id(self.tree_id.slot_index("chevron", path.clone()))
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
                    .id(self.tree_id.slot_index("line-h", path.clone()))
                    .w(px(8.0))
                    .h(super::utils::hairline_px(window))
                    .bg(resolve_hsla(&self.theme, &self.tokens.line)),
            )
        } else {
            None
        };

        let label = div()
            .id(self.tree_id.slot_index("label", path.clone()))
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
            .id(self.tree_id.slot_index("node", path.clone()))
            .w_full()
            .gap_0()
            .child(row);

        if has_children && is_expanded {
            let mut child_list = Stack::vertical()
                .id(self.tree_id.slot_index("children", path.clone()))
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
        let selected = self.resolved_value();
        let expanded_values = self
            .resolved_expanded()
            .into_iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>();
        let expanded_set = expanded_values.iter().cloned().collect::<BTreeSet<_>>();
        let mut visible_nodes = Vec::new();
        Self::collect_visible_nodes(&self.nodes, &expanded_set, None, &mut visible_nodes);
        let ctx = TreeRenderCtx {
            tree_id: self.id.clone(),
            theme: self.theme.clone(),
            tokens: self.theme.components.tree.clone(),
            selected: selected.clone(),
            expanded: expanded_set,
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

        let tree_id = self.id.clone();
        let selected_snapshot = selected.as_ref().map(|value| value.to_string());
        let expanded_snapshot = ctx.expanded_values.clone();
        let visible_snapshot = visible_nodes.clone();
        let selected_controlled = self.value_controlled;
        let expanded_controlled = self.expanded_controlled;
        let on_select = self.on_select.clone();
        let on_expanded_change = self.on_expanded_change.clone();

        let mut root = Stack::vertical()
            .id(self.id.clone())
            .w_full()
            .gap_0p5()
            .focusable()
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if visible_snapshot.is_empty() {
                    return;
                }

                let current_selected = control::optional_text_state(
                    &tree_id,
                    "value",
                    selected_controlled.then_some(selected_snapshot.clone()),
                    selected_snapshot.clone(),
                );
                let enabled_values = visible_snapshot
                    .iter()
                    .filter(|node| !node.disabled)
                    .map(|node| node.value.as_str())
                    .collect::<Vec<_>>();
                if enabled_values.is_empty() {
                    return;
                }

                let current_index = current_selected.as_ref().and_then(|selected| {
                    enabled_values
                        .iter()
                        .position(|value| *value == selected.as_str())
                });
                let mut next_selected = None::<String>;
                let mut next_expanded = None::<Vec<String>>;

                match key {
                    "up" => {
                        if let Some(index) = current_index {
                            if index > 0 {
                                next_selected = Some(enabled_values[index - 1].to_string());
                            } else {
                                next_selected = Some(enabled_values[0].to_string());
                            }
                        } else {
                            next_selected = Some(enabled_values[0].to_string());
                        }
                    }
                    "down" => {
                        if let Some(index) = current_index {
                            let next_index =
                                (index + 1).min(enabled_values.len().saturating_sub(1));
                            next_selected = Some(enabled_values[next_index].to_string());
                        } else {
                            next_selected = Some(enabled_values[0].to_string());
                        }
                    }
                    "home" => {
                        next_selected = Some(enabled_values[0].to_string());
                    }
                    "end" => {
                        if let Some(last) = enabled_values.last() {
                            next_selected = Some((*last).to_string());
                        }
                    }
                    "right" => {
                        if let Some(selected_value) = current_selected.as_ref()
                            && let Some(node) = visible_snapshot
                                .iter()
                                .find(|node| node.value == *selected_value)
                        {
                            if node.has_children && !node.expanded {
                                let current = if expanded_controlled {
                                    expanded_snapshot.clone()
                                } else {
                                    control::list_state(
                                        &tree_id,
                                        "expanded",
                                        None,
                                        expanded_snapshot.clone(),
                                    )
                                };
                                let mut set = current.into_iter().collect::<BTreeSet<_>>();
                                set.insert(node.value.clone());
                                next_expanded = Some(set.into_iter().collect());
                            } else if node.has_children
                                && node.expanded
                                && let Some(first_child) = node.first_child.as_ref()
                            {
                                next_selected = Some(first_child.clone());
                            }
                        }
                    }
                    "left" => {
                        if let Some(selected_value) = current_selected.as_ref()
                            && let Some(node) = visible_snapshot
                                .iter()
                                .find(|node| node.value == *selected_value)
                        {
                            if node.has_children && node.expanded {
                                let current = if expanded_controlled {
                                    expanded_snapshot.clone()
                                } else {
                                    control::list_state(
                                        &tree_id,
                                        "expanded",
                                        None,
                                        expanded_snapshot.clone(),
                                    )
                                };
                                let mut set = current.into_iter().collect::<BTreeSet<_>>();
                                set.remove(node.value.as_str());
                                next_expanded = Some(set.into_iter().collect());
                            } else if let Some(parent) = node.parent.as_ref() {
                                next_selected = Some(parent.clone());
                            }
                        }
                    }
                    "enter" | "space" => {
                        if current_selected.is_none() {
                            next_selected = Some(enabled_values[0].to_string());
                        } else if key == "space"
                            && let Some(selected_value) = current_selected.as_ref()
                            && let Some(node) = visible_snapshot
                                .iter()
                                .find(|node| node.value == *selected_value)
                            && node.has_children
                        {
                            let current = if expanded_controlled {
                                expanded_snapshot.clone()
                            } else {
                                control::list_state(
                                    &tree_id,
                                    "expanded",
                                    None,
                                    expanded_snapshot.clone(),
                                )
                            };
                            let next = TreeRenderCtx::toggled_values(current, node.value.as_str());
                            next_expanded = Some(next);
                        }
                    }
                    _ => {}
                }

                if let Some(next) = next_selected {
                    if !selected_controlled {
                        control::set_optional_text_state(&tree_id, "value", Some(next.clone()));
                        window.refresh();
                    }
                    if let Some(handler) = on_select.as_ref() {
                        (handler)(Some(SharedString::from(next)), window, cx);
                    }
                }

                if let Some(next) = next_expanded {
                    if !expanded_controlled {
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
                }
            });

        for (index, node) in self.nodes.into_iter().enumerate() {
            root = root.child(ctx.render_node(window, node, 0, index.to_string()));
        }

        gpui::Refineable::refine(gpui::Styled::style(&mut root), &self.style);
        root.with_enter_transition(self.id.slot("enter"), self.motion)
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
