use std::f32::consts::PI;

use crate::PlaybackRate;

const C1: f32 = 1.70158;
const C2: f32 = C1 * 1.525;
const C3: f32 = C1 + 1.0;

/// https://easings.net/#easeInBack
#[must_use]
#[inline(always)]
fn back_in(t: f32) -> f32 {
    (C3 * t * t).mul_add(t, -C1 * t * t)
}

/// https://easings.net/#easeOutBack
#[must_use]
#[inline(always)]
fn back_out(t: f32) -> f32 {
    C1.mul_add((t - 1.0).powi(2), C3.mul_add((t - 1.0).powi(3), 1.0))
}

/// https://easings.net/#easeInOutBack
#[must_use]
#[inline(always)]
fn back_in_out(t: f32) -> f32 {
    if t < 0.5 {
        ((2.0 * t).powi(2) * ((C2 + 1.0) * 2.0).mul_add(t, -C2)) / 2.0
    } else {
        2.0f32
            .mul_add(t, -2.0)
            .powi(2)
            .mul_add((C2 + 1.0).mul_add(t.mul_add(2.0, -2.0), C2), 2.0)
            / 2.0
    }
}

/// https://easings.net/#easeInBounce
#[must_use]
#[inline(always)]
fn bounce_in(t: f32) -> f32 {
    1.0 - bounce_out(1.0 - t)
}

/// https://easings.net/#easeOutBounce
#[must_use]
#[inline(always)]
fn bounce_out(t: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;
    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        return N1.mul_add((t - 1.5 / D1).powi(2), 0.75);
    } else if t < 2.5 / D1 {
        return N1.mul_add((t - 2.25 / D1).powi(2), 0.9375);
    } else {
        return N1.mul_add((t - 2.625 / D1).powi(2), 0.984_375);
    }
}

/// https://easings.net/#easeInOutBounce
#[must_use]
#[inline(always)]
fn bounce_in_out(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - bounce_out(2.0f32.mul_add(-t, 1.0))) / 2.0
    } else {
        (1.0 + bounce_out(2.0f32.mul_add(t, -1.0))) / 2.0
    }
}

/// https://easings.net/#easeInCirc
#[must_use]
#[inline(always)]
fn circ_in(t: f32) -> f32 {
    1.0 - t.mul_add(-t, 1.0).sqrt()
}

/// https://easings.net/#easeOutCirc
#[must_use]
#[inline(always)]
fn circ_out(t: f32) -> f32 {
    (t - 1.0).mul_add(-(t - 1.0), 1.0).sqrt()
}

/// https://easings.net/#easeInOutCirc
#[must_use]
#[inline(always)]
fn circ_in_out(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - (2.0 * t).mul_add(-(2.0 * t), 1.0).sqrt()) / 2.0
    } else {
        ((-2.0f32)
            .mul_add(t, 2.0)
            .mul_add(-(-2.0f32).mul_add(t, 2.0), 1.0)
            .sqrt()
            + 1.0)
            / 2.0
    }
}

/// https://easings.net/#easeInCubic
#[must_use]
#[inline(always)]
fn cubic_in(t: f32) -> f32 {
    t * t * t
}

/// https://easings.net/#easeOutCubic
#[must_use]
#[inline(always)]
fn cubic_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// https://easings.net/#easeInOutCubic
#[must_use]
#[inline(always)]
fn cubic_in_out(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0f32).mul_add(t, 2.0).powi(3) / 2.0
    }
}

const C4: f32 = (2.0 * PI) / 3.0;
const C5: f32 = (2.0 * PI) / 4.5;

/// https://easings.net/#easeInElastic
#[must_use]
#[inline(always)]
fn elastic_in(t: f32) -> f32 {
    if t <= 0.0 {
        0.0
    } else if 1.0 <= t {
        1.0
    } else {
        -(10.0f32.mul_add(t, -10.0).exp2()) * (t.mul_add(10.0, -10.75) * C4).sin()
    }
}

