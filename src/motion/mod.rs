#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MotionLevel {
    Full,
    Reduced,
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionPreset {
    None,
    Fade,
    FadeUp,
    FadeDown,
    FadeLeft,
    FadeRight,
    ScaleIn,
    Pop,
    Bounce,
    Pulse,
    Shake,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Easing {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    Quadratic,
    QuintOut,
    BounceInOut,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpringConfig {
    pub stiffness: u16,
    pub damping: u16,
    pub mass: u16,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            stiffness: 220,
            damping: 18,
            mass: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MotionTransition {
    pub preset: TransitionPreset,
    pub duration_ms: u16,
    pub delay_ms: u16,
    pub offset_px: i16,
    pub start_opacity_pct: u8,
    pub easing: Easing,
    pub spring: Option<SpringConfig>,
}

impl Default for MotionTransition {
    fn default() -> Self {
        Self {
            preset: TransitionPreset::Fade,
            duration_ms: 220,
            delay_ms: 0,
            offset_px: 8,
            start_opacity_pct: 0,
            easing: Easing::EaseOut,
            spring: None,
        }
    }
}

impl MotionTransition {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn preset(mut self, preset: TransitionPreset) -> Self {
        self.preset = preset;
        self
    }

    pub fn duration_ms(mut self, duration_ms: u16) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    pub fn delay_ms(mut self, delay_ms: u16) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    pub fn offset_px(mut self, offset_px: i16) -> Self {
        self.offset_px = offset_px;
        self
    }

    pub fn start_opacity_pct(mut self, start_opacity_pct: u8) -> Self {
        self.start_opacity_pct = start_opacity_pct.min(100);
        self
    }

    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    pub fn spring(mut self, spring: SpringConfig) -> Self {
        self.spring = Some(spring);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MotionConfig {
    pub level: MotionLevel,
    pub enter: MotionTransition,
    pub exit: MotionTransition,
}

impl Default for MotionConfig {
    fn default() -> Self {
        Self {
            level: MotionLevel::Full,
            enter: MotionTransition::default(),
            exit: MotionTransition::new()
                .preset(TransitionPreset::Fade)
                .duration_ms(160)
                .easing(Easing::EaseIn),
        }
    }
}

impl MotionConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn level(mut self, level: MotionLevel) -> Self {
        self.level = level;
        self
    }

    pub fn enter(mut self, enter: MotionTransition) -> Self {
        self.enter = enter;
        self
    }

    pub fn exit(mut self, exit: MotionTransition) -> Self {
        self.exit = exit;
        self
    }
}
