//! Scroll physics engine — pure Rust.
//!
//! Physics: velocity-based with exponential friction decay + rubber-band springs at bounds.
//! Animation loop: `spawn` + `tokio::time::sleep` ticking at ~60fps.
//! DOM writes: single-line `eval()` calls setting `style.transform` properties.

use dioxus::document::eval;

pub const TAU: f64 = 0.22;
pub const ACCEL: f64 = 4.5;
pub const INPUT_TAU: f64 = 0.05;
pub const MAX_V: f64 = 15000.0;
pub const R_K: f64 = 250.0;
pub const R_DAMP: f64 = 30.0;
pub const R_VIS_MAX: f64 = 60.0;
pub const STICK_HEIGHTS: [f64; 3] = [0.0, 45.0, 90.0];

fn snap_px(value: f64) -> f64 {
    value.round()
}

pub struct ScrollPhysics {
    pub offset: f64,
    pub velocity: f64,
    pending_wheel_delta: f64,
}

impl Default for ScrollPhysics {
    fn default() -> Self {
        Self {
            offset: 0.0,
            velocity: 0.0,
            pending_wheel_delta: 0.0,
        }
    }
}

impl ScrollPhysics {
    pub fn add_wheel_delta(&mut self, delta_y: f64, max_scroll: f64) {
        let mut delta = shape_wheel_delta(delta_y) * ACCEL;
        if (self.offset < 0.0 && delta < 0.0) || (self.offset > max_scroll && delta > 0.0) {
            delta *= 0.1;
        }
        self.pending_wheel_delta = (self.pending_wheel_delta + delta).clamp(-MAX_V, MAX_V);
    }

    pub fn tick(&mut self, dt: f64, max_scroll: f64) -> bool {
        self.flush_pending_wheel(dt);
        self.velocity *= (-dt / TAU).exp();

        if self.offset < 0.0 {
            self.velocity += (-self.offset * R_K - self.velocity * R_DAMP) * dt;
        } else if self.offset > max_scroll {
            self.velocity += (-(self.offset - max_scroll) * R_K - self.velocity * R_DAMP) * dt;
        }

        self.velocity = self.velocity.clamp(-MAX_V, MAX_V);
        self.offset += self.velocity * dt;

        let in_bounds = self.offset >= -0.5 && self.offset <= max_scroll + 0.5;
        if in_bounds && self.velocity.abs() < 1.0 {
            self.offset = self.offset.clamp(0.0, max_scroll);
            self.velocity = 0.0;
            return false;
        }
        true
    }

    pub fn visual_offset(&self, max_scroll: f64) -> f64 {
        if self.offset < 0.0 {
            let over = -self.offset;
            -R_VIS_MAX * (1.0 - (-over / R_VIS_MAX).exp())
        } else if self.offset > max_scroll {
            let over = self.offset - max_scroll;
            max_scroll + R_VIS_MAX * (1.0 - (-over / R_VIS_MAX).exp())
        } else {
            self.offset
        }
    }

    pub fn reset(&mut self) {
        self.offset = 0.0;
        self.velocity = 0.0;
        self.pending_wheel_delta = 0.0;
    }

    fn flush_pending_wheel(&mut self, dt: f64) {
        if self.pending_wheel_delta.abs() < 0.01 {
            self.pending_wheel_delta = 0.0;
            return;
        }

        let alpha = 1.0 - (-dt / INPUT_TAU).exp();
        let mut injected = self.pending_wheel_delta * alpha;
        if self.pending_wheel_delta.abs() < 1.0 {
            injected = self.pending_wheel_delta;
        }

        self.pending_wheel_delta -= injected;
        self.velocity = (self.velocity + injected).clamp(-MAX_V, MAX_V);
    }
}

fn shape_wheel_delta(delta_y: f64) -> f64 {
    let magnitude = delta_y.abs();
    if magnitude == 0.0 {
        0.0
    } else {
        delta_y.signum() * magnitude.powf(0.96)
    }
}

pub fn write_transforms(visual_offset: f64, header_natural_ys: &[f64]) {
    let snapped_visual_offset = snap_px(visual_offset);
    let mut script = format!(
        "document.getElementById('scroll-content').style.transform='translate3d(0,{}px,0)';",
        -snapped_visual_offset
    );

    let ids = ["sh-active", "sh-backlog", "sh-done"];
    for (idx, &natural_y) in header_natural_ys.iter().enumerate() {
        if idx >= ids.len() || idx >= STICK_HEIGHTS.len() {
            break;
        }
        let stick_at = STICK_HEIGHTS[idx];
        let stick_point = natural_y - stick_at;
        if snapped_visual_offset > stick_point {
            let ty = snap_px(snapped_visual_offset - natural_y + stick_at);
            script.push_str(&format!(
                "var h=document.getElementById('{}');if(h){{h.style.transform='translate3d(0,{}px,0)';h.style.zIndex='99'}}",
                ids[idx], ty
            ));
        } else {
            script.push_str(&format!(
                "var h=document.getElementById('{}');if(h){{h.style.transform='';h.style.zIndex=''}}",
                ids[idx]
            ));
        }
    }

    let _ = eval(&script);
}

