use super::*;
use crate::contracts::{ComponentThemeOverridable, Themable};
use crate::feedback::ToastManager;
use crate::overlay::ModalManager;
use crate::theme::ComponentOverrides;
use gpui::div;

fn apply_component_theme<T: ComponentThemeOverridable>(component: T) -> T {
    component
        .theme(|overrides| overrides)
        .with_theme_overrides(ComponentOverrides::default())
        .clear_theme_overrides()
}

fn apply_themable<T: Themable>(component: T) -> T {
    component.themed(|overrides| overrides)
}

#[test]
fn theme_api_smoke_for_themable_components() {
    let _ = apply_themable(apply_component_theme(Button::new().label("button")));
    let _ = apply_themable(apply_component_theme(ButtonGroup::new()));
    let _ = apply_themable(apply_component_theme(TextInput::new()));
    let _ = apply_themable(apply_component_theme(PasswordInput::new()));
    let _ = apply_themable(apply_component_theme(PinInput::new(4)));
    let _ = apply_themable(apply_component_theme(Radio::new()));
    let _ = apply_themable(apply_component_theme(RadioGroup::new()));
    let _ = apply_themable(apply_component_theme(Checkbox::new()));
    let _ = apply_themable(apply_component_theme(CheckboxGroup::new()));
    let _ = apply_themable(apply_component_theme(Switch::new()));
    let _ = apply_themable(apply_component_theme(Chip::new()));
    let _ = apply_themable(apply_component_theme(ChipGroup::new()));
    let _ = apply_themable(apply_component_theme(Badge::new()));
    let _ = apply_themable(apply_component_theme(Accordion::new()));
    let _ = apply_themable(apply_component_theme(Menu::new()));
    let _ = apply_themable(apply_component_theme(Progress::new()));
    let _ = apply_themable(apply_component_theme(Slider::new()));
    let _ = apply_themable(apply_component_theme(Overlay::new()));
    let _ = apply_themable(apply_component_theme(Loader::new()));
    let _ = apply_themable(apply_component_theme(LoadingOverlay::new()));
    let _ = apply_themable(apply_component_theme(Popover::new()));
    let _ = apply_themable(apply_component_theme(Tooltip::new()));
    let _ = apply_themable(apply_component_theme(HoverCard::new()));
    let _ = apply_themable(apply_component_theme(Select::new()));
    let _ = apply_themable(apply_component_theme(MultiSelect::new()));
    let _ = apply_themable(apply_component_theme(Modal::new()));
    let _ = apply_themable(apply_component_theme(ModalLayer::new(ModalManager::new())));
    let _ = apply_themable(apply_component_theme(ToastLayer::new(ToastManager::new())));
    let _ = apply_themable(apply_component_theme(Alert::new()));
    let _ = apply_themable(apply_component_theme(Divider::horizontal()));
    let _ = apply_themable(apply_component_theme(ScrollArea::new()));
    let _ = apply_themable(apply_component_theme(Drawer::new()));
    let _ = apply_themable(apply_component_theme(AppShell::new(div())));
    let _ = apply_themable(apply_component_theme(Sidebar::new()));
    let _ = apply_themable(apply_component_theme(TitleBar::new()));
    let _ = apply_themable(apply_component_theme(Markdown::new("demo")));
    let _ = apply_themable(apply_component_theme(Text::new("demo")));
    let _ = apply_themable(apply_component_theme(Title::new("demo")));
    let _ = apply_themable(apply_component_theme(Paper::new()));
    let _ = apply_themable(apply_component_theme(ActionIcon::new()));
    let _ = apply_themable(apply_component_theme(SegmentedControl::new()));
    let _ = apply_themable(apply_component_theme(Textarea::new()));
    let _ = apply_themable(apply_component_theme(NumberInput::new()));
    let _ = apply_themable(apply_component_theme(RangeSlider::new()));
    let _ = apply_themable(apply_component_theme(Rating::new()));
    let _ = apply_themable(apply_component_theme(Tabs::new()));
    let _ = apply_themable(apply_component_theme(Pagination::new()));
    let _ = apply_themable(apply_component_theme(Breadcrumbs::new()));
    let _ = apply_themable(apply_component_theme(Table::new()));
    let _ = apply_themable(apply_component_theme(Stepper::new()));
    let _ = apply_themable(apply_component_theme(Timeline::new()));
    let _ = apply_themable(apply_component_theme(Tree::new()));
    let _ = apply_themable(apply_component_theme(Grid::new()));
    let _ = apply_themable(apply_component_theme(SimpleGrid::new()));
    let _ = apply_themable(apply_component_theme(Space::new()));
}
