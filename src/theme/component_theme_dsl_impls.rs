use crate::components::{
    Accordion, ActionIcon, AppShell, Badge, Breadcrumbs, Button, ButtonGroup, Checkbox,
    CheckboxGroup, Chip, ChipGroup, Divider, Drawer, HoverCard, LoadingOverlay, Menu, Modal,
    ModalLayer, MultiSelect, NumberInput, Overlay, Pagination, Paper, PasswordInput, PinInput,
    Popover, Progress, Radio, RadioGroup, RangeSlider, Rating, ScrollArea, SegmentedControl,
    Select, Slider, Stepper, Switch, Table, Tabs, Text, TextInput, Textarea, Timeline, Title,
    ToastLayer, Tooltip, Tree,
};

use super::{
    AccordionOverrides, ActionIconOverrides, AppShellOverrides, BadgeOverrides,
    BreadcrumbsOverrides, ButtonOverrides, CheckboxOverrides, ChipOverrides, DividerOverrides,
    DrawerOverrides, HoverCardOverrides, LoadingOverlayOverrides, MenuOverrides, ModalOverrides,
    NumberInputOverrides, OverlayOverrides, PaginationOverrides, PaperOverrides, PopoverOverrides,
    ProgressOverrides, RadioOverrides, RangeSliderOverrides, RatingOverrides, ScrollAreaOverrides,
    SegmentedControlOverrides, SelectOverrides, SliderOverrides, StepperOverrides, SwitchOverrides,
    TableOverrides, TabsOverrides, TextOverrides, TextareaOverrides, TimelineOverrides,
    TreeOverrides,
};

crate::impl_component_theme_dsl!(Button, button, ButtonOverrides);
crate::impl_component_theme_dsl!(ButtonGroup, button, ButtonOverrides);
crate::impl_component_theme_dsl!(TextInput, input, super::InputOverrides);
crate::impl_component_theme_dsl!(PasswordInput, input, super::InputOverrides);
crate::impl_component_theme_dsl!(PinInput, input, super::InputOverrides);
crate::impl_component_theme_dsl!(Radio, radio, RadioOverrides);
crate::impl_component_theme_dsl!(RadioGroup, radio, RadioOverrides);
crate::impl_component_theme_dsl!(Checkbox, checkbox, CheckboxOverrides);
crate::impl_component_theme_dsl!(CheckboxGroup, checkbox, CheckboxOverrides);
crate::impl_component_theme_dsl!(Switch, switch, SwitchOverrides);
crate::impl_component_theme_dsl!(Chip, chip, ChipOverrides);
crate::impl_component_theme_dsl!(ChipGroup, chip, ChipOverrides);
crate::impl_component_theme_dsl!(Badge, badge, BadgeOverrides);
crate::impl_component_theme_dsl!(Accordion, accordion, AccordionOverrides);
crate::impl_component_theme_dsl!(Menu, menu, MenuOverrides);
crate::impl_component_theme_dsl!(Progress, progress, ProgressOverrides);
crate::impl_component_theme_dsl!(Slider, slider, SliderOverrides);
crate::impl_component_theme_dsl!(Overlay, overlay, OverlayOverrides);
crate::impl_component_theme_dsl!(LoadingOverlay, loading_overlay, LoadingOverlayOverrides);
crate::impl_component_theme_dsl!(Popover, popover, PopoverOverrides);
crate::impl_component_theme_dsl!(Tooltip, tooltip, super::TooltipOverrides);
crate::impl_component_theme_dsl!(HoverCard, hover_card, HoverCardOverrides);
crate::impl_component_theme_dsl!(Select, select, SelectOverrides);
crate::impl_component_theme_dsl!(MultiSelect, select, SelectOverrides);
crate::impl_component_theme_dsl!(Modal, modal, ModalOverrides);
crate::impl_component_theme_dsl!(ModalLayer, modal, ModalOverrides);
crate::impl_component_theme_dsl!(ToastLayer, toast, super::ToastOverrides);
crate::impl_component_theme_dsl!(Divider, divider, DividerOverrides);
crate::impl_component_theme_dsl!(ScrollArea, scroll_area, ScrollAreaOverrides);
crate::impl_component_theme_dsl!(Drawer, drawer, DrawerOverrides);
crate::impl_component_theme_dsl!(AppShell, app_shell, AppShellOverrides);
crate::impl_component_theme_dsl!(Text, text, TextOverrides);
crate::impl_component_theme_dsl!(Title, title, super::TitleOverrides);
crate::impl_component_theme_dsl!(Paper, paper, PaperOverrides);
crate::impl_component_theme_dsl!(ActionIcon, action_icon, ActionIconOverrides);
crate::impl_component_theme_dsl!(
    SegmentedControl,
    segmented_control,
    SegmentedControlOverrides
);
crate::impl_component_theme_dsl!(Textarea, textarea, TextareaOverrides);
crate::impl_component_theme_dsl!(NumberInput, number_input, NumberInputOverrides);
crate::impl_component_theme_dsl!(RangeSlider, range_slider, RangeSliderOverrides);
crate::impl_component_theme_dsl!(Rating, rating, RatingOverrides);
crate::impl_component_theme_dsl!(Tabs, tabs, TabsOverrides);
crate::impl_component_theme_dsl!(Pagination, pagination, PaginationOverrides);
crate::impl_component_theme_dsl!(Breadcrumbs, breadcrumbs, BreadcrumbsOverrides);
crate::impl_component_theme_dsl!(Table, table, TableOverrides);
crate::impl_component_theme_dsl!(Stepper, stepper, StepperOverrides);
crate::impl_component_theme_dsl!(Timeline, timeline, TimelineOverrides);
crate::impl_component_theme_dsl!(Tree, tree, TreeOverrides);
