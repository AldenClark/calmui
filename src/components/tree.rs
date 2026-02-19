use std::{collections::BTreeSet, rc::Rc};

use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::{MotionAware, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::Stack;
use super::icon::Icon;
use super::interaction_adapter::{ActivateHandler, PressAdapter, bind_press_adapter};
use super::transition::TransitionExt;
use super::tree_state::{self, TreeVisibleNode};
use super::utils::{apply_radius, resolve_hsla};

type SelectHandler = Rc<dyn Fn(Option<SharedString>, &mut Window, &mut gpui::App)>;
type ExpandedChangeHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut gpui::App)>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeNode {
    pub value: SharedString,
    pub label: Option<SharedString>,
    pub children: Vec<TreeNode>,
    pub disabled: bool,
    pub default_expanded: bool,
}

impl TreeNode {
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: None,
            children: Vec::new(),
            disabled: false,
            default_expanded: false,
        }
    }

    pub fn labeled(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self::new(value).label(label)
    }

    pub fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
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

    fn collect_default_expanded(nodes: &[TreeNode], output: &mut Vec<SharedString>) {
        struct Frame<'a> {
            nodes: &'a [TreeNode],
            index: usize,
        }

        let mut stack = vec![Frame { nodes, index: 0 }];
        while let Some(frame) = stack.last_mut() {
            if frame.index >= frame.nodes.len() {
                stack.pop();
                continue;
            }

            let node = &frame.nodes[frame.index];
            frame.index += 1;
            if node.default_expanded {
                output.push(node.value.clone());
            }

            if !node.children.is_empty() {
                stack.push(Frame {
                    nodes: &node.children,
                    index: 0,
                });
            }
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
    ) -> Vec<TreeVisibleNode> {
        struct Frame<'a> {
            nodes: &'a [TreeNode],
            index: usize,
            depth: usize,
            parent: Option<String>,
            path_prefix: String,
        }

        let mut output = Vec::new();
        let mut stack = vec![Frame {
            nodes,
            index: 0,
            depth: 0,
            parent: None,
            path_prefix: String::new(),
        }];

        while !stack.is_empty() {
            let mut next_frame = None;
            {
                let frame = stack.last_mut().expect("stack is not empty");
                if frame.index >= frame.nodes.len() {
                    stack.pop();
                    continue;
                }

                let index = frame.index;
                let node = &frame.nodes[index];
                frame.index += 1;

                let path = if frame.path_prefix.is_empty() {
                    index.to_string()
                } else {
                    format!("{}-{index}", frame.path_prefix)
                };
                let value = node.value.to_string();
                let has_children = !node.children.is_empty();
                let is_expanded = expanded.contains(value.as_str());
                let first_child = node.children.first().map(|child| child.value.to_string());

                output.push(TreeVisibleNode {
                    value: value.clone(),
                    parent: frame.parent.clone(),
                    label: node.label.as_ref().map(ToString::to_string),
                    depth: frame.depth,
                    path: path.clone(),
                    disabled: node.disabled,
                    has_children,
                    first_child,
                });

                if has_children && is_expanded {
                    next_frame = Some(Frame {
                        nodes: &node.children,
                        index: 0,
                        depth: frame.depth + 1,
                        parent: Some(value),
                        path_prefix: path,
                    });
                }
            }

            if let Some(frame) = next_frame {
                stack.push(frame);
            }
        }

        output
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
    size_preset: crate::theme::TreeSizePreset,
    radius: Radius,
    selected_bg: gpui::Hsla,
    on_select: Option<SelectHandler>,
    on_expanded_change: Option<ExpandedChangeHandler>,
}

impl TreeRenderCtx {
    fn render_visible_row(&self, window: &gpui::Window, node: &TreeVisibleNode) -> AnyElement {
        let value_key = node.value.clone();
        let has_children = node.has_children;
        let is_expanded = self.expanded.contains(value_key.as_str());
        let is_selected = self
            .selected
            .as_ref()
            .is_some_and(|selected| selected.as_ref() == node.value.as_str());

        let mut row = div()
            .id(self.tree_id.slot_index("row", node.path.clone()))
            .w_full()
            .flex()
            .items_center()
            .gap(self.size_preset.row_inner_gap)
            .pl(px(node.depth as f32 * f32::from(self.size_preset.indent)))
            .pr(self.size_preset.row_padding_right)
            .py(self.size_preset.row_padding_y)
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
            .id(self.tree_id.slot_index("toggle", node.path.clone()))
            .w(self.size_preset.toggle_size)
            .h(self.size_preset.toggle_size)
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
                .with_id(self.tree_id.slot_index("chevron", node.path.clone()))
                .size(f32::from(self.size_preset.toggle_icon_size)),
            );
            if !node.disabled {
                let tree_id = self.tree_id.clone();
                let value = SharedString::from(value_key.clone());
                let controlled = self.expanded_controlled;
                let on_expanded_change = self.on_expanded_change.clone();
                let expanded_snapshot = self.expanded_values.clone();
                let activate_handler: ActivateHandler = Rc::new(move |window, cx| {
                    let current = tree_state::resolve_expanded(
                        &tree_id,
                        controlled,
                        expanded_snapshot.clone(),
                        expanded_snapshot.clone(),
                    );
                    let next = tree_state::toggled_values(current, value.as_ref());
                    let should_refresh =
                        tree_state::apply_expanded(&tree_id, controlled, next.clone());
                    if let Some(handler) = on_expanded_change.as_ref() {
                        (handler)(
                            next.into_iter().map(SharedString::from).collect(),
                            window,
                            cx,
                        );
                    }
                    if should_refresh {
                        window.refresh();
                    }
                });
                toggle = toggle.cursor_pointer();
                toggle = bind_press_adapter(
                    toggle,
                    PressAdapter::new(self.tree_id.slot_index("toggle", node.path.clone()))
                        .on_activate(Some(activate_handler)),
                );
            }
        }

        let connector = if self.show_lines && node.depth > 0 {
            Some(
                div()
                    .id(self.tree_id.slot_index("line-h", node.path.clone()))
                    .w(self.size_preset.connector_stub_width)
                    .h(super::utils::hairline_px(window))
                    .bg(resolve_hsla(&self.theme, &self.tokens.line)),
            )
        } else {
            None
        };

        let label = div()
            .id(self.tree_id.slot_index("label", node.path.clone()))
            .flex_1()
            .min_w_0()
            .text_size(self.size_preset.label_size)
            .truncate()
            .child(
                node.label
                    .clone()
                    .map(SharedString::from)
                    .unwrap_or_else(|| SharedString::from(value_key.clone())),
            );

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
            let value = SharedString::from(value_key.clone());
            let controlled = self.selected_controlled;
            let on_select = self.on_select.clone();
            let activate_handler: ActivateHandler = Rc::new(move |window, cx| {
                let should_refresh =
                    tree_state::apply_selected(&tree_id, controlled, Some(value.to_string()));
                if let Some(handler) = on_select.as_ref() {
                    (handler)(Some(value.clone()), window, cx);
                }
                if should_refresh {
                    window.refresh();
                }
            });
            row = row.cursor_pointer();
            row = bind_press_adapter(
                row,
                PressAdapter::new(self.tree_id.slot_index("row", node.path.clone()))
                    .on_activate(Some(activate_handler)),
            );
        } else {
            row = row.opacity(0.55).cursor_default();
        }
        row.into_any_element()
    }
}

