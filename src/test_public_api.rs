use gpui::{IntoElement, div};

fn into_any(element: impl IntoElement) -> gpui::AnyElement {
    element.into_any_element()
}

fn assert_render_once<T: gpui::RenderOnce>() {}

#[test]
fn widgets_facade_exports_render_components() {
    assert_render_once::<crate::widgets::form::TextInput>();
    assert_render_once::<crate::widgets::form::Textarea>();
    assert_render_once::<crate::widgets::form::Select>();
    assert_render_once::<crate::widgets::form::MultiSelect>();
    assert_render_once::<crate::widgets::form::Button>();
    assert_render_once::<crate::widgets::overlay::Modal>();
    assert_render_once::<crate::widgets::overlay::Popover>();
    assert_render_once::<crate::widgets::navigation::Tree>();
    assert_render_once::<crate::widgets::navigation::Tabs>();
    assert_render_once::<crate::widgets::layout::ScrollArea>();
    assert_render_once::<crate::widgets::data::Table>();
    assert_render_once::<crate::widgets::display::Text>();
    assert_render_once::<crate::widgets::feedback::ToastLayer>();
}

#[test]
fn prelude_smoke_builds_core_widgets() {
    use crate::prelude::*;

    let _ = into_any(Button::new().label("button"));
    let _ = into_any(TextInput::new().placeholder("input"));
    let _ = into_any(Textarea::new().placeholder("textarea"));
    let _ = into_any(Select::new().option(SelectOption::new("a").label("A")));
    let _ = into_any(
        MultiSelect::new()
            .option(SelectOption::new("a").label("A"))
            .option(SelectOption::new("b").label("B")),
    );
    let _ = into_any(Modal::new().title("modal"));
    let _ = into_any(Popover::new().trigger(div()).content(div()));
    let _ = into_any(Tree::new().node(TreeNode::new("root").label("Root")));
    let _ = into_any(Tabs::new().item(TabItem::new("tab").label("Tab")));
    let _ = into_any(Table::new().header("Name").row(TableRow::new()));
}

#[test]
fn foundation_facade_exports_core_types() {
    let _ = crate::foundation::style::Size::Md;
    let _ = crate::foundation::style::Radius::Sm;
    let _ = crate::foundation::style::Variant::Default;
    let _ = crate::foundation::style::FieldLayout::Vertical;
    let _ = crate::foundation::motion::MotionConfig::default();
    let _ = crate::foundation::theme::Theme::default();
}
