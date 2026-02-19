use super::*;
use crate::contracts::{
    Disableable, FieldLike, Openable, Radiused, Sized as SizedContract, Varianted, Visible,
};
use crate::feedback::{ToastEntry, ToastKind, ToastManager, ToastPosition, ToastViewport};
use crate::overlay::ModalManager;
use crate::style::{FieldLayout, Radius, Size, Variant};
use gpui::{AnyElement, IntoElement, SharedString, div};

fn into_any(element: impl IntoElement) -> AnyElement {
    element.into_any_element()
}

fn exercise_disableable<T, F>(mut make: F)
where
    T: Disableable + IntoElement,
    F: FnMut() -> T,
{
    let _ = into_any(make().disabled(false));
    let _ = into_any(make().disabled(true));
}

fn exercise_openable<T, F>(mut make: F)
where
    T: Openable + IntoElement,
    F: FnMut() -> T,
{
    let _ = into_any(make().opened(false));
    let _ = into_any(make().opened(true));
}

fn exercise_visible<T, F>(mut make: F)
where
    T: Visible + IntoElement,
    F: FnMut() -> T,
{
    let _ = into_any(make().visible(false));
    let _ = into_any(make().visible(true));
}

fn exercise_field_like<T, F>(mut make: F)
where
    T: FieldLike + IntoElement,
    F: FnMut() -> T,
{
    let _ = into_any(
        make()
            .label("Label")
            .description("Description")
            .error("Error")
            .required(true)
            .layout(FieldLayout::Horizontal),
    );
    let _ = into_any(make().layout(FieldLayout::Vertical));
}

fn exercise_variant_size_radius<T, F>(mut make: F)
where
    T: Varianted + SizedContract + Radiused + IntoElement,
    F: FnMut() -> T,
{
    let _ = into_any(
        make()
            .with_variant(Variant::Outline)
            .with_size(Size::Lg)
            .with_radius(Radius::Pill),
    );
    let _ = into_any(
        make()
            .with_variant(Variant::Ghost)
            .with_size(Size::Sm)
            .with_radius(Radius::Xs),
    );
}

#[test]
fn behavior_matrix_for_disableable_components() {
    let _ = AccordionItem::new("item-a").label("Item A").disabled(true);
    let _ = BreadcrumbItem::new().label("Crumb").disabled(true);
    exercise_disableable(|| ActionIcon::new().child(Icon::named("sparkles")));
    exercise_disableable(|| Button::new().label("button"));
    let _ = ButtonGroupItem::new("group-a").label("A").disabled(true);
    exercise_disableable(|| Checkbox::new().label("checkbox"));
    let _ = CheckboxOption::new("check-a").label("A").disabled(true);
    exercise_disableable(|| Chip::new().label("chip"));
    let _ = ChipOption::new("chip-a").label("A").disabled(true);
    exercise_disableable(|| HoverCard::new().trigger(div()).content(div()));
    exercise_disableable(|| Indicator::new().child(div()));
    exercise_disableable(|| {
        Menu::new()
            .item(MenuItem::new("a").label("A"))
            .trigger(div())
    });
    let _ = MenuItem::new("menu-a").label("A").disabled(true);
    exercise_disableable(|| {
        MultiSelect::new()
            .option(SelectOption::new("a").label("A"))
            .option(SelectOption::new("b").label("B"))
    });
    exercise_disableable(NumberInput::new);
    exercise_disableable(|| Pagination::new().total(100).value(1));
    exercise_disableable(|| PasswordInput::new().placeholder("password"));
    exercise_disableable(|| PinInput::new(6).value("123456"));
    exercise_disableable(|| Popover::new().trigger(div()).content(div()));
    exercise_disableable(|| Radio::new().label("radio"));
    let _ = RadioOption::new("radio-a").label("A").disabled(true);
    exercise_disableable(|| RangeSlider::new().values(10.0, 90.0));
    exercise_disableable(|| Rating::new().value(3.5));
    exercise_disableable(|| Select::new().option(SelectOption::new("a").label("A")));
    let _ = SelectOption::new("select-a").label("A").disabled(true);
    let _ = SegmentedControlItem::new("segment-a")
        .label("A")
        .disabled(true);
    exercise_disableable(|| Slider::new().value(25.0));
    let _ = StepperStep::new("step-a").labeled("A").disabled(true);
    exercise_disableable(|| Switch::new().label("switch"));
    let _ = TabItem::new("tab-a").label("A").disabled(true);
    exercise_disableable(|| TextInput::new().placeholder("input"));
    exercise_disableable(|| Textarea::new().placeholder("textarea"));
    exercise_disableable(|| Tooltip::new().label("tip").trigger(div()));
    let _ = TreeNode::new("node-a").label("Node A").disabled(true);
}

