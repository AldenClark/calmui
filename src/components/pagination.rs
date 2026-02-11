use std::{collections::BTreeSet, rc::Rc};

use gpui::{
    ClickEvent, Component, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div,
};

use crate::contracts::{MotionAware, ThemeScoped, VariantSupport, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};
use crate::theme::Theme;

use super::control;
use super::primitives::h_stack;
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PaginationNode {
    Page(usize),
    Ellipsis,
}

pub struct Pagination {
    id: String,
    total: usize,
    value: Option<usize>,
    value_controlled: bool,
    default_value: usize,
    siblings: usize,
    boundaries: usize,
    disabled: bool,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: Theme,
    motion: MotionConfig,
    on_change: Option<ChangeHandler>,
}

impl Pagination {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("pagination"),
            total: 1,
            value: None,
            value_controlled: false,
            default_value: 1,
            siblings: 1,
            boundaries: 1,
            disabled: false,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Sm,
            theme: Theme::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    pub fn total(mut self, value: usize) -> Self {
        self.total = value.max(1);
        self
    }

    pub fn value(mut self, value: usize) -> Self {
        self.value = Some(value.max(1));
        self.value_controlled = true;
        self
    }

    pub fn default_value(mut self, value: usize) -> Self {
        self.default_value = value.max(1);
        self
    }

    pub fn siblings(mut self, value: usize) -> Self {
        self.siblings = value.min(4);
        self
    }