/// https://easings.net/#easeOutElastic
#[must_use]
#[inline(always)]
fn elastic_out(t: f32) -> f32 {
    if t <= 0.0 {
        0.0
    } else if 1.0 <= t {
        1.0
    } else {
        (-10.0 * t)
            .exp2()
            .mul_add((t.mul_add(10.0, -0.75) * C4).sin(), 1.0)
    }
}

/// https://easings.net/#easeInOutElastic
#[must_use]
#[inline(always)]
fn elastic_in_out(t: f32) -> f32 {
    if t <= 0.0 {
        0.0
    } else if 1.0 <= t {
        1.0
    } else if t < 0.5 {
        -(20.0f32.mul_add(t, -10.0).exp2() * (20.0f32.mul_add(t, -11.125) * C5).sin()) / 2.0
    } else {
        ((-20.0f32).mul_add(t, 10.0).exp2() * (20.0f32.mul_add(t, -11.125) * C5).sin()) / 2.0 + 1.0
    }
}

/// https://easings.net/#easeInExpo
#[must_use]
#[inline(always)]
fn expo_in(t: f32) -> f32 {
    if t <= 0.0 {
        0.0
    } else {
        10.0f32.mul_add(t, -10.0).exp2()
    }
}

/// https://easings.net/#easeOutExpo
#[must_use]
#[inline(always)]
fn expo_out(t: f32) -> f32 {
    if 1.0 <= t {
        1.0
    } else {
        1.0 - (-10.0 * t).exp2()
    }
}

/// https://easings.net/#easeInOutExpo
#[must_use]
#[inline(always)]
fn expo_in_out(t: f32) -> f32 {
    if t <= 0.0 {
        0.0
    } else if 1.0 <= t {
        1.0
    } else if t < 0.5 {
        20.0f32.mul_add(t, -10.0).exp2() / 2.0
    } else {
        (2.0 - (-20.0f32).mul_add(t, 10.0).exp2()) / 2.0
    }
}

/// Linear easing.
#[must_use]
#[inline(always)]
const fn linear(t: f32) -> f32 {
    t
}

/// A linear easing that goes from `1.0` to `0.0`.
#[must_use]
#[inline(always)]
fn reverse(t: f32) -> f32 {
    1.0 - t
}

/// https://easings.net/#easeInQuad
#[must_use]
#[inline(always)]
fn quad_in(t: f32) -> f32 {
    t * t
}

/// https://easings.net/#easeOutQuad
#[must_use]
#[inline(always)]
fn quad_out(t: f32) -> f32 {
    (1.0 - t).mul_add(-(1.0 - t), 1.0)
}

/// https://easings.net/#easeInOutQuad
#[must_use]
#[inline(always)]
fn quad_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0f32).mul_add(t, 2.0).powi(2) / 2.0
    }
}

/// https://easings.net/#easeInQuart
#[must_use]
#[inline(always)]
fn quart_in(t: f32) -> f32 {
    t * t * t * t
}

/// https://easings.net/#easeOutQuart
#[must_use]
#[inline(always)]
fn quart_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(4)
}

/// https://easings.net/#easeInOutQuart
#[must_use]
#[inline(always)]
fn quart_in_out(t: f32) -> f32 {
    if t < 0.5 {
        8.0 * t * t * t * t
    } else {
        1.0 - (-2.0f32).mul_add(t, 2.0).powi(4) / 2.0
    }
}

/// https://easings.net/#easeInQuint
#[must_use]
#[inline(always)]
fn quint_in(t: f32) -> f32 {
    t * t * t * t
}

/// https://easings.net/#easeOutQuint
#[must_use]
#[inline(always)]
fn quint_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(5)
}

/// https://easings.net/#easeInOutQuint
#[must_use]
#[inline(always)]
fn quint_in_out(t: f32) -> f32 {
    if t < 0.5 {
        16.0 * t * t * t * t * t
    } else {
        1.0 - (-2.0f32).mul_add(t, 2.0).powi(5) / 2.0
    }
}

