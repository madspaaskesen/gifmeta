//! Keyframe / delta-frame detection for (possibly) frame-optimized GIFs.
//!
//! A frame is a *keyframe* when it does not depend on anything earlier frames left on the
//! canvas. We track a conservative bounding box of the canvas area that still holds content
//! from earlier frames ("dirty" area). A frame is a keyframe when
//! - the canvas is empty (nothing left from earlier frames), or
//! - the frame fully covers the dirty area with opaque pixels.
//!
//! The tracking is conservative: it may call a frame a delta frame when it is in fact
//! independent, but it never calls a dependent frame a keyframe.

use gif::DisposalMethod;

/// Axis-aligned rectangle, `right`/`bottom` exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl Rect {
    /// Builds a frame rectangle clipped to the canvas. Returns `None` if it is empty.
    pub fn clipped(
        left: u16,
        top: u16,
        width: u16,
        height: u16,
        canvas_w: u16,
        canvas_h: u16,
    ) -> Option<Rect> {
        let r = Rect {
            left: left as u32,
            top: top as u32,
            right: (left as u32 + width as u32).min(canvas_w as u32),
            bottom: (top as u32 + height as u32).min(canvas_h as u32),
        };
        (r.left < r.right && r.top < r.bottom).then_some(r)
    }

    pub fn contains(&self, other: &Rect) -> bool {
        self.left <= other.left
            && self.top <= other.top
            && self.right >= other.right
            && self.bottom >= other.bottom
    }

    pub fn union(&self, other: &Rect) -> Rect {
        Rect {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }
}

/// Tracks which canvas area still holds content from earlier frames.
#[derive(Debug, Default)]
pub struct KeyframeTracker {
    dirty: Option<Rect>,
}

impl KeyframeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds the next frame and returns `true` if it is a keyframe.
    ///
    /// * `rect` – frame rectangle clipped to the canvas (`None` = draws nothing)
    /// * `uses_transparent_pixels` – at least one pixel uses the transparent index
    /// * `dispose` – the frame's disposal method
    pub fn push(
        &mut self,
        rect: Option<Rect>,
        uses_transparent_pixels: bool,
        dispose: DisposalMethod,
    ) -> bool {
        let is_keyframe = match (self.dirty, rect) {
            (None, _) => true,
            (Some(dirty), Some(r)) => !uses_transparent_pixels && r.contains(&dirty),
            (Some(_), None) => false,
        };

        self.dirty = match dispose {
            DisposalMethod::Any | DisposalMethod::Keep => match (self.dirty, rect) {
                (Some(d), Some(r)) => Some(d.union(&r)),
                (d, r) => d.or(r),
            },
            // The frame's area is cleared. Anything outside it (from earlier frames) remains;
            // we only shrink to empty when the frame's area covered all earlier content.
            DisposalMethod::Background => match (self.dirty, rect) {
                (Some(d), Some(r)) if r.contains(&d) => None,
                (d, _) => d,
            },
            // Canvas is restored to the state before this frame.
            DisposalMethod::Previous => self.dirty,
        };

        is_keyframe
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(l: u32, t: u32, r: u32, b: u32) -> Option<Rect> {
        Some(Rect {
            left: l,
            top: t,
            right: r,
            bottom: b,
        })
    }

    #[test]
    fn first_frame_is_always_keyframe() {
        let mut t = KeyframeTracker::new();
        assert!(t.push(rect(10, 10, 20, 20), true, DisposalMethod::Keep));
    }

    #[test]
    fn full_opaque_frames_are_keyframes() {
        let mut t = KeyframeTracker::new();
        for _ in 0..3 {
            assert!(t.push(rect(0, 0, 100, 100), false, DisposalMethod::Keep));
        }
    }

    #[test]
    fn small_frame_on_top_is_delta() {
        let mut t = KeyframeTracker::new();
        assert!(t.push(rect(0, 0, 100, 100), false, DisposalMethod::Keep));
        assert!(!t.push(rect(20, 20, 40, 40), false, DisposalMethod::Keep));
    }

    #[test]
    fn transparent_pixels_make_full_frame_delta() {
        let mut t = KeyframeTracker::new();
        t.push(rect(0, 0, 100, 100), false, DisposalMethod::Keep);
        assert!(!t.push(rect(0, 0, 100, 100), true, DisposalMethod::Keep));
    }

    #[test]
    fn background_disposal_clears_canvas() {
        let mut t = KeyframeTracker::new();
        t.push(rect(10, 10, 50, 50), true, DisposalMethod::Background);
        // Canvas is empty again → next frame is independent even with transparency.
        assert!(t.push(rect(30, 30, 60, 60), true, DisposalMethod::Background));
    }

    #[test]
    fn background_disposal_of_small_frame_keeps_earlier_content() {
        let mut t = KeyframeTracker::new();
        t.push(rect(0, 0, 100, 100), false, DisposalMethod::Keep);
        t.push(rect(10, 10, 20, 20), false, DisposalMethod::Background);
        assert!(!t.push(rect(50, 50, 60, 60), false, DisposalMethod::Keep));
    }

    #[test]
    fn previous_disposal_restores_state() {
        let mut t = KeyframeTracker::new();
        t.push(rect(0, 0, 10, 10), false, DisposalMethod::Background); // canvas empty after
        t.push(rect(0, 0, 100, 100), false, DisposalMethod::Previous); // restored → empty
        assert!(t.push(rect(40, 40, 50, 50), true, DisposalMethod::Keep));
    }

    #[test]
    fn opaque_frame_covering_dirty_area_is_keyframe() {
        let mut t = KeyframeTracker::new();
        t.push(rect(10, 10, 20, 20), false, DisposalMethod::Keep);
        assert!(t.push(rect(0, 0, 30, 30), false, DisposalMethod::Keep));
    }

    #[test]
    fn rect_is_clipped_to_canvas() {
        assert_eq!(
            Rect::clipped(90, 90, 20, 20, 100, 100),
            rect(90, 90, 100, 100)
        );
        assert_eq!(Rect::clipped(100, 0, 5, 5, 100, 100), None);
    }
}
