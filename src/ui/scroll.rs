//! Scroll physics engine — pure Rust.
//!
//! Physics: velocity-based with exponential friction decay + rubber-band springs at bounds.
//! Animation loop: `spawn` + `tokio::time::sleep` ticking at ~60fps.
//! DOM writes: single-line `eval()` calls setting `style.transform` properties
//! (same pattern as the existing `document.body.style.zoom` call in app.rs).

use dioxus::document::eval;

// ── Physics constants ────────────────────────────────────────────────────
pub const TAU: f64 = 0.22;           // Friction time constant (seconds). Lower = snappier.
pub const ACCEL: f64 = 4.5;          // Wheel delta → velocity multiplier.
pub const MAX_V: f64 = 15000.0;      // Velocity cap (px/s).
pub const R_K: f64 = 250.0;          // Rubber-band spring stiffness at bounds.
pub const R_DAMP: f64 = 30.0;        // Rubber-band damping.
pub const R_VIS_MAX: f64 = 60.0;     // Max visual overscroll (px).
pub const STICK_HEIGHTS: [f64; 3] = [0.0, 45.0, 90.0]; // Sticky header stack offsets.

/// Pure physics state — no JS, no DOM, just math.
pub struct ScrollPhysics {
    pub offset: f64,
    pub velocity: f64,
}

impl Default for ScrollPhysics {
    fn default() -> Self {
        Self { offset: 0.0, velocity: 0.0 }
    }
}

impl ScrollPhysics {
    /// Accumulate a wheel delta into current velocity (compounds during rapid scrolling).
    pub fn add_wheel_delta(&mut self, delta_y: f64, max_scroll: f64) {
        let mut d = delta_y * ACCEL;
        // Dampen input if we are pushing further into the overscroll rubber-band
        if (self.offset < 0.0 && d < 0.0) || (self.offset > max_scroll && d > 0.0) {
            d *= 0.1;
        }
        self.velocity += d;
        self.velocity = self.velocity.clamp(-MAX_V, MAX_V);
    }

    /// Advance physics by `dt` seconds. Returns `true` if still animating.
    pub fn tick(&mut self, dt: f64, max_scroll: f64) -> bool {
        // Exponential friction decay: v(t) = v₀ × e^(-t/τ)
        self.velocity *= (-dt / TAU).exp();

        // Rubber-band spring force at bounds
        if self.offset < 0.0 {
            self.velocity += (-self.offset * R_K - self.velocity * R_DAMP) * dt;
        } else if self.offset > max_scroll {
            self.velocity += (-(self.offset - max_scroll) * R_K - self.velocity * R_DAMP) * dt;
        }

        self.velocity = self.velocity.clamp(-MAX_V, MAX_V);
        self.offset += self.velocity * dt;

        // Convergence: sleep if inside bounds and nearly stopped
        let in_bounds = self.offset >= -0.5 && self.offset <= max_scroll + 0.5;
        if in_bounds && self.velocity.abs() < 1.0 {
            self.offset = self.offset.clamp(0.0, max_scroll);
            self.velocity = 0.0;
            return false;
        }
        true
    }

    /// Calculate soft offset for visual display (asymptotic rubber-band mapping).
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

    /// Reset scroll to top.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.offset = 0.0;
        self.velocity = 0.0;
    }
}

/// Write all scroll transforms in a single IPC call.
pub fn write_transforms(visual_offset: f64, header_natural_ys: &[f64]) {
    let mut script = format!(
        "document.getElementById('scroll-content').style.transform='translate3d(0,{}px,0)';",
        -visual_offset
    );

    let ids = ["sh-active", "sh-backlog", "sh-done"];
    for (i, &natural_y) in header_natural_ys.iter().enumerate() {
        if i >= ids.len() || i >= STICK_HEIGHTS.len() { break; }
        let stick_at = STICK_HEIGHTS[i];
        let stick_point = natural_y - stick_at;
        if visual_offset > stick_point {
            let ty = visual_offset - natural_y + stick_at;
            script.push_str(&format!(
                "var h=document.getElementById('{}');if(h){{h.style.transform='translate3d(0,{}px,0)';h.style.zIndex='99'}}",
                ids[i], ty
            ));
        } else {
            script.push_str(&format!(
                "var h=document.getElementById('{}');if(h){{h.style.transform='';h.style.zIndex=''}}",
                ids[i]
            ));
        }
    }
    let _ = eval(&script);
}

/// Toggle the `.is-scrolling` class on the root body element to disable paint-heavy hover states.
pub fn set_is_scrolling(scrolling: bool) {
    let _ = eval(&format!("document.body.classList.toggle('is-scrolling', {})", scrolling));
}

