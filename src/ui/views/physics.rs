use std::collections::HashMap;

const FIXED_DT: f32 = 1.0 / 120.0;
const MAX_DT: f32 = 1.0 / 30.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
    pub pos: f32,
    pub target: f32,
    pub vel: f32,
}

impl Spring {
    pub fn new(stiffness: f32, damping: f32, mass: f32, initial: f32) -> Self {
        Self { stiffness, damping, mass, pos: initial, target: initial, vel: 0.0 }
    }

    pub fn step(&mut self, dt: f32) {
        let dt = dt.min(MAX_DT);
        let mut remaining = dt;
        while remaining > 1e-6 {
            let h = remaining.min(FIXED_DT);
            let x = self.pos - self.target;
            let acc = (-self.stiffness * x - self.damping * self.vel) / self.mass;
            self.vel += acc * h;
            self.pos += self.vel * h;
            remaining -= h;
        }
    }

    pub fn done(&self, threshold: f32) -> bool {
        (self.pos - self.target).abs() < threshold && self.vel.abs() < threshold
    }

    pub fn set(&mut self, v: f32) {
        self.pos = v;
        self.target = v;
        self.vel = 0.0;
    }
}

/// Pending reorder to fire after settle completes
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PendingReorder {
    pub drag_id: u32,
    pub target_id: u32,
    pub insert_after: bool,
}

#[derive(Clone)]
pub struct DragState {
    pub dragging_id: Option<u32>,
    pub settling_id: Option<u32>,
    pub is_dragging: bool,
    pub start_y: f32,
    pub start_x: f32,
    pub cur_y: f32,
    pub cur_x: f32,
    // Smoothed velocity (prototype style)
    pub vx: f32,
    pub vy: f32,
    pub orig_idx: usize,
    pub cur_idx: usize,
    pub nat_tops: Vec<f32>,
    pub layout_ids: Vec<u32>,
    // Springs tuned to match prototype exactly
    pub scale_spring: Spring,  // k=650, c=28
    pub x_return: Spring,      // k=420, c=34
    pub y_return: Spring,      // k=560, c=40
    pub item_springs: HashMap<u32, Spring>, // k=900, c=45
    /// Reorder to commit after settle animation completes
    pub pending_reorder: Option<PendingReorder>,
}

const VEL_SMOOTH: f32 = 0.35;

impl Default for DragState {
    fn default() -> Self {
        Self {
            dragging_id: None,
            settling_id: None,
            is_dragging: false,
            start_y: 0.0,
            start_x: 0.0,
            cur_y: 0.0,
            cur_x: 0.0,
            vx: 0.0,
            vy: 0.0,
            orig_idx: 0,
            cur_idx: 0,
            nat_tops: vec![],
            layout_ids: vec![],
            // Match prototype spring constants exactly
            scale_spring: Spring::new(650.0, 28.0, 1.0, 1.0),
            x_return: Spring::new(420.0, 34.0, 1.0, 0.0),
            y_return: Spring::new(560.0, 40.0, 1.0, 0.0),
            item_springs: HashMap::new(),
            pending_reorder: None,
        }
    }
}

impl DragState {
    pub fn is_active(&self) -> bool {
        self.is_dragging || self.settling_id.is_some()
    }

    pub fn reset(&mut self) {
        self.is_dragging = false;
        self.dragging_id = None;
        self.settling_id = None;
        self.item_springs.clear();
        self.scale_spring.set(1.0);
        self.x_return.set(0.0);
        self.y_return.set(0.0);
        self.vx = 0.0;
        self.vy = 0.0;
        self.pending_reorder = None;
    }

    /// Update smoothed velocity (call each frame during drag)
    pub fn update_velocity(&mut self, new_x: f32, new_y: f32, dt: f32) {
        let dt = dt.max(1.0 / 120.0);
        let inst_vx = (new_x - self.cur_x) / dt;
        let inst_vy = (new_y - self.cur_y) / dt;
        self.vx += (inst_vx - self.vx) * VEL_SMOOTH;
        self.vy += (inst_vy - self.vy) * VEL_SMOOTH;
        self.cur_x = new_x;
        self.cur_y = new_y;
    }

    pub fn step_drag(&mut self, dt: f32) {
        self.scale_spring.step(dt);
        self.update_slot_detection();
        self.update_item_springs(dt);
    }

    fn update_slot_detection(&mut self) {
        let dy = self.cur_y - self.start_y;
        let half_h = self.slot_half_height();
        let center = self.nat_tops.get(self.orig_idx).unwrap_or(&0.0) + half_h + dy;
        let mut best_idx = self.orig_idx;
        let mut best_dist = f32::INFINITY;
        for (i, &top) in self.nat_tops.iter().enumerate() {
            let dist = (center - (top + half_h)).abs();
            if dist < best_dist {
                best_dist = dist;
                best_idx = i;
            }
        }
        self.cur_idx = best_idx;
    }

    fn update_item_springs(&mut self, dt: f32) {
        let orig = self.orig_idx;
        let cur = self.cur_idx;
        let slot_size = self.slot_size();
        let ids = self.layout_ids.clone();
        for (i, &id) in ids.iter().enumerate() {
            if i == orig { continue; }
            let shift = calculate_shift(orig, cur, i, slot_size);
            let spring = self
                .item_springs
                .entry(id)
                // Match prototype: k=900, c=45
                .or_insert_with(|| Spring::new(900.0, 45.0, 1.0, 0.0));
            spring.target = shift;
            spring.step(dt);
        }
    }

    /// Returns Some(pending_reorder) when settle is fully complete
    pub fn step_settle(&mut self, dt: f32) -> Option<PendingReorder> {
        self.y_return.step(dt);
        self.x_return.step(dt);
        self.scale_spring.step(dt);

        let mut all_done = self.y_return.done(0.25)
            && self.x_return.done(0.25)
            && self.scale_spring.done(0.002);

        for spring in self.item_springs.values_mut() {
            spring.step(dt);
            if !spring.done(0.25) {
                all_done = false;
            }
        }

        if all_done {
            let reorder = self.pending_reorder.take();
            self.settling_id = None;
            self.item_springs.clear();
            return reorder;
        }

        None
    }

    pub fn slot_size(&self) -> f32 {
        if self.nat_tops.len() < 2 { return 62.0; }
        self.nat_tops[1] - self.nat_tops[0]
    }

    fn slot_half_height(&self) -> f32 {
        self.slot_size() * 0.4
    }

    /// Prepare settle springs after pointer-up.
    /// Carries release velocity into the spring for momentum.
    pub fn begin_settle(&mut self, dy: f32, dx: f32, new_idx: usize) {
        let old_top = self.nat_tops.get(self.orig_idx).copied().unwrap_or(0.0);
        let new_top = self.nat_tops.get(new_idx).copied().unwrap_or(old_top);

        self.y_return.pos = old_top + dy - new_top;
        self.y_return.target = 0.0;
        self.y_return.vel = self.vy * 0.3;

        self.x_return.pos = dx;
        self.x_return.target = 0.0;
        self.x_return.vel = self.vx * 0.15;

        self.scale_spring.target = 1.0;

        for spring in self.item_springs.values_mut() {
            spring.target = 0.0;
        }

        self.is_dragging = false;
    }
}

pub fn calculate_shift(orig: usize, cur: usize, i: usize, slot_size: f32) -> f32 {
    if orig < cur && i > orig && i <= cur { -slot_size }
    else if orig > cur && i >= cur && i < orig { slot_size }
    else { 0.0 }
}

#[cfg(test)]
mod tests;
