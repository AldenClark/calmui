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

#[cfg(test)]
mod tests {
    use super::FieldVariantRuntime;
    use crate::style::Variant;

    #[test]
    fn control_bg_applies_expected_variant_alpha_shape() {
        let base = gpui::black().opacity(0.8);

        let default_bg = FieldVariantRuntime::control_bg(base, Variant::Default);
        assert!((default_bg.a - base.a).abs() < f32::EPSILON);

        let ghost_bg = FieldVariantRuntime::control_bg(base, Variant::Ghost);
        assert!((ghost_bg.a - 0.0).abs() < f32::EPSILON);

        let subtle_bg = FieldVariantRuntime::control_bg(base, Variant::Subtle);
        assert!(subtle_bg.a < base.a);
    }

    #[test]
    fn control_border_preserves_or_reduces_alpha_by_variant_state() {
        let base = gpui::black().opacity(0.9);

        let ghost_idle = FieldVariantRuntime::control_border(base, Variant::Ghost, false, false);
        assert!((ghost_idle.a - 0.0).abs() < f32::EPSILON);

        let ghost_active = FieldVariantRuntime::control_border(base, Variant::Ghost, true, false);
        assert!(ghost_active.a > 0.0);

        let default_border =
            FieldVariantRuntime::control_border(base, Variant::Default, false, false);
        assert!((default_border.a - base.a).abs() < f32::EPSILON);
    }
}
