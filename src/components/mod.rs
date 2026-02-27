mod accordion;
mod action_icon;
mod alert;
mod app_shell;
mod badge;
mod breadcrumbs;
mod button;
mod checkbox;
mod chip;
mod control;
mod divider;
mod drawer;
mod field_variant;
mod hovercard;
mod icon;
mod indicator;
mod input;
mod interaction_adapter;
mod layers;
mod layout;
mod loader;
mod loading_overlay;
mod markdown;
mod menu;
mod menu_state;
mod modal;
mod number_input;
mod overlay;
mod pagination;
mod paper;
mod popover;
mod popup;
mod popup_state;
mod progress;
mod radio;
mod range_slider;
mod rating;
mod scroll_area;
mod segmented_control;
mod select;
mod select_state;
mod selection_state;
mod slider;
mod slider_axis;
mod stepper;
mod switch;
mod table;
mod table_state;
mod tabs;
mod text;
mod text_input_actions;
mod text_input_state;
mod textarea;
mod timeline;
mod title;
mod title_bar;
mod toggle;
mod tooltip;
mod transition;
mod tree;
mod tree_state;
mod utils;

#[cfg(test)]
#[path = "test_state_logic.rs"]
mod test_state_logic;

pub use accordion::{Accordion, AccordionItem, AccordionItemMeta};
pub use action_icon::ActionIcon;
pub use alert::{Alert, AlertKind};
pub use app_shell::{AppShell, PaneChrome, PanelMode, Sidebar, SidebarMode};
pub use badge::Badge;
pub use breadcrumbs::{BreadcrumbItem, Breadcrumbs};
pub use button::{Button, ButtonGroup, ButtonGroupItem};
pub use checkbox::{Checkbox, CheckboxGroup, CheckboxOption};
pub use chip::{Chip, ChipGroup, ChipOption, ChipSelectionMode};
pub use divider::{Divider, DividerLabelPosition};
pub use drawer::{Drawer, DrawerPlacement};
pub use hovercard::{HoverCard, HoverCardPlacement};
pub use icon::Icon;
pub use indicator::{Indicator, IndicatorPosition};
pub use input::{PasswordInput, PinInput, TextInput};
pub use layers::{ModalLayer, ToastLayer};
pub use layout::{Grid, SimpleGrid, Space, Stack};
pub use loader::{Loader, LoaderElement, LoaderVariant};
pub use loading_overlay::LoadingOverlay;
pub use markdown::{Markdown, MarkdownLinkClick};
pub use menu::{Menu, MenuItem};
pub use modal::Modal;
pub use number_input::NumberInput;
pub use overlay::{Overlay, OverlayCoverage, OverlayMaterialMode};
pub use pagination::Pagination;
pub use paper::Paper;
pub use popover::{Popover, PopoverPlacement};
pub use progress::{Progress, ProgressSection};
pub use radio::{Radio, RadioGroup, RadioOption};
pub use range_slider::RangeSlider;
pub use rating::Rating;
pub use scroll_area::{ScrollArea, ScrollDirection};
pub use segmented_control::{SegmentedControl, SegmentedControlItem};
pub use select::{MultiSelect, Select, SelectOption};
pub use slider::Slider;
pub use stepper::{Stepper, StepperContentPosition, StepperStep};
pub use switch::{Switch, SwitchLabelPosition};
pub use table::{
    Table, TableAlign, TableCell, TablePaginationPosition, TableRow, TableSort, TableSortDirection,
};
pub use tabs::{TabItem, Tabs};
pub use text::{Text, TextTone};
pub use textarea::Textarea;
pub use timeline::{Timeline, TimelineItem};
pub use title::Title;
pub use title_bar::TitleBar;
pub use tooltip::{Tooltip, TooltipPlacement};
pub use transition::{TransitionExt, TransitionStage};
pub use tree::{Tree, TreeNode, TreeTogglePosition};

