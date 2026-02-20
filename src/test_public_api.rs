use crate::form::FormModel as _;
use gpui::SharedString;
use gpui::{IntoElement, div};
use rust_decimal::Decimal;

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
    let _ = crate::foundation::form::FormOptions::default();
    let _ = crate::foundation::form::ValidationMode::OnSubmit;
    let _ = crate::foundation::form::RevalidateMode::OnChange;
    let _ = crate::foundation::form::compat::CompatibilityStatus::Experimental;
}

#[derive(Clone, crate::form::FormModel)]
struct ApiSmokeForm {
    title: SharedString,
    enabled: bool,
    amount: Decimal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ApiSmokeError(&'static str);

impl crate::form::ValidationError for ApiSmokeError {
    fn message(&self) -> SharedString {
        self.0.into()
    }
}

fn validate_smoke_title(_model: &ApiSmokeForm, value: &SharedString) -> Result<(), ApiSmokeError> {
    if value.trim().is_empty() {
        Err(ApiSmokeError("required"))
    } else {
        Ok(())
    }
}

#[test]
fn form_public_api_smoke_compiles() {
    let controller = crate::form::FormController::<ApiSmokeForm, ApiSmokeError>::new(
        ApiSmokeForm {
            title: "draft".into(),
            enabled: false,
            amount: Decimal::from_i128_with_scale(500, 2),
        },
        crate::form::FormOptions::default(),
    );
    let fields = ApiSmokeForm::fields();

    controller
        .register_field_validator(fields.title(), validate_smoke_title)
        .expect("register field validator");
    controller
        .register_required_field(fields.title())
        .expect("register required");
    controller
        .register_field_description(fields.title(), "required")
        .expect("register description");
    controller
        .register_dependency(fields.title(), fields.amount())
        .expect("register dependency");
    controller
        .set(fields.title(), "".into())
        .expect("set value");
    controller.touch(fields.title()).expect("touch field");
    controller.validate_form().expect("validate form");
    let _ = controller
        .field_error_for_display(fields.title())
        .expect("display error");
    let _ = controller
        .bind_text_input(fields.title(), crate::widgets::TextInput::new())
        .expect("bind text input");
    let _ = controller
        .bind_checkbox(fields.enabled(), crate::widgets::Checkbox::new())
        .expect("bind checkbox");
    let _ = controller
        .bind_number_input(fields.amount(), crate::widgets::NumberInput::new())
        .expect("bind number input");

    let store = crate::form::InMemoryDraftStore::new();
    controller.save_draft(&store).expect("save draft");
    controller.reset_to_initial().expect("reset");
    let _ = controller.load_draft(&store).expect("load draft");
    controller.clear_draft(&store).expect("clear draft");
}
