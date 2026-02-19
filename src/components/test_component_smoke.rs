use super::*;
use crate::feedback::ToastManager;
use crate::overlay::ModalManager;
use gpui::{AnyElement, IntoElement, div};

fn into_any(element: impl IntoElement) -> AnyElement {
    element.into_any_element()
}

#[test]
fn smoke_primitives_render_into_any_element() {
    let _ = into_any(ActionIcon::new().child(Icon::named("sparkles")));
    let _ = into_any(Badge::new().label("badge"));
    let _ = into_any(Breadcrumbs::new().item(BreadcrumbItem::new().label("crumb")));
    let _ = into_any(Button::new().label("button"));
    let _ = into_any(
        ButtonGroup::new()
            .item(ButtonGroupItem::new("a").label("A"))
            .item(ButtonGroupItem::new("b").label("B")),
    );
    let _ = into_any(Checkbox::new().label("check"));
    let _ = into_any(
        CheckboxGroup::new()
            .option(CheckboxOption::new("a").label("A"))
            .option(CheckboxOption::new("b").label("B")),
    );
    let _ = into_any(Chip::new().label("chip"));
    let _ = into_any(
        ChipGroup::new()
            .option(ChipOption::new("a").label("A"))
            .option(ChipOption::new("b").label("B")),
    );
    let _ = into_any(Divider::horizontal());
    let _ = into_any(Icon::named("info"));
    let _ = into_any(Indicator::new().child(div().into_any_element()));
    let _ = into_any(Loader::new().label("loading"));
    let _ = into_any(LoadingOverlay::new().content(div()));
    let _ = into_any(Markdown::new("# hello"));
    let _ = into_any(Paper::new().child(div().into_any_element()));
    let _ = into_any(Progress::new().value(40.0));
    let _ = into_any(
        Progress::new()
            .value(60.0)
            .section(ProgressSection::new(20.0))
            .section(ProgressSection::new(40.0)),
    );
    let _ = into_any(Rating::new().value(3.0));
    let _ = into_any(Text::new("text"));
    let _ = into_any(Title::new("title"));
}

#[test]
fn smoke_form_and_picker_components_render_into_any_element() {
    let _ = into_any(TextInput::new().placeholder("input"));
    let _ = into_any(PasswordInput::new().placeholder("password"));
    let _ = into_any(PinInput::new(6).value("123456"));
    let _ = into_any(Textarea::new().placeholder("textarea"));
    let _ = into_any(NumberInput::new().value(42.0));
    let _ = into_any(Select::new().option(SelectOption::new("a").label("A")));
    let _ = into_any(
        MultiSelect::new()
            .option(SelectOption::new("a").label("A"))
            .option(SelectOption::new("b").label("B")),
    );
    let _ = into_any(Slider::new().value(30.0));
    let _ = into_any(RangeSlider::new().values(10.0, 90.0));
    let _ = into_any(Switch::new().label("switch"));
    let _ = into_any(
        SegmentedControl::new()
            .item(SegmentedControlItem::new("one").label("One"))
            .item(SegmentedControlItem::new("two").label("Two")),
    );
    let _ = into_any(Tabs::new().item(TabItem::new("tab").label("Tab")));
    let _ = into_any(
        Stepper::new()
            .step(StepperStep::new("1").labeled("Step 1"))
            .step(StepperStep::new("2").labeled("Step 2")),
    );
}

#[test]
fn smoke_popup_overlay_and_navigation_components_render_into_any_element() {
    let _ = into_any(Alert::new().title("alert"));
    let _ = into_any(Drawer::new().content(div()));
    let _ = into_any(HoverCard::new().trigger(div()).content(div()));
    let _ = into_any(
        Menu::new()
            .item(MenuItem::new("v").label("Item"))
            .trigger(div()),
    );
    let _ = into_any(Modal::new().title("modal"));
    let _ = into_any(Overlay::new().content(div()));
    let _ = into_any(Pagination::new().total(100).value(2));
    let _ = into_any(Popover::new().trigger(div()).content(div()));
    let _ = into_any(ScrollArea::new().child(div().into_any_element()));
    let _ = into_any(Tooltip::new().label("tip").trigger(div()));
    let _ = into_any(TitleBar::new().title("titlebar"));
}

#[test]
fn smoke_layout_and_shell_components_render_into_any_element() {
    let _ = into_any(Grid::new().child(div().into_any_element()));
    let _ = into_any(SimpleGrid::new().child(div().into_any_element()));
    let _ = into_any(Space::new());
    let _ = into_any(Sidebar::new().content(div()));
    let _ = into_any(AppShell::new(div()));
    let _ = into_any(ToastLayer::new(ToastManager::new()));
    let _ = into_any(ModalLayer::new(ModalManager::new()));
}

#[test]
fn smoke_data_heavy_components_render_into_any_element() {
    let table = Table::new()
        .header("Name")
        .row(TableRow::new().cell(TableCell::new("Alice")));
    let _ = into_any(table);

    let tree = Tree::new().node(TreeNode::new("root").label("Root"));
    let _ = into_any(tree);

    let timeline = Timeline::new().item(TimelineItem::new().title("Event"));
    let _ = into_any(timeline);

    let accordion = Accordion::new().item(AccordionItem::new("a").label("A").content(div()));
    let _ = into_any(accordion);
}
