use gpui::Hsla;

use crate::style::Variant;

pub struct FieldVariantRuntime;

impl FieldVariantRuntime {
    pub fn control_bg(base: Hsla, variant: Variant) -> Hsla {
        match variant {
            Variant::Filled | Variant::Default => base,
            Variant::Light => base.alpha(0.9),
            Variant::Subtle => base.alpha(0.74),
            Variant::Outline => base.alpha(0.22),
            Variant::Ghost => base.alpha(0.0),
        }
    }

    pub fn control_border(base: Hsla, variant: Variant, active: bool, invalid: bool) -> Hsla {
        match variant {
            Variant::Ghost if !active && !invalid => base.alpha(0.0),
            Variant::Ghost => base.alpha(0.88),
            Variant::Subtle => base.alpha(if active { 0.78 } else { 0.52 }),
            Variant::Light => base.alpha(if active { 0.92 } else { 0.7 }),
            _ => base,
        }
    }
}
