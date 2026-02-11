use std::collections::BTreeMap;

pub const COLOR_STOPS: usize = 10;
pub type ColorScale = [&'static str; COLOR_STOPS];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PaletteKey {
    Dark,
    Gray,
    Red,
    Pink,
    Grape,
    Violet,
    Indigo,
    Blue,
    Cyan,
    Teal,
    Green,
    Lime,
    Yellow,
    Orange,
}

impl PaletteKey {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Gray => "gray",
            Self::Red => "red",
            Self::Pink => "pink",
            Self::Grape => "grape",
            Self::Violet => "violet",
            Self::Indigo => "indigo",
            Self::Blue => "blue",
            Self::Cyan => "cyan",
            Self::Teal => "teal",
            Self::Green => "green",
            Self::Lime => "lime",
            Self::Yellow => "yellow",
            Self::Orange => "orange",
        }
    }
}

pub const PALETTE_KEYS: [PaletteKey; 14] = [
    PaletteKey::Dark,
    PaletteKey::Gray,
    PaletteKey::Red,
    PaletteKey::Pink,
    PaletteKey::Grape,
    PaletteKey::Violet,
    PaletteKey::Indigo,
    PaletteKey::Blue,
    PaletteKey::Cyan,
    PaletteKey::Teal,
    PaletteKey::Green,
    PaletteKey::Lime,
    PaletteKey::Yellow,
    PaletteKey::Orange,
];

pub struct PaletteCatalog;

impl PaletteCatalog {
    pub const fn scale(key: PaletteKey) -> ColorScale {
        match key {
            PaletteKey::Dark => [
                "#C9C9C9", "#b8b8b8", "#828282", "#696969", "#424242", "#3b3b3b", "#2e2e2e",
                "#242424", "#1f1f1f", "#141414",
            ],
            PaletteKey::Gray => [
                "#f8f9fa", "#f1f3f5", "#e9ecef", "#dee2e6", "#ced4da", "#adb5bd", "#868e96",
                "#495057", "#343a40", "#212529",
            ],
            PaletteKey::Red => [
                "#fff5f5", "#ffe3e3", "#ffc9c9", "#ffa8a8", "#ff8787", "#ff6b6b", "#fa5252",
                "#f03e3e", "#e03131", "#c92a2a",
            ],
            PaletteKey::Pink => [
                "#fff0f6", "#ffdeeb", "#fcc2d7", "#faa2c1", "#f783ac", "#f06595", "#e64980",
                "#d6336c", "#c2255c", "#a61e4d",
            ],
            PaletteKey::Grape => [
                "#f8f0fc", "#f3d9fa", "#eebefa", "#e599f7", "#da77f2", "#cc5de8", "#be4bdb",
                "#ae3ec9", "#9c36b5", "#862e9c",
            ],
            PaletteKey::Violet => [
                "#f3f0ff", "#e5dbff", "#d0bfff", "#b197fc", "#9775fa", "#845ef7", "#7950f2",
                "#7048e8", "#6741d9", "#5f3dc4",
            ],
            PaletteKey::Indigo => [
                "#edf2ff", "#dbe4ff", "#bac8ff", "#91a7ff", "#748ffc", "#5c7cfa", "#4c6ef5",
                "#4263eb", "#3b5bdb", "#364fc7",
            ],
            PaletteKey::Blue => [
                "#e7f5ff", "#d0ebff", "#a5d8ff", "#74c0fc", "#4dabf7", "#339af0", "#228be6",
                "#1c7ed6", "#1971c2", "#1864ab",
            ],
            PaletteKey::Cyan => [
                "#e3fafc", "#c5f6fa", "#99e9f2", "#66d9e8", "#3bc9db", "#22b8cf", "#15aabf",
                "#1098ad", "#0c8599", "#0b7285",
            ],
            PaletteKey::Teal => [
                "#e6fcf5", "#c3fae8", "#96f2d7", "#63e6be", "#38d9a9", "#20c997", "#12b886",
                "#0ca678", "#099268", "#087f5b",
            ],
            PaletteKey::Green => [
                "#ebfbee", "#d3f9d8", "#b2f2bb", "#8ce99a", "#69db7c", "#51cf66", "#40c057",
                "#37b24d", "#2f9e44", "#2b8a3e",
            ],
            PaletteKey::Lime => [
                "#f4fce3", "#e9fac8", "#d8f5a2", "#c0eb75", "#a9e34b", "#94d82d", "#82c91e",
                "#74b816", "#66a80f", "#5c940d",
            ],
            PaletteKey::Yellow => [
                "#fff9db", "#fff3bf", "#ffec99", "#ffe066", "#ffd43b", "#fcc419", "#fab005",
                "#f59f00", "#f08c00", "#e67700",
            ],
            PaletteKey::Orange => [
                "#fff4e6", "#ffe8cc", "#ffd8a8", "#ffc078", "#ffa94d", "#ff922b", "#fd7e14",
                "#f76707", "#e8590c", "#d9480f",
            ],
        }
    }

    pub fn store() -> BTreeMap<PaletteKey, ColorScale> {
        let mut palette_store = BTreeMap::new();
        for key in PALETTE_KEYS {
            palette_store.insert(key, Self::scale(key));
        }
        palette_store
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedScale {
    pub xs: &'static str,
    pub sm: &'static str,
    pub md: &'static str,
    pub lg: &'static str,
    pub xl: &'static str,
}

impl NamedScale {
    pub const fn new(
        xs: &'static str,
        sm: &'static str,
        md: &'static str,
        lg: &'static str,
        xl: &'static str,
    ) -> Self {
        Self { xs, sm, md, lg, xl }
    }
}

pub const SPACING: NamedScale = NamedScale::new("0.625rem", "0.75rem", "1rem", "1.25rem", "2rem");
pub const RADIUS: NamedScale = NamedScale::new("0.125rem", "0.25rem", "0.5rem", "1rem", "2rem");
pub const FONT_SIZES: NamedScale =
    NamedScale::new("0.75rem", "0.875rem", "1rem", "1.125rem", "1.25rem");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DurationScale {
    pub fast_ms: u16,
    pub normal_ms: u16,
    pub slow_ms: u16,
}

pub const MOTION_DURATIONS: DurationScale = DurationScale {
    fast_ms: 150,
    normal_ms: 220,
    slow_ms: 320,
};