crate::impl_with_id_for_field!(Accordion, id);
crate::impl_with_id_for_field!(ActionIcon, id);
crate::impl_with_id_for_field!(Alert, id);
crate::impl_with_id_for_field!(AppShell, id);
crate::impl_with_id_for_field!(Badge, id);
crate::impl_with_id_for_field!(Breadcrumbs, id);
crate::impl_with_id_for_field!(Button, id);
crate::impl_with_id_for_field!(ButtonGroup, id);
crate::impl_with_id_for_field!(Checkbox, id);
crate::impl_with_id_for_field!(CheckboxGroup, id);
crate::impl_with_id_for_field!(Chip, id);
crate::impl_with_id_for_field!(ChipGroup, id);
crate::impl_with_id_for_field!(Divider, id);
crate::impl_with_id_for_field!(Drawer, id);
crate::impl_with_id_for_field!(Grid, id);
crate::impl_with_id_for_field!(HoverCard, id);
crate::impl_with_id_for_field!(Icon, id);
crate::impl_with_id_for_field!(Indicator, id);
crate::impl_with_id_for_field!(LoadingOverlay, id);
crate::impl_with_id_for_field!(Loader, id);
crate::impl_with_id_for_field!(Markdown, id);
crate::impl_with_id_for_field!(Menu, id);
crate::impl_with_id_for_field!(Modal, id);
crate::impl_with_id_for_field!(ModalLayer, id);
crate::impl_with_id_for_field!(MultiSelect, id);
crate::impl_with_id_for_field!(NumberInput, id);
crate::impl_with_id_for_field!(Overlay, id);
crate::impl_with_id_for_field!(Pagination, id);
crate::impl_with_id_for_field!(Paper, id);
crate::impl_with_id_for_field!(PasswordInput, id);
crate::impl_with_id_for_field!(PinInput, id);
crate::impl_with_id_for_field!(Popover, id);
crate::impl_with_id_for_field!(Progress, id);
crate::impl_with_id_for_field!(Radio, id);
crate::impl_with_id_for_field!(RadioGroup, id);
crate::impl_with_id_for_field!(RangeSlider, id);
crate::impl_with_id_for_field!(Rating, id);
crate::impl_with_id_for_field!(ScrollArea, id);
crate::impl_with_id_for_field!(SegmentedControl, id);
crate::impl_with_id_for_field!(Select, id);
crate::impl_with_id_for_field!(Sidebar, id);
crate::impl_with_id_for_field!(SimpleGrid, id);
crate::impl_with_id_for_field!(Slider, id);
crate::impl_with_id_for_field!(Space, id);
crate::impl_with_id_for_field!(Stepper, id);
crate::impl_with_id_for_field!(Switch, id);
crate::impl_with_id_for_field!(Table, id);
crate::impl_with_id_for_field!(Tabs, id);
crate::impl_with_id_for_field!(Text, id);
crate::impl_with_id_for_field!(TextInput, id);
crate::impl_with_id_for_field!(Textarea, id);
crate::impl_with_id_for_field!(Timeline, id);
crate::impl_with_id_for_field!(Title, id);
crate::impl_with_id_for_field!(TitleBar, id);
crate::impl_with_id_for_field!(ToastLayer, id);
crate::impl_with_id_for_field!(Tooltip, id);
crate::impl_with_id_for_field!(Tree, id);

crate::impl_default_via_new!(
    Accordion,
    ActionIcon,
    Alert,
    Badge,
    Breadcrumbs,
    Button,
    ButtonGroup,
    Checkbox,
    CheckboxGroup,
    Chip,
    ChipGroup,
    Drawer,
    Grid,
    HoverCard,
    Indicator,
    Loader,
    LoadingOverlay,
    Menu,
    Modal,
    MultiSelect,
    NumberInput,
    Overlay,
    Pagination,
    Paper,
    PasswordInput,
    Popover,
    Progress,
    Radio,
    RadioGroup,
    RangeSlider,
    Rating,
    ScrollArea,
    SegmentedControl,
    Select,
    Sidebar,
    SimpleGrid,
    Slider,
    Space,
    Stepper,
    Switch,
    Table,
    Tabs,
    TextInput,
    Textarea,
    Timeline,
    TitleBar,
    Tooltip,
    Tree
);

crate::impl_component_theme_overridable!(Accordion, |this| &mut this.theme);
crate::impl_component_theme_overridable!(ActionIcon, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Alert, |this| &mut this.theme);
crate::impl_component_theme_overridable!(AppShell, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Badge, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Breadcrumbs, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Button, |this| &mut this.theme);
crate::impl_component_theme_overridable!(ButtonGroup, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Checkbox, |this| &mut this.theme);
crate::impl_component_theme_overridable!(CheckboxGroup, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Chip, |this| &mut this.theme);
crate::impl_component_theme_overridable!(ChipGroup, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Divider, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Drawer, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Grid, |this| &mut this.theme);
crate::impl_component_theme_overridable!(HoverCard, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Icon, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Indicator, |this| &mut this.theme);
crate::impl_component_theme_overridable!(LoadingOverlay, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Loader, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Markdown, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Menu, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Modal, |this| &mut this.theme);
crate::impl_component_theme_overridable!(ModalLayer, |this| &mut this.theme);
crate::impl_component_theme_overridable!(MultiSelect, |this| &mut this.theme);
crate::impl_component_theme_overridable!(NumberInput, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Overlay, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Pagination, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Paper, |this| &mut this.theme);
crate::impl_component_theme_overridable!(PasswordInput, |this| &mut this.inner.theme);
crate::impl_component_theme_overridable!(PinInput, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Popover, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Progress, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Radio, |this| &mut this.theme);
crate::impl_component_theme_overridable!(RadioGroup, |this| &mut this.theme);
crate::impl_component_theme_overridable!(RangeSlider, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Rating, |this| &mut this.theme);
crate::impl_component_theme_overridable!(ScrollArea, |this| &mut this.theme);
crate::impl_component_theme_overridable!(SegmentedControl, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Select, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Sidebar, |this| &mut this.theme);
crate::impl_component_theme_overridable!(SimpleGrid, |this| this.inner.local_theme_mut());
crate::impl_component_theme_overridable!(Slider, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Space, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Stepper, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Switch, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Table, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Tabs, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Text, |this| &mut this.theme);
crate::impl_component_theme_overridable!(TextInput, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Textarea, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Timeline, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Title, |this| &mut this.theme);
crate::impl_component_theme_overridable!(TitleBar, |this| &mut this.theme);
crate::impl_component_theme_overridable!(ToastLayer, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Tooltip, |this| &mut this.theme);
crate::impl_component_theme_overridable!(Tree, |this| &mut this.theme);