    pub fn boundaries(mut self, value: usize) -> Self {
        self.boundaries = value.min(4);
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    fn resolved_page(&self) -> usize {
        let total = self.total.max(1);
        let controlled = self.value_controlled.then_some(
            self.value
                .unwrap_or(self.default_value)
                .clamp(1, total)
                .to_string(),
        );
        let default = self.default_value.clamp(1, total).to_string();
        control::text_state(&self.id, "page", controlled, default)
            .parse::<usize>()
            .ok()
            .unwrap_or(1)
            .clamp(1, total)
    }

    fn apply_item_size<T: Styled>(size: Size, node: T) -> T {
        match size {
            Size::Xs => node.text_xs().py_0p5().px_1p5(),
            Size::Sm => node.text_sm().py_1().px_2(),
            Size::Md => node.text_base().py_1().px_2p5(),
            Size::Lg => node.text_lg().py_1p5().px_3(),
            Size::Xl => node.text_xl().py_2().px_3p5(),
        }
    }

    fn active_bg(&self) -> gpui::Hsla {
        let base = resolve_hsla(
            &self.theme,
            &self.theme.components.pagination.item_active_bg,
        );
        match self.variant {
            Variant::Filled | Variant::Default => base,
            Variant::Light => base.alpha(0.82),
            Variant::Subtle => base.alpha(0.72),
            Variant::Outline => base.alpha(0.9),
            Variant::Ghost => base.alpha(0.64),
        }
    }

    fn nodes(&self, current: usize) -> Vec<PaginationNode> {
        let total = self.total.max(1);
        if total <= 7 {
            return (1..=total).map(PaginationNode::Page).collect();
        }

        let mut pages = BTreeSet::new();
        let boundaries = self.boundaries.max(1);
        let siblings = self.siblings;

        for page in 1..=boundaries.min(total) {
            pages.insert(page);
        }

        let start_tail = total.saturating_sub(boundaries).saturating_add(1);
        for page in start_tail..=total {
            pages.insert(page);
        }

        let start_middle = current.saturating_sub(siblings).max(1);
        let end_middle = (current + siblings).min(total);
        for page in start_middle..=end_middle {
            pages.insert(page);
        }

        let mut nodes = Vec::new();
        let mut previous: Option<usize> = None;
        for page in pages {
            if let Some(prev) = previous {
                if page > prev + 1 {
                    nodes.push(PaginationNode::Ellipsis);
                }
            }
            nodes.push(PaginationNode::Page(page));
            previous = Some(page);
        }
        nodes
    }
}

impl WithId for Pagination {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantSupport for Pagination {
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

impl MotionAware for Pagination {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl ThemeScoped for Pagination {
    fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl RenderOnce for Pagination {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let tokens = self.theme.components.pagination.clone();
        let theme = self.theme.clone();
        let current = self.resolved_page();
        let total = self.total.max(1);
        let nodes = self.nodes(current);
        let active_bg = self.active_bg();
        let on_change = self.on_change.clone();
        let controlled = self.value_controlled;
        let pagination_id = self.id.clone();

        let make_item = |id_suffix: String, label: String, target: usize, disabled: bool| {
            let mut item = div()
                .id(id_suffix)
                .border_1()
                .border_color(resolve_hsla(&theme, &tokens.item_border))
                .bg(resolve_hsla(&theme, &tokens.item_bg))
                .text_color(if disabled {
                    resolve_hsla(&theme, &tokens.item_disabled_fg)
                } else {
                    resolve_hsla(&theme, &tokens.item_fg)
                })
                .cursor_pointer()
                .child(label);

            item = Self::apply_item_size(self.size, item);
            item = apply_radius(item, self.radius);

            if disabled || self.disabled {
                item = item.cursor_default().opacity(0.6);
            } else {
                let id = pagination_id.clone();
                let on_change = on_change.clone();
                let hover_bg = resolve_hsla(&theme, &tokens.item_hover_bg);
                item = item.hover(move |style| style.bg(hover_bg)).on_click(
                    move |_: &ClickEvent, window, cx| {
                        if !controlled {
                            control::set_text_state(&id, "page", target.to_string());
                            window.refresh();
                        }
                        if let Some(handler) = on_change.as_ref() {
                            (handler)(target, window, cx);
                        }
                    },
                );
            }

            item.into_any_element()
        };

        let prev_disabled = current <= 1 || self.disabled;
        let next_disabled = current >= total || self.disabled;

        let mut children = vec![make_item(
            format!("{}-prev", self.id),
            "Prev".to_string(),
            current.saturating_sub(1).max(1),
            prev_disabled,
        )];

        for (index, node) in nodes.into_iter().enumerate() {
            match node {
                PaginationNode::Page(page) => {
                    let is_active = page == current;
                    let mut page_item = div()
                        .id(format!("{}-page-{index}", self.id))
                        .border_1()
                        .border_color(resolve_hsla(&theme, &tokens.item_border))
                        .bg(if is_active {
                            active_bg
                        } else {
                            resolve_hsla(&theme, &tokens.item_bg)
                        })
                        .text_color(if is_active {
                            resolve_hsla(&theme, &tokens.item_active_fg)
                        } else if self.disabled {
                            resolve_hsla(&theme, &tokens.item_disabled_fg)
                        } else {
                            resolve_hsla(&theme, &tokens.item_fg)
                        })
                        .cursor_pointer()
                        .child(page.to_string());

                    page_item = Self::apply_item_size(self.size, page_item);
                    page_item = apply_radius(page_item, self.radius);

                    if self.disabled {
                        page_item = page_item.cursor_default().opacity(0.6);
                    } else {
                        let id = self.id.clone();
                        let on_change = on_change.clone();
                        let hover_bg = resolve_hsla(&theme, &tokens.item_hover_bg);
                        page_item = page_item.hover(move |style| style.bg(hover_bg)).on_click(
                            move |_: &ClickEvent, window, cx| {
                                if !controlled {
                                    control::set_text_state(&id, "page", page.to_string());
                                    window.refresh();
                                }
                                if let Some(handler) = on_change.as_ref() {
                                    (handler)(page, window, cx);
                                }
                            },
                        );
                    }

                    children.push(page_item.into_any_element());
                }
                PaginationNode::Ellipsis => {
                    let mut dots = div()
                        .id(format!("{}-dots-{index}", self.id))
                        .text_color(resolve_hsla(&theme, &tokens.dots_fg))
                        .child("...");
                    dots = Self::apply_item_size(self.size, dots);
                    children.push(dots.into_any_element());
                }
            }
        }

        children.push(make_item(
            format!("{}-next", self.id),
            "Next".to_string(),
            (current + 1).min(total),
            next_disabled,
        ));

        h_stack()
            .id(self.id.clone())
            .items_center()
            .gap_1()
            .children(children)
            .with_enter_transition(format!("{}-enter", self.id), self.motion)
    }
}

impl IntoElement for Pagination {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}
