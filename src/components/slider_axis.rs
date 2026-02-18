use super::control;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SliderAxis {
    Horizontal,
    Vertical,
}

impl SliderAxis {
    pub fn length(self, width: f32, height: f32) -> f32 {
        match self {
            Self::Horizontal => width.max(1.0),
            Self::Vertical => height.max(1.0),
        }
    }

    pub fn local(self, pointer_x: f32, pointer_y: f32, origin_x: f32, origin_y: f32) -> f32 {
        match self {
            Self::Horizontal => pointer_x - origin_x,
            Self::Vertical => pointer_y - origin_y,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RailGeometry {
    pub origin_x: f32,
    pub origin_y: f32,
    pub width: f32,
    pub height: f32,
}

impl RailGeometry {
    pub fn from_state(id: &str, fallback_width: f32, fallback_height: f32) -> Self {
        let origin_x = control::f32_state(id, "rail-origin-x", None, 0.0);
        let origin_y = control::f32_state(id, "rail-origin-y", None, 0.0);
        let width = control::f32_state(id, "rail-width", None, fallback_width).max(1.0);
        let height = control::f32_state(id, "rail-height", None, fallback_height).max(1.0);
        Self {
            origin_x,
            origin_y,
            width,
            height,
        }
    }

    pub fn store(id: &str, origin_x: f32, origin_y: f32, width: f32, height: f32) {
        control::set_f32_state(id, "rail-origin-x", origin_x);
        control::set_f32_state(id, "rail-origin-y", origin_y);
        control::set_f32_state(id, "rail-width", width.max(1.0));
        control::set_f32_state(id, "rail-height", height.max(1.0));
    }
}

pub fn normalize(min: f32, max: f32, step: f32, raw: f32) -> f32 {
    let (min, max) = if min <= max { (min, max) } else { (max, min) };
    let step = step.max(0.001);
    let clamped = raw.clamp(min, max);
    let snapped = ((clamped - min) / step).round() * step + min;
    snapped.clamp(min, max)
}

pub fn normalize_pair(min: f32, max: f32, step: f32, left: f32, right: f32) -> (f32, f32) {
    let mut left = normalize(min, max, step, left);
    let mut right = normalize(min, max, step, right);
    if left > right {
        std::mem::swap(&mut left, &mut right);
    }
    (left, right)
}

pub fn ratio(min: f32, max: f32, value: f32) -> f32 {
    let span = (max - min).max(0.001);
    ((value - min) / span).clamp(0.0, 1.0)
}

pub fn value_from_local(axis: SliderAxis, local: f32, axis_len: f32, min: f32, max: f32) -> f32 {
    let len = axis_len.max(1.0);
    let ratio = match axis {
        SliderAxis::Horizontal => (local / len).clamp(0.0, 1.0),
        SliderAxis::Vertical => (1.0 - (local / len)).clamp(0.0, 1.0),
    };
    min + ((max - min).max(0.001) * ratio)
}

pub fn thumb_offset(axis: SliderAxis, track_len: f32, thumb_size: f32, value_ratio: f32) -> f32 {
    let span = (track_len - thumb_size).max(0.0);
    match axis {
        SliderAxis::Horizontal => span * value_ratio,
        SliderAxis::Vertical => span * (1.0 - value_ratio),
    }
}