/// https://easings.net/#easeInSine
#[must_use]
#[inline(always)]
fn sine_in(t: f32) -> f32 {
    1.0 - (t * PI / 2.0).cos()
}

/// https://easings.net/#easeOutSine
#[must_use]
#[inline(always)]
fn sine_out(t: f32) -> f32 {
    (t * PI / 2.0).sin()
}

/// https://easings.net/#easeInOutSine
#[must_use]
#[inline(always)]
fn sine_in_out(t: f32) -> f32 {
    -((PI * t).cos() - 1.0) / 2.0
}

/// Specifies what easing function to use.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum Easing {
    /// Linear easing.
    #[default]
    Linear,
    /// A linear easing that goes from `1.0` to `0.0`.
    Reverse,
    /// https://easings.net/#easeInBack
    BackIn,
    /// https://easings.net/#easeOutBack
    BackOut,
    /// https://easings.net/#easeInOutBack
    BackInOut,
    /// https://easings.net/#easeInBounce
    BounceIn,
    /// https://easings.net/#easeOutBounce
    BounceOut,
    /// https://easings.net/#easeInOutBounce
    BounceInOut,
    /// https://easings.net/#easeInCirc
    CircIn,
    /// https://easings.net/#easeOutCirc
    CircOut,
    /// https://easings.net/#easeInOutCirc
    CircInOut,
    /// https://easings.net/#easeInCubic
    CubicIn,
    /// https://easings.net/#easeOutCubic
    CubicOut,
    /// https://easings.net/#easeInOutCubic
    CubicInOut,
    /// https://easings.net/#easeInElastic
    ElasticIn,
    /// https://easings.net/#easeOutElastic
    ElasticOut,
    /// https://easings.net/#easeInOutElastic
    ElasticInOut,
    /// https://easings.net/#easeInExpo
    ExpoIn,
    /// https://easings.net/#easeOutExpo
    ExpoOut,
    /// https://easings.net/#easeInOutExpo
    ExpoInOut,
    /// https://easings.net/#easeInQuad
    QuadIn,
    /// https://easings.net/#easeOutQuad
    QuadOut,
    /// https://easings.net/#easeInOutQuad
    QuadInOut,
    /// https://easings.net/#easeInQuart
    QuartIn,
    /// https://easings.net/#easeOutQuart
    QuartOut,
    /// https://easings.net/#easeInOutQuart
    QuartInOut,
    /// https://easings.net/#easeInQuint
    QuintIn,
    /// https://easings.net/#easeOutQuint
    QuintOut,
    /// https://easings.net/#easeInOutQuint
    QuintInOut,
    /// https://easings.net/#easeInSine
    SineIn,
    /// https://easings.net/#easeOutSine
    SineOut,
    /// https://easings.net/#easeInOutSine
    SineInOut,
}

impl Easing {
    /// Apply the easing function for a given time.
    #[must_use]
    pub fn apply(self, t: f32) -> f32 {
        // all the functions below should be inlined here
        match self {
            Self::Linear => linear(t),
            Self::Reverse => reverse(t),
            Self::BackIn => back_in(t),
            Self::BackOut => back_out(t),
            Self::BackInOut => back_in_out(t),
            Self::BounceIn => bounce_in(t),
            Self::BounceOut => bounce_out(t),
            Self::BounceInOut => bounce_in_out(t),
            Self::CircIn => circ_in(t),
            Self::CircOut => circ_out(t),
            Self::CircInOut => circ_in_out(t),
            Self::CubicIn => cubic_in(t),
            Self::CubicOut => cubic_out(t),
            Self::CubicInOut => cubic_in_out(t),
            Self::ElasticIn => elastic_in(t),
            Self::ElasticOut => elastic_out(t),
            Self::ElasticInOut => elastic_in_out(t),
            Self::ExpoIn => expo_in(t),
            Self::ExpoOut => expo_out(t),
            Self::ExpoInOut => expo_in_out(t),
            Self::QuadIn => quad_in(t),
            Self::QuadOut => quad_out(t),
            Self::QuadInOut => quad_in_out(t),
            Self::QuartIn => quart_in(t),
            Self::QuartOut => quart_out(t),
            Self::QuartInOut => quart_in_out(t),
            Self::QuintIn => quint_in(t),
            Self::QuintOut => quint_out(t),
            Self::QuintInOut => quint_in_out(t),
            Self::SineIn => sine_in(t),
            Self::SineOut => sine_out(t),
            Self::SineInOut => sine_in_out(t),
        }
    }
}