/// Measure the max scrollable distance from the DOM. Returns a future.
pub async fn measure_max_scroll() -> f64 {
    let mut result = eval(
        "var c=document.getElementById('scroll-content'),v=document.querySelector('.content');\
         dioxus.send(c&&v?c.scrollHeight-v.clientHeight:0)"
    );
    result.recv::<f64>().await.unwrap_or_default().max(0.0)
}

/// Measure the natural Y positions of section headers from the DOM. Returns a future.
pub async fn measure_header_positions() -> Vec<f64> {
    let mut result = eval(
        "var hs=document.querySelectorAll('.section-head');\
         var ys=[];hs.forEach(function(h){ys.push(h.offsetTop)});\
         dioxus.send(ys)"
    );
    result.recv::<Vec<f64>>().await.unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_scroll_physics_timing() {
        // Simulate the scroll loop locally to prove dt math and stable frame times don't stutter
        let mut physics = ScrollPhysics::default();
        
        let mut last_tick = Instant::now();
        let mut frames = 0;
        let mut total_time = 0.0;
        let mut max_frame_time = 0.0;

        // Kick it off
        physics.add_wheel_delta(-50.0, 1000.0);
        let max_scroll = 1000.0;

        let start = Instant::now();
        while Instant::now() - start < Duration::from_millis(50) {
            // We just sleep thread here for 16ms to simulate a frame
            std::thread::sleep(Duration::from_millis(16));

            let now = Instant::now();
            let dt = (now - last_tick).as_secs_f64().clamp(0.001, 0.050);
            let dt_ms = dt * 1000.0;
            
            frames += 1;
            total_time += dt_ms;
            if dt_ms > max_frame_time { max_frame_time = dt_ms; }
            last_tick = now;

            let _still_moving = physics.tick(dt, max_scroll);
        }

        let avg = total_time / (frames as f64);
        println!("Test [Scroll Metrics] Frames: {} | Avg: {:.1}ms | Max: {:.1}ms", frames, avg, max_frame_time);
        
        // Assert we got at least 2 frames and max wasn't egregiously high (allowing OS variance, < 25ms is solid 60ish fps bounds)
        assert!(frames >= 2);
        assert!(max_frame_time < 30.0);
    }

    #[test]
    fn test_exponential_rubber_banding() {
        let mut physics = ScrollPhysics::default();
        let max_scroll = 1000.0;
        
        // Test 1: Normal bounds tracking
        physics.offset = 500.0;
        assert_eq!(physics.visual_offset(max_scroll), 500.0);

        // Test 2: Negative overscroll (top rubber band)
        // Push offset way past bounds
        physics.offset = -1000.0;
        let top_visual = physics.visual_offset(max_scroll);
        // It must never exceed -R_VIS_MAX (-60) but should be very close to it
        assert!(top_visual >= -R_VIS_MAX);
        assert!(top_visual < -59.9);

        // Test 3: Positive overscroll (bottom rubber band)
        physics.offset = 2000.0;
        let bottom_visual = physics.visual_offset(max_scroll);
        // It must never exceed max_scroll + R_VIS_MAX (1060)
        assert!(bottom_visual <= max_scroll + R_VIS_MAX);
        assert!(bottom_visual > max_scroll + 59.9);
    }

    #[test]
    fn test_manual_velocity_dampening_scrub() {
        let mut physics = ScrollPhysics::default();
        let max_scroll = 1000.0;
        
        // Give it a massive velocity
        physics.add_wheel_delta(100.0, max_scroll);
        let initial_v = physics.velocity;
        
        // Simulate normal coasting for 1 second
        let mut normal_coast = ScrollPhysics { velocity: initial_v, offset: 0.0 };
        for _ in 0..60 { normal_coast.tick(0.016, max_scroll); }
        
        // Simulate coasting with the user "scrubbing" the mouse for exactly 3 ticks
        let mut scrub_coast = ScrollPhysics { velocity: initial_v, offset: 0.0 };
        for i in 0..60 { 
            if i < 3 {
                // The exact logic from our onpointermove block
                scrub_coast.velocity *= 0.85; 
            }
            scrub_coast.tick(0.016, max_scroll);
        }
        
        // The scrub coast should have halted significantly faster, proving the modifier 
        // exponentially decays the momentum compared to baseline TAU.
        assert!(scrub_coast.velocity < normal_coast.velocity);
        
        // With 3 consecutive scrub ticks, the velocity should drop precisely by ~38.5%
        // independently of the baseline friction (0.85^3 ~ 0.614)
        assert!((scrub_coast.velocity / normal_coast.velocity) < 0.62);
    }
}
