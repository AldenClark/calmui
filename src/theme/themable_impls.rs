use crate::components::{
    Accordion, ActionIcon, Alert, AppShell, Badge, Breadcrumbs, Button, ButtonGroup, Checkbox,
    CheckboxGroup, Chip, ChipGroup, Divider, Drawer, Grid, HoverCard, Loader, LoadingOverlay,
    Markdown, Menu, Modal, ModalLayer, MultiSelect, NumberInput, Overlay, Pagination, Paper,
    PasswordInput, PinInput, Popover, Progress, Radio, RadioGroup, RangeSlider, Rating, ScrollArea,
    SegmentedControl, Select, Sidebar, SimpleGrid, Slider, Space, Stepper, Switch, Table, Tabs,
    Text, TextInput, Textarea, Timeline, Title, TitleBar, ToastLayer, Tooltip, Tree,
};

use super::{
    AccordionOverrides, ActionIconOverrides, AppShellOverrides, BadgeOverrides,
    BreadcrumbsOverrides, ButtonOverrides, CheckboxOverrides, ChipOverrides, DividerOverrides,
    DrawerOverrides, HoverCardOverrides, LayoutOverrides, LoaderOverrides, LoadingOverlayOverrides,
    MarkdownOverrides, MenuOverrides, ModalOverrides, NumberInputOverrides, OverlayOverrides,
    PaginationOverrides, PaperOverrides, PopoverOverrides, ProgressOverrides, RadioOverrides,
    RangeSliderOverrides, RatingOverrides, ScrollAreaOverrides, SegmentedControlOverrides,
    SelectOverrides, SidebarOverrides, SliderOverrides, StepperOverrides, SwitchOverrides,
    TableOverrides, TabsOverrides, TextOverrides, TextareaOverrides, TimelineOverrides,
    TitleBarOverrides, TreeOverrides,
};

crate::impl_themable!(Button, button, ButtonOverrides);
crate::impl_themable!(ButtonGroup, button, ButtonOverrides);
crate::impl_themable!(TextInput, input, super::InputOverrides);
crate::impl_themable!(PasswordInput, input, super::InputOverrides);
crate::impl_themable!(PinInput, input, super::InputOverrides);
crate::impl_themable!(Radio, radio, RadioOverrides);
crate::impl_themable!(RadioGroup, radio, RadioOverrides);
crate::impl_themable!(Checkbox, checkbox, CheckboxOverrides);
crate::impl_themable!(CheckboxGroup, checkbox, CheckboxOverrides);
crate::impl_themable!(Switch, switch, SwitchOverrides);
crate::impl_themable!(Chip, chip, ChipOverrides);
crate::impl_themable!(ChipGroup, chip, ChipOverrides);
crate::impl_themable!(Badge, badge, BadgeOverrides);
crate::impl_themable!(Accordion, accordion, AccordionOverrides);
crate::impl_themable!(Menu, menu, MenuOverrides);
crate::impl_themable!(Progress, progress, ProgressOverrides);
crate::impl_themable!(Slider, slider, SliderOverrides);
crate::impl_themable!(Overlay, overlay, OverlayOverrides);
crate::impl_themable!(Loader, loader, LoaderOverrides);
crate::impl_themable!(LoadingOverlay, loading_overlay, LoadingOverlayOverrides);
crate::impl_themable!(Popover, popover, PopoverOverrides);
crate::impl_themable!(Tooltip, tooltip, super::TooltipOverrides);
crate::impl_themable!(HoverCard, hover_card, HoverCardOverrides);
crate::impl_themable!(Select, select, SelectOverrides);
crate::impl_themable!(MultiSelect, select, SelectOverrides);
crate::impl_themable!(Modal, modal, ModalOverrides);
crate::impl_themable!(ModalLayer, modal, ModalOverrides);
crate::impl_themable!(ToastLayer, toast, super::ToastOverrides);
crate::impl_themable!(Alert, toast, super::ToastOverrides);
crate::impl_themable!(Divider, divider, DividerOverrides);
crate::impl_themable!(ScrollArea, scroll_area, ScrollAreaOverrides);
crate::impl_themable!(Drawer, drawer, DrawerOverrides);
crate::impl_themable!(AppShell, app_shell, AppShellOverrides);
crate::impl_themable!(Sidebar, sidebar, SidebarOverrides);
crate::impl_themable!(TitleBar, title_bar, TitleBarOverrides);
crate::impl_themable!(Markdown, markdown, MarkdownOverrides);
crate::impl_themable!(Text, text, TextOverrides);
crate::impl_themable!(Title, title, super::TitleOverrides);
crate::impl_themable!(Paper, paper, PaperOverrides);
crate::impl_themable!(ActionIcon, action_icon, ActionIconOverrides);
crate::impl_themable!(
    SegmentedControl,
    segmented_control,
    SegmentedControlOverrides
);
crate::impl_themable!(Textarea, textarea, TextareaOverrides);
crate::impl_themable!(NumberInput, number_input, NumberInputOverrides);
crate::impl_themable!(RangeSlider, range_slider, RangeSliderOverrides);
crate::impl_themable!(Rating, rating, RatingOverrides);
crate::impl_themable!(Tabs, tabs, TabsOverrides);
crate::impl_themable!(Pagination, pagination, PaginationOverrides);
crate::impl_themable!(Breadcrumbs, breadcrumbs, BreadcrumbsOverrides);
crate::impl_themable!(Table, table, TableOverrides);
crate::impl_themable!(Stepper, stepper, StepperOverrides);
crate::impl_themable!(Timeline, timeline, TimelineOverrides);
crate::impl_themable!(Tree, tree, TreeOverrides);
crate::impl_themable!(Grid, layout, LayoutOverrides);
crate::impl_themable!(SimpleGrid, layout, LayoutOverrides);
crate::impl_themable!(Space, layout, LayoutOverrides);
