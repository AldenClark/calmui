use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Variant {
    Filled,
    Light,
    Subtle,
    Outline,
    Ghost,
    Default,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Size {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Radius {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Pill,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldLayout {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroupOrientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentState {
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
    Loading,
    Invalid,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StyleMap {
    tokens: BTreeMap<String, String>,
}

impl StyleMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn token(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.tokens.insert(name.into(), value.into());
        self
    }

    pub fn extend(mut self, other: Self) -> Self {
        for (name, value) in other.tokens {
            self.tokens.insert(name, value);
        }
        self
    }

    pub fn read(&self, name: &str) -> Option<&str> {
        self.tokens.get(name).map(String::as_str)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.tokens.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SlotStyles {
    slots: BTreeMap<String, StyleMap>,
}

impl SlotStyles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn slot(mut self, name: impl Into<String>, styles: StyleMap) -> Self {
        self.slots.insert(name.into(), styles);
        self
    }

    pub fn read(&self, name: &str) -> Option<&StyleMap> {
        self.slots.get(name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StyleContext {
    pub variant: Variant,
    pub size: Size,
    pub radius: Radius,
    pub state: ComponentState,
    pub color_key: String,
}

impl Default for StyleContext {
    fn default() -> Self {
        Self {
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Sm,
            state: ComponentState::Normal,
            color_key: "primary".to_string(),
        }
    }
}

impl StyleContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn variant(mut self, value: Variant) -> Self {
        self.variant = value;
        self
    }

    pub fn size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    pub fn radius(mut self, value: Radius) -> Self {
        self.radius = value;
        self
    }

    pub fn state(mut self, value: ComponentState) -> Self {
        self.state = value;
        self
    }

    pub fn color_key(mut self, value: impl Into<String>) -> Self {
        self.color_key = value.into();
        self
    }
}