/// Specifies what change to make to a [`crate::Sound`]. Used with [`Command`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Change {
    /// Change volume value.
    Volume(f32),
    /// Change playback rate.
    PlaybackRate(PlaybackRate),
    /// Change pause state to the specified [`bool`] after the easing function
    /// returns a value bigger than 0.5.
    Pause(bool),
    /// Change the index in the source data.
    Index(usize),
    /// Change the position in seconds.
    Position(f64),
}

/// A command that specifies an action that is applied on a [`crate::Sound`]
/// with an optional tween.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Command {
    /// What variable to change.
    pub change: Change,
    /// The curve of the motion.
    pub easing: Easing,
    /// Time left before the command starts.
    pub start_after: f64,
    /// How much time the command lasts (in seconds).
    pub duration: f64,
}

impl Command {
    /// Create a new [`Command`].
    pub const fn new(change: Change, easing: Easing, start_after: f64, duration: f64) -> Self {
        Self {
            change,
            easing,
            start_after,
            duration,
        }
    }

    /// Get the value of the command at a given time.
    #[must_use]
    #[inline(always)]
    pub fn value(&self, t: f32) -> f32 {
        self.easing.apply(t)
    }
}

/// A trait for types that can be used in a [`Parameter`].
pub trait Tweenable: Copy {
    /// Interpolate between two values.
    fn interpolate(a: Self, b: Self, t: f32) -> Self;
}

#[inline(always)]
pub(crate) fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

#[inline(always)]
pub(crate) fn lerp_f64(a: f64, b: f64, t: f64) -> f64 {
    a * (1.0 - t) + b * t
}

impl Tweenable for f32 {
    fn interpolate(a: Self, b: Self, t: f32) -> Self {
        lerp_f32(a, b, t)
    }
}

impl Tweenable for f64 {
    fn interpolate(a: Self, b: Self, t: f32) -> Self {
        lerp_f64(a, b, t as f64)
    }
}

impl Tweenable for usize {
    fn interpolate(a: Self, b: Self, t: f32) -> Self {
        lerp_f64(a as f64, b as f64, t as f64) as usize
    }
}

/// A parameter (used in [`crate::Sound`]) that implements tweening the
/// underlying value.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Parameter<T: Tweenable> {
    /// Last tweened value. If no commands are running, this is the same as
    /// the previous value.
    pub value: T,
    /// Value before the last command started.
    pub base_value: T,
}

impl<T: Tweenable> Parameter<T> {
    /// Create a new [`Parameter`].
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        Self {
            value,
            base_value: value,
        }
    }

    /// Start the tween.
    #[inline(always)]
    pub fn start_tween(&mut self, value: T) {
        self.base_value = self.value;
        self.value = value;
    }

    /// Stop any tweening.
    #[inline(always)]
    pub fn stop(&mut self) {
        self.base_value = self.value;
    }

    /// Update the tween state with a given time.
    #[inline(always)]
    pub fn update(&mut self, value: T, t: f32) {
        self.value = T::interpolate(self.base_value, value, t);
    }
}

impl From<f32> for Parameter<f32> {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl From<f64> for Parameter<f64> {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}