#[test]
fn behavior_matrix_for_openable_and_visible_components() {
    exercise_openable(|| Drawer::new().content(div()));
    exercise_openable(|| HoverCard::new().trigger(div()).content(div()));
    exercise_openable(|| {
        Menu::new()
            .item(MenuItem::new("a").label("A"))
            .trigger(div())
    });
    exercise_openable(|| Modal::new().title("Modal"));
    exercise_openable(|| MultiSelect::new().option(SelectOption::new("a").label("A")));
    exercise_openable(|| Popover::new().trigger(div()).content(div()));
    exercise_openable(|| Select::new().option(SelectOption::new("a").label("A")));
    exercise_openable(|| Tooltip::new().label("tip").trigger(div()));

    exercise_visible(LoadingOverlay::new);
    exercise_visible(|| Overlay::new().content(div()));
}

#[test]
fn behavior_matrix_for_field_components() {
    exercise_field_like(TextInput::new);
    exercise_field_like(PasswordInput::new);
    exercise_field_like(Textarea::new);
    exercise_field_like(|| Select::new().option(SelectOption::new("a").label("A")));
    exercise_field_like(|| MultiSelect::new().option(SelectOption::new("a").label("A")));
    exercise_field_like(NumberInput::new);
}

#[test]
fn behavior_matrix_for_variant_size_radius_components() {
    exercise_variant_size_radius(|| Accordion::new().item(AccordionItem::new("a").label("A")));
    exercise_variant_size_radius(ActionIcon::new);
    exercise_variant_size_radius(|| Badge::new().label("badge"));
    exercise_variant_size_radius(|| Button::new().label("button"));
    exercise_variant_size_radius(|| ButtonGroup::new().item(ButtonGroupItem::new("a").label("A")));
    exercise_variant_size_radius(|| Checkbox::new().label("check"));
    exercise_variant_size_radius(|| {
        CheckboxGroup::new().option(CheckboxOption::new("a").label("A"))
    });
    exercise_variant_size_radius(|| Chip::new().label("chip"));
    exercise_variant_size_radius(|| ChipGroup::new().option(ChipOption::new("a").label("A")));
    exercise_variant_size_radius(|| MultiSelect::new().option(SelectOption::new("a").label("A")));
    exercise_variant_size_radius(|| Pagination::new().total(20).value(1));
    exercise_variant_size_radius(|| Progress::new().value(60.0));
    exercise_variant_size_radius(|| Radio::new().label("radio"));
    exercise_variant_size_radius(|| RadioGroup::new().option(RadioOption::new("a").label("A")));
    exercise_variant_size_radius(|| RangeSlider::new().values(20.0, 80.0));
    exercise_variant_size_radius(|| Rating::new().value(4.0));
    exercise_variant_size_radius(|| {
        SegmentedControl::new().item(SegmentedControlItem::new("a").label("A"))
    });
    exercise_variant_size_radius(|| Select::new().option(SelectOption::new("a").label("A")));
    exercise_variant_size_radius(|| Slider::new().value(50.0));
    exercise_variant_size_radius(|| Stepper::new().step(StepperStep::new("a").labeled("A")));
    exercise_variant_size_radius(|| Switch::new().label("switch"));
    exercise_variant_size_radius(|| Tabs::new().item(TabItem::new("a").label("A")));
    exercise_variant_size_radius(|| Timeline::new().item(TimelineItem::new().title("Event")));
    exercise_variant_size_radius(|| Tree::new().node(TreeNode::new("root").label("Root")));
}

#[test]
fn behavior_render_scenarios_group_a() {
    let _ = into_any(Accordion::new().item(AccordionItem::new("a").label("A").content(div())));
    let _ = into_any(ActionIcon::new().child(Icon::named("sparkles")));
    let _ = into_any(Alert::new().title("alert"));
    let _ = into_any(
        AppShell::new(div())
            .title_bar(TitleBar::new().title("Shell"))
            .sidebar(div())
            .inspector(div())
            .bottom_panel(div())
            .sidebar_mode(PanelMode::Overlay)
            .inspector_mode(PanelMode::Overlay)
            .sidebar_overlay_default_opened(true)
            .inspector_overlay_default_opened(true)
            .inline_dividers(true),
    );
    let _ = into_any(Badge::new().label("9+"));
    let _ = into_any(Breadcrumbs::new().item(BreadcrumbItem::new().label("Crumb")));
    let _ = into_any(Button::new().label("button").loading(true));
    let _ = into_any(
        ButtonGroup::new()
            .item(ButtonGroupItem::new("a").label("A"))
            .item(ButtonGroupItem::new("b").label("B"))
            .default_value("a"),
    );
    let _ = into_any(Checkbox::new().label("check"));
    let _ = into_any(CheckboxGroup::new().option(CheckboxOption::new("a").label("A")));
    let _ = into_any(Chip::new().label("chip"));
    let _ = into_any(ChipGroup::new().option(ChipOption::new("a").label("A")));
    let _ = into_any(Divider::horizontal());
    let _ = into_any(
        Drawer::new()
            .content(div())
            .placement(DrawerPlacement::Left),
    );
    let _ = into_any(Grid::new().columns(3).child(div()).child(div()));
    let _ = into_any(
        HoverCard::new()
            .trigger(div())
            .content(div())
            .match_trigger_width(true),
    );
    let _ = into_any(Icon::named("info"));
    let _ = into_any(Indicator::new().processing(true).child(div()));
    let _ = into_any(Loader::new().variant(LoaderVariant::Bars).label("loading"));
    let _ = into_any(LoadingOverlay::new().content(div()).label("Busy"));
    let _ = into_any(Markdown::new("## Heading"));
    let _ = into_any(
        Menu::new()
            .item(MenuItem::new("a").label("A"))
            .trigger(div()),
    );
    let _ = into_any(Modal::new().title("modal").body("content"));
}

