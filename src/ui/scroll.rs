//! Scroll physics engine — pure Rust.
//!
//! Physics: velocity-based with exponential friction decay + rubber-band springs at bounds.
//! Animation loop: `spawn` + `tokio::time::sleep` ticking at ~60fps.
//! DOM writes: single-line `eval()` calls setting `style.transform` properties
//! (same pattern as the existing `document.body.style.zoom` call in app.rs).

use dioxus::document::eval;

// ── Physics constants ────────────────────────────────────────────────────
pub const TAU: f64 = 0.35;           // Friction time constant (seconds). Lower = snappier.
pub const ACCEL: f64 = 4.5;          // Wheel delta → velocity multiplier.
pub const MAX_V: f64 = 15000.0;      // Velocity cap (px/s).
pub const R_K: f64 = 120.0;          // Rubber-band spring stiffness at bounds.
pub const R_DAMP: f64 = 15.0;        // Rubber-band damping.
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
    pub fn add_wheel_delta(&mut self, delta_y: f64) {
        self.velocity += delta_y * ACCEL;
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

    /// Clamp offset for visual display (allows slight rubber-band overscroll).
    pub fn visual_offset(&self, max_scroll: f64) -> f64 {
        self.offset.clamp(-R_VIS_MAX, max_scroll + R_VIS_MAX)
    }

    /// Reset scroll to top.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.offset = 0.0;
        self.velocity = 0.0;
    }
}

/// Write scroll transform to the DOM. One eval() call, one style property assignment.
pub fn write_content_transform(visual_offset: f64) {
    let _ = eval(&format!(
        "document.getElementById('scroll-content').style.transform='translate3d(0,{}px,0)'",
        -visual_offset
    ));
}

/// Write sticky header transforms. Each header gets a single style.transform assignment.
pub fn write_header_transforms(visual_offset: f64, header_natural_ys: &[f64]) {
    let ids = ["sh-active", "sh-backlog", "sh-done"];
    for (i, &natural_y) in header_natural_ys.iter().enumerate() {
        if i >= ids.len() || i >= STICK_HEIGHTS.len() { break; }
        let stick_at = STICK_HEIGHTS[i];
        let stick_point = natural_y - stick_at;
        if visual_offset > stick_point {
            let ty = visual_offset - natural_y + stick_at;
            let _ = eval(&format!(
                "var h=document.getElementById('{}');if(h){{h.style.transform='translate3d(0,{}px,0)';h.style.zIndex='99'}}",
                ids[i], ty
            ));
        } else {
            let _ = eval(&format!(
                "var h=document.getElementById('{}');if(h){{h.style.transform='';h.style.zIndex=''}}",
                ids[i]
            ));
        }
    }
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
