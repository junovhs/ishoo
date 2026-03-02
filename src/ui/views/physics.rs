use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    stiffness: f32,
    damping: f32,
    mass: f32,
    pub pos: f32,
    pub target: f32,
    vel: f32,
}

impl Spring {
    pub fn new(stiffness: f32, damping: f32, mass: f32, initial: f32) -> Self {
        Self {
            stiffness,
            damping,
            mass,
            pos: initial,
            target: initial,
            vel: 0.0,
        }
    }

    pub fn step(&mut self, dt: f32) {
        let dt = dt.min(0.033);
        let x = self.pos - self.target;
        self.vel += ((-self.stiffness * x + -self.damping * self.vel) / self.mass) * dt;
        self.pos += self.vel * dt;
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

// neti:allow(AHF) — public fields are intentional for direct state access from UI
#[derive(Clone)]
pub struct DragState {
    pub dragging_id: Option<u32>,
    pub settling_id: Option<u32>,
    pub is_dragging: bool,
    pub start_y: f32,
    pub start_x: f32,
    pub cur_y: f32,
    pub cur_x: f32,
    pub prev_x: f32,
    pub x_vel: f32,

    pub orig_idx: usize,
    pub cur_idx: usize,
    pub nat_tops: Vec<f32>,
    pub layout_ids: Vec<u32>,

    pub rot_spring: Spring,
    pub scale_spring: Spring,
    pub x_return: Spring,
    pub y_return: Spring,
    pub item_springs: HashMap<u32, Spring>,
}

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
            prev_x: 0.0,
            x_vel: 0.0,
            orig_idx: 0,
            cur_idx: 0,
            nat_tops: vec![],
            layout_ids: vec![],
            rot_spring: Spring::new(150.0, 12.0, 0.8, 0.0),
            scale_spring: Spring::new(600.0, 25.0, 1.0, 1.0),
            x_return: Spring::new(400.0, 30.0, 1.0, 0.0),
            y_return: Spring::new(500.0, 35.0, 1.0, 0.0),
            item_springs: HashMap::new(),
        }
    }
}

impl DragState {
    pub fn is_active(&self) -> bool {
        self.is_dragging || self.settling_id.is_some()
    }

    pub fn step_drag(&mut self, dt: f32) {
        self.update_velocity(dt);
        self.update_rotation(dt);
        self.update_slot_detection();
        self.update_item_springs(dt);
    }

    fn update_velocity(&mut self, dt: f32) {
        self.x_vel = (self.cur_x - self.prev_x) / dt.max(1.0 / 120.0);
        self.prev_x = self.cur_x;
    }

    fn update_rotation(&mut self, dt: f32) {
        let vel = self.x_vel;
        self.rot_spring.target = (vel.clamp(-1000.0, 1000.0) / 1000.0) * -15.0;
        self.rot_spring.step(dt);
        self.scale_spring.step(dt);
    }

    fn update_slot_detection(&mut self) {
        let dy = self.cur_y - self.start_y;
        let center = self.nat_tops.get(self.orig_idx).unwrap_or(&0.0) + 31.0 + dy;
        let mut best_idx = self.orig_idx;
        let mut best_dist = f32::INFINITY;

        for (i, &top) in self.nat_tops.iter().enumerate() {
            let dist = (center - (top + 31.0)).abs();
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
        let ids = self.layout_ids.clone();

        for (i, &id) in ids.iter().enumerate() {
            if i == orig {
                continue;
            }
            let shift = calculate_shift(orig, cur, i);
            let spring = self
                .item_springs
                .entry(id)
                .or_insert_with(|| Spring::new(800.0, 40.0, 1.0, 0.0));
            spring.target = shift;
            spring.step(dt);
        }
    }

    pub fn step_settle(&mut self, dt: f32) -> bool {
        self.y_return.step(dt);
        self.x_return.step(dt);
        self.scale_spring.step(dt);
        self.rot_spring.target = 0.0;
        self.rot_spring.step(dt);

        let mut done =
            self.y_return.done(0.5) && self.x_return.done(0.5) && self.scale_spring.done(0.01);

        for spring in self.item_springs.values_mut() {
            spring.step(dt);
            if !spring.done(0.5) {
                done = false;
            }
        }

        if done {
            self.settling_id = None;
            self.item_springs.clear();
        }

        done
    }
}

fn calculate_shift(orig: usize, cur: usize, i: usize) -> f32 {
    if orig < cur && i > orig && i <= cur {
        -62.0
    } else if orig > cur && i >= cur && i < orig {
        62.0
    } else {
        0.0
    }
}