#[test]
fn behavior_render_scenarios_group_b() {
    let modal_manager = ModalManager::new();
    let _ = modal_manager.open(Modal::titled("Managed"));
    let _ = into_any(ModalLayer::new(modal_manager.clone()));

    let _ = into_any(
        MultiSelect::new()
            .option(SelectOption::new("a").label("A"))
            .option(SelectOption::new("b").label("B")),
    );
    let _ = into_any(
        NumberInput::new()
            .value(42.0)
            .min(0.0)
            .max(100.0)
            .step(0.5)
            .precision(1),
    );
    let _ = into_any(
        Overlay::new()
            .content(div())
            .opacity(0.85)
            .blur_strength(1.2),
    );
    let _ = into_any(Pagination::new().total(100).value(2));
    let _ = into_any(Paper::new().child(div()));
    let _ = into_any(Popover::new().trigger(div()).content(div()));
    let _ = into_any(
        Progress::new()
            .value(60.0)
            .section(ProgressSection::new(25.0))
            .section(ProgressSection::new(35.0)),
    );
    let _ = into_any(Radio::new().label("radio"));
    let _ = into_any(RadioGroup::new().option(RadioOption::new("a").label("A")));
    let _ = into_any(RangeSlider::new().values(15.0, 85.0));
    let _ = into_any(Rating::new().value(3.5));
    let _ = into_any(
        ScrollArea::new()
            .child(div())
            .direction(ScrollDirection::Both),
    );
    let _ = into_any(
        SegmentedControl::new()
            .item(SegmentedControlItem::new("one").label("One"))
            .item(SegmentedControlItem::new("two").label("Two")),
    );
    let _ = into_any(
        Select::new()
            .option(SelectOption::new("a").label("A"))
            .option(SelectOption::new("b").label("B")),
    );
    let _ = into_any(Sidebar::new().header(div()).content(div()).footer(div()));
    let _ = into_any(SimpleGrid::new().cols(2).child(div()).child(div()));
    let _ = into_any(Slider::new().value(30.0).step(5.0));
    let _ = into_any(Space::new().with_size(Size::Lg));
    let _ = into_any(
        Stepper::new()
            .step(StepperStep::new("1").labeled("Step 1"))
            .step(StepperStep::new("2").labeled("Step 2")),
    );
    let _ = into_any(Switch::new().label("switch"));
    let _ = into_any(
        Table::new()
            .header("Name")
            .row(TableRow::new().cell(TableCell::new("Alice"))),
    );
    let _ = into_any(Tabs::new().item(TabItem::new("tab").label("Tab").panel("Panel")));
    let _ = into_any(Text::new("text"));
}

#[test]
fn behavior_render_scenarios_group_c() {
    let _ = into_any(TextInput::new().placeholder("input"));
    let _ = into_any(PasswordInput::new().placeholder("password"));
    let _ = into_any(PinInput::new(6).value("123456"));
    let _ = into_any(Textarea::new().placeholder("textarea"));
    let _ = into_any(Timeline::new().item(TimelineItem::new().title("Event").body("Body")));
    let _ = into_any(Title::new("title"));
    let _ = into_any(
        TitleBar::new()
            .title("titlebar")
            .show_window_controls(false),
    );
}

#[test]
fn behavior_render_scenarios_group_d() {
    let toast_manager = ToastManager::new();
    toast_manager.configure_viewport(ToastViewport::new(ToastPosition::TopRight).max_visible(3));
    let _ = toast_manager.show(ToastEntry::new("Saved", "Done").kind(ToastKind::Success));
    let _ = into_any(ToastLayer::new(toast_manager.clone()));

    let _ = into_any(
        Tooltip::new()
            .label("tip")
            .trigger(div())
            .trigger_on_click(true),
    );
    let _ = into_any(
        Tree::new()
            .node(
                TreeNode::new("root")
                    .label("Root")
                    .child(TreeNode::new("leaf").label("Leaf")),
            )
            .value("root")
            .expanded_values(vec![SharedString::from("root")]),
    );
}