pub fn jump_to_top() {
    let _ = eval(
        "var c=document.getElementById('scroll-content');\
         if(c){c.style.transform='translate3d(0,0px,0)';}\
         var ids=['sh-active','sh-backlog','sh-done'];\
         ids.forEach(function(id){var h=document.getElementById(id);if(h){h.style.transform='';h.style.zIndex='';}}"
    );
}

pub fn set_is_scrolling(scrolling: bool) {
    let _ = eval(&format!(
        "document.body.classList.toggle('is-scrolling', {})",
        scrolling
    ));
}

pub async fn measure_scroll_metrics() -> (f64, f64, f64) {
    let mut result = eval(
        "var c=document.getElementById('scroll-content'),v=document.querySelector('.content');\
         var scrollHeight=c?c.scrollHeight:0;\
         var viewportHeight=v?v.clientHeight:0;\
         var maxScroll=Math.max(scrollHeight-viewportHeight,0);\
         dioxus.send([maxScroll, viewportHeight, scrollHeight])",
    );
    let values = result.recv::<Vec<f64>>().await.unwrap_or_default();
    match values.as_slice() {
        [max_scroll, viewport_height, scroll_height, ..] => (
            (*max_scroll).max(0.0),
            (*viewport_height).max(0.0),
            (*scroll_height).max(0.0),
        ),
        _ => (0.0, 0.0, 0.0),
    }
}

pub async fn measure_header_positions() -> Vec<f64> {
    let mut result = eval(
        "var hs=document.querySelectorAll('.section-head');\
         var ys=[];hs.forEach(function(h){ys.push(h.offsetTop)});\
         dioxus.send(ys)",
    );
    result.recv::<Vec<f64>>().await.unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_scroll_physics_timing() {
        let mut physics = ScrollPhysics::default();
        let mut last_tick = Instant::now();
        let mut frames = 0;
        let mut total_time = 0.0;
        let mut max_frame_time = 0.0;

        physics.add_wheel_delta(-50.0, 1000.0);
        let max_scroll = 1000.0;

        let start = Instant::now();
        while Instant::now() - start < Duration::from_millis(50) {
            std::thread::sleep(Duration::from_millis(16));
            let now = Instant::now();
            let dt = (now - last_tick).as_secs_f64().clamp(0.001, 0.050);
            let dt_ms = dt * 1000.0;
            frames += 1;
            total_time += dt_ms;
            if dt_ms > max_frame_time {
                max_frame_time = dt_ms;
            }
            last_tick = now;
            let _ = physics.tick(dt, max_scroll);
        }

        let avg = total_time / frames as f64;
        println!(
            "Test [Scroll Metrics] Frames: {} | Avg: {:.1}ms | Max: {:.1}ms",
            frames, avg, max_frame_time
        );
        assert!(frames >= 2);
        assert!(max_frame_time < 30.0);
    }

    #[test]
    fn test_exponential_rubber_banding() {
        let mut physics = ScrollPhysics::default();
        let max_scroll = 1000.0;
        physics.offset = 500.0;
        assert_eq!(physics.visual_offset(max_scroll), 500.0);

        physics.offset = -1000.0;
        let top_visual = physics.visual_offset(max_scroll);
        assert!(top_visual >= -R_VIS_MAX);
        assert!(top_visual < -59.9);

        physics.offset = 2000.0;
        let bottom_visual = physics.visual_offset(max_scroll);
        assert!(bottom_visual <= max_scroll + R_VIS_MAX);
        assert!(bottom_visual > max_scroll + 59.9);
    }

    #[test]
    fn test_snap_px_rounds_to_whole_pixels() {
        assert_eq!(snap_px(12.49), 12.0);
        assert_eq!(snap_px(12.5), 13.0);
        assert_eq!(snap_px(-12.5), -13.0);
    }

    #[test]
    fn wheel_input_is_coalesced_across_ticks() {
        let mut physics = ScrollPhysics::default();
        physics.add_wheel_delta(120.0, 1000.0);

        physics.tick(0.016, 1000.0);
        let first_frame_velocity = physics.velocity;

        assert!(first_frame_velocity > 0.0);
        assert!(first_frame_velocity < shape_wheel_delta(120.0) * ACCEL);

        physics.tick(0.016, 1000.0);
        assert!(physics.velocity > first_frame_velocity);
    }

    #[test]
    fn test_manual_velocity_dampening_scrub() {
        let max_scroll = 1000.0;
        let initial_v = shape_wheel_delta(100.0) * ACCEL;

        let mut normal_coast = ScrollPhysics {
            velocity: initial_v,
            offset: 0.0,
            pending_wheel_delta: 0.0,
        };
        for _ in 0..60 {
            normal_coast.tick(0.016, max_scroll);
        }

        let mut scrub_coast = ScrollPhysics {
            velocity: initial_v,
            offset: 0.0,
            pending_wheel_delta: 0.0,
        };
        for idx in 0..60 {
            if idx < 3 {
                scrub_coast.velocity *= 0.85;
            }
            scrub_coast.tick(0.016, max_scroll);
        }

        assert!(scrub_coast.velocity < normal_coast.velocity);
        assert!((scrub_coast.velocity / normal_coast.velocity) < 0.62);
    }
}