impl RenderOnce for Tree {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let selected_controlled_value = self.value.as_ref().map(|value| value.to_string());
        let selected_default_value = self.default_value.as_ref().map(|value| value.to_string());
        let selected = tree_state::resolve_selected(
            &self.id,
            self.value_controlled,
            selected_controlled_value.clone(),
            selected_default_value.clone(),
        )
        .map(SharedString::from);
        let expanded_controlled_values = self
            .expanded_values
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>();
        let expanded_default_values = if self.default_expanded_values.is_empty() {
            let mut values = Vec::new();
            Self::collect_default_expanded(&self.nodes, &mut values);
            values
        } else {
            self.default_expanded_values.clone()
        }
        .into_iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
        let expanded_values = tree_state::resolve_expanded(
            &self.id,
            self.expanded_controlled,
            expanded_controlled_values,
            expanded_default_values,
        );
        let expanded_set = expanded_values.iter().cloned().collect::<BTreeSet<_>>();
        let tokens = self.theme.components.tree.clone();
        let tree_size_preset = tokens.sizes.for_size(self.size);
        let visible_nodes = Self::collect_visible_nodes(&self.nodes, &expanded_set);
        let ctx = TreeRenderCtx {
            tree_id: self.id.clone(),
            theme: self.theme.clone(),
            tokens: tokens.clone(),
            selected: selected.clone(),
            expanded: expanded_set,
            expanded_values,
            selected_controlled: self.value_controlled,
            expanded_controlled: self.expanded_controlled,
            show_lines: self.show_lines,
            toggle_position: self.toggle_position,
            size_preset: tree_size_preset,
            radius: self.radius,
            selected_bg: self.selected_bg(),
            on_select: self.on_select.clone(),
            on_expanded_change: self.on_expanded_change.clone(),
        };

        let tree_id = self.id.clone();
        let selected_state_snapshot = selected.as_ref().map(|value| value.to_string());
        let expanded_state_snapshot = ctx.expanded_values.clone();
        let visible_snapshot = visible_nodes.clone();
        let selected_controlled = self.value_controlled;
        let expanded_controlled = self.expanded_controlled;
        let on_select = self.on_select.clone();
        let on_expanded_change = self.on_expanded_change.clone();

        let mut root = Stack::vertical()
            .id(self.id.clone())
            .w_full()
            .gap(tokens.root_gap)
            .focusable()
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if visible_snapshot.is_empty() {
                    return;
                }

                let current_selected = tree_state::resolve_selected(
                    &tree_id,
                    selected_controlled,
                    selected_state_snapshot.clone(),
                    selected_state_snapshot.clone(),
                );
                let current_expanded = tree_state::resolve_expanded(
                    &tree_id,
                    expanded_controlled,
                    expanded_state_snapshot.clone(),
                    expanded_state_snapshot.clone(),
                );
                let next = tree_state::key_transition(
                    key,
                    current_selected.as_deref(),
                    &visible_snapshot,
                    &current_expanded,
                );

                let mut should_refresh = false;
                if let Some(next_selected) = next.next_selected {
                    should_refresh |= tree_state::apply_selected(
                        &tree_id,
                        selected_controlled,
                        Some(next_selected.clone()),
                    );
                    if let Some(handler) = on_select.as_ref() {
                        (handler)(Some(SharedString::from(next_selected)), window, cx);
                    }
                }

                if let Some(next_expanded) = next.next_expanded {
                    should_refresh |= tree_state::apply_expanded(
                        &tree_id,
                        expanded_controlled,
                        next_expanded.clone(),
                    );
                    if let Some(handler) = on_expanded_change.as_ref() {
                        (handler)(
                            next_expanded.into_iter().map(SharedString::from).collect(),
                            window,
                            cx,
                        );
                    }
                }

                if should_refresh {
                    window.refresh();
                }
            });

        for node in &visible_nodes {
            root = root.child(ctx.render_visible_row(window, node));
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
