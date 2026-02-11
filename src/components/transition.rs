use std::time::Duration;

use gpui::{Animation, AnimationElement, AnimationExt, ElementId, Styled, px};

use crate::motion::{Easing, MotionConfig, MotionLevel, MotionTransition, TransitionPreset};

use super::utils::offset_with_progress;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionStage {
    Enter,
    Exit,
}

pub trait TransitionExt: Sized + AnimationExt + Styled + 'static {
    fn with_transition(
        self,
        id: impl Into<ElementId>,
        motion: MotionConfig,
        stage: TransitionStage,
    ) -> AnimationElement<Self> {
        if motion.level == MotionLevel::None {
            return self
                .with_animation(id, Animation::new(Duration::from_millis(1)), |this, _| this);
        }

        let profile = match stage {
            TransitionStage::Enter => motion.enter,
            TransitionStage::Exit => motion.exit,
        };

        self.with_transition_profile(id, profile, stage)
    }

    fn with_enter_transition(
        self,
        id: impl Into<ElementId>,
        motion: MotionConfig,
    ) -> AnimationElement<Self> {
        self.with_transition(id, motion, TransitionStage::Enter)
    }

    fn with_exit_transition(
        self,
        id: impl Into<ElementId>,
        motion: MotionConfig,
    ) -> AnimationElement<Self> {
        self.with_transition(id, motion, TransitionStage::Exit)
    }

    fn with_repeating_transition(
        self,
        id: impl Into<ElementId>,
        profile: MotionTransition,
    ) -> AnimationElement<Self> {
        let easing = easing_fn(profile.easing);
        let animation = Animation::new(Duration::from_millis(profile.duration_ms as u64))
            .repeat()
            .with_easing(easing);

        if profile.delay_ms > 0 {
            let idle = Animation::new(Duration::from_millis(profile.delay_ms as u64));
            return self.with_animations(id, vec![idle, animation], move |this, ix, delta| {
                if ix == 0 {
                    apply_preset(this, profile, 0.0)
                } else {
                    apply_preset(this, profile, delta)
                }
            });
        }

        self.with_animation(id, animation, move |this, delta| {
            apply_preset(this, profile, delta)
        })
    }

    fn with_transition_profile(
        self,
        id: impl Into<ElementId>,
        profile: MotionTransition,
        stage: TransitionStage,
    ) -> AnimationElement<Self> {
        let easing = easing_fn(profile.easing);
        let animation =
            Animation::new(Duration::from_millis(profile.duration_ms as u64)).with_easing(easing);

        if profile.delay_ms > 0 {
            let idle = Animation::new(Duration::from_millis(profile.delay_ms as u64));
            return self.with_animations(id, vec![idle, animation], move |this, ix, delta| {
                if ix == 0 {
                    let waiting_progress = match stage {
                        TransitionStage::Enter => 0.0,
                        TransitionStage::Exit => 1.0,
                    };
                    apply_preset(this, profile, waiting_progress)
                } else {
                    let progress = match stage {
                        TransitionStage::Enter => delta,
                        TransitionStage::Exit => 1.0 - delta,
                    };
                    apply_preset(this, profile, progress)
                }
            });
        }

        self.with_animation(id, animation, move |this, delta| {
            let progress = match stage {
                TransitionStage::Enter => delta,
                TransitionStage::Exit => 1.0 - delta,
            };
            apply_preset(this, profile, progress)
        })
    }
}

impl<E> TransitionExt for E where E: Sized + AnimationExt + Styled + 'static {}

fn easing_fn(easing: Easing) -> impl Fn(f32) -> f32 {
    move |delta| match easing {
        Easing::Linear => gpui::linear(delta),
        Easing::Ease => gpui::ease_in_out(delta),
        Easing::EaseIn => gpui::quadratic(delta),
        Easing::EaseOut => gpui::ease_out_quint()(delta),
        Easing::EaseInOut => gpui::ease_in_out(delta),
        Easing::Quadratic => gpui::quadratic(delta),
        Easing::QuintOut => gpui::ease_out_quint()(delta),
        Easing::BounceInOut => gpui::bounce(gpui::ease_out_quint())(delta),
    }
}

fn apply_preset<E: Styled>(element: E, profile: MotionTransition, progress: f32) -> E {
    let progress = progress.clamp(0.0, 1.0);
    let start_opacity = (profile.start_opacity_pct as f32 / 100.0).clamp(0.0, 1.0);
    let opacity = start_opacity + (1.0 - start_opacity) * progress;

    match profile.preset {
        TransitionPreset::None => element,
        TransitionPreset::Fade => element.opacity(opacity),
        TransitionPreset::FadeUp => element
            .opacity(opacity)
            .mt(px(offset_with_progress(profile.offset_px, progress))),
        TransitionPreset::FadeDown => element
            .opacity(opacity)
            .mt(px(-offset_with_progress(profile.offset_px, progress))),
        TransitionPreset::FadeLeft => element
            .opacity(opacity)
            .ml(px(offset_with_progress(profile.offset_px, progress))),
        TransitionPreset::FadeRight => element
            .opacity(opacity)
            .ml(px(-offset_with_progress(profile.offset_px, progress))),
        TransitionPreset::ScaleIn => element.opacity(opacity),
        TransitionPreset::Pop => {
            let eased = (progress * std::f32::consts::PI).sin().abs() * 0.08;
            element
                .opacity(opacity)
                .mt(px(-eased * profile.offset_px as f32))
        }
        TransitionPreset::Bounce => {
            let wave = (progress * std::f32::consts::PI * 2.0).sin().abs();
            element
                .opacity(opacity)
                .mt(px(-(profile.offset_px as f32 * 0.2) * wave))
        }
        TransitionPreset::Pulse => {
            let wave = (progress * std::f32::consts::PI * 2.0).sin().abs();
            element.opacity(0.35 + (0.65 * wave))
        }
        TransitionPreset::Shake => {
            let wave = (progress * std::f32::consts::PI * 6.0).sin();
            element
                .opacity(opacity)
                .ml(px((profile.offset_px as f32 * 0.2) * wave))
        }
    }
}
