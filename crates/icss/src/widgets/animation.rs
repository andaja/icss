//! Tick-driven animation primitives for iced.
//!
//! Iced has no built-in transitions, so we drive animations via
//! `iced::time::every(16ms)` subscriptions and interpolate manually.
//!
//! # Usage
//!
//! ```ignore
//! // In state:
//! fade: Animation,
//!
//! // Start:
//! self.fade.start(AnimKind::FadeIn, Duration::from_millis(500));
//!
//! // In update (on tick):
//! self.fade.tick(now);
//!
//! // In view:
//! let alpha = self.fade.value(); // 0.0 → 1.0
//! ```

use std::time::{Duration, Instant};

/// Easing curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    /// Cubic ease-in-out: smooth start and end.
    EaseInOut,
    /// Cubic ease-in: slow start, fast end.
    EaseIn,
    /// Cubic ease-out: fast start, slow end.
    EaseOut,
    /// Linear interpolation.
    Linear,
}

impl Easing {
    /// Map linear progress `t` (0.0–1.0) through the easing curve.
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t * t,
            Easing::EaseOut => {
                let inv = 1.0 - t;
                1.0 - inv * inv * inv
            }
            Easing::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
        }
    }
}

/// Animation direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimKind {
    /// Fade from 0 → 1 (invisible → visible).
    FadeIn,
    /// Fade from 1 → 0 (visible → invisible).
    FadeOut,
    /// Slide in from the given edge. Value goes 1.0 → 0.0 (offset → no offset).
    SlideIn(Edge),
    /// Slide out toward the given edge. Value goes 0.0 → 1.0 (no offset → offset).
    SlideOut(Edge),
}

/// Edge for slide animations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

/// A single animation instance.
#[derive(Debug, Clone)]
pub struct Animation {
    kind: AnimKind,
    easing: Easing,
    duration: Duration,
    started: Option<Instant>,
    progress: f32, // 0.0 → 1.0 (linear, before easing)
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            kind: AnimKind::FadeIn,
            easing: Easing::EaseInOut,
            duration: Duration::from_millis(500),
            started: None,
            progress: 0.0,
        }
    }
}

impl Animation {
    /// Create a new idle animation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start (or restart) the animation with the given kind and duration.
    pub fn start(&mut self, kind: AnimKind, duration: Duration) {
        self.kind = kind;
        self.duration = duration;
        self.started = Some(Instant::now());
        self.progress = 0.0;
    }

    /// Start with a specific easing.
    pub fn start_with(&mut self, kind: AnimKind, duration: Duration, easing: Easing) {
        self.easing = easing;
        self.start(kind, duration);
    }

    /// Advance the animation to the current time. Call this from your tick handler.
    pub fn tick(&mut self, now: Instant) {
        if let Some(start) = self.started {
            let elapsed = now.duration_since(start);
            self.progress = (elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0);
            if self.progress >= 1.0 {
                self.started = None; // done
            }
        }
    }

    /// Whether the animation is currently running.
    pub fn is_running(&self) -> bool {
        self.started.is_some()
    }

    /// Whether the animation has completed (ran and finished).
    pub fn is_done(&self) -> bool {
        self.started.is_none() && self.progress >= 1.0
    }

    /// Whether the animation is idle (never started or reset).
    pub fn is_idle(&self) -> bool {
        self.started.is_none() && self.progress == 0.0
    }

    /// Reset to idle state.
    pub fn reset(&mut self) {
        self.started = None;
        self.progress = 0.0;
    }

    /// Raw linear progress (0.0–1.0).
    pub fn linear_progress(&self) -> f32 {
        self.progress
    }

    /// Eased progress (0.0–1.0), mapped through the easing curve.
    fn eased(&self) -> f32 {
        self.easing.apply(self.progress)
    }

    /// The current animation value, interpreted by kind:
    ///
    /// - `FadeIn`: alpha 0.0 → 1.0
    /// - `FadeOut`: alpha 1.0 → 0.0
    /// - `SlideIn`: offset 1.0 → 0.0 (multiply by your slide distance)
    /// - `SlideOut`: offset 0.0 → 1.0
    pub fn value(&self) -> f32 {
        let e = self.eased();
        match self.kind {
            AnimKind::FadeIn => e,
            AnimKind::FadeOut => 1.0 - e,
            AnimKind::SlideIn(_) => 1.0 - e,
            AnimKind::SlideOut(_) => e,
        }
    }

    /// Convenience: current alpha for fade animations. Returns 1.0 for non-fade kinds.
    pub fn alpha(&self) -> f32 {
        match self.kind {
            AnimKind::FadeIn | AnimKind::FadeOut => self.value(),
            _ => 1.0,
        }
    }

    /// Convenience: current offset for slide animations. Returns 0.0 for non-slide kinds.
    /// Multiply by your desired slide distance in pixels.
    pub fn offset(&self) -> f32 {
        match self.kind {
            AnimKind::SlideIn(_) | AnimKind::SlideOut(_) => self.value(),
            _ => 0.0,
        }
    }

    /// The edge this slide animation targets, if any.
    pub fn edge(&self) -> Option<Edge> {
        match self.kind {
            AnimKind::SlideIn(e) | AnimKind::SlideOut(e) => Some(e),
            _ => None,
        }
    }

    /// Convert offset to padding for use with iced containers.
    /// `distance` is the max slide distance in pixels.
    /// Returns `[top, right, bottom, left]` padding values.
    pub fn slide_padding(&self, distance: f32) -> [f32; 4] {
        let px = self.offset() * distance;
        match self.edge() {
            Some(Edge::Top) => [px, 0.0, 0.0, 0.0],
            Some(Edge::Bottom) => [0.0, 0.0, px, 0.0],
            Some(Edge::Left) => [0.0, 0.0, 0.0, px],
            Some(Edge::Right) => [0.0, px, 0.0, 0.0],
            None => [0.0; 4],
        }
    }
}
