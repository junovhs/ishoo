use super::*;

fn ds_with_stale_springs() -> DragState {
    DragState {
        item_springs: {
            let mut m = std::collections::HashMap::new();
            m.insert(1, Spring::new(900.0, 45.0, 1.0, 30.0));
            m.insert(2, Spring::new(900.0, 45.0, 1.0, -30.0));
            m
        },
        dragging_id: Some(99),
        settling_id: Some(99),
        is_dragging: true,
        ..DragState::default()
    }
}

#[test]
fn reset_clears_item_springs_completely() {
    let mut ds = ds_with_stale_springs();
    assert!(!ds.item_springs.is_empty());
    ds.reset();
    assert!(ds.item_springs.is_empty());
    assert!(!ds.is_dragging);
    assert!(ds.dragging_id.is_none());
    assert!(ds.settling_id.is_none());
}

#[test]
fn step_settle_completes_and_returns_true() {
    let mut ds = DragState {
        settling_id: Some(1),
        ..Default::default()
    };
    // Springs already at rest
    ds.y_return.set(0.0);
    ds.x_return.set(0.0);
    ds.scale_spring.set(1.0);
    ds.scale_spring.target = 1.0;

    let done = ds.step_settle(0.016);
    assert!(done);
    assert!(ds.settling_id.is_none());
}

#[test]
fn step_settle_not_done_while_springs_active() {
    let mut ds = DragState {
        settling_id: Some(1),
        ..Default::default()
    };
    // y_return has a large displacement — not done yet
    ds.y_return.pos = 200.0;
    ds.y_return.target = 0.0;

    let done = ds.step_settle(0.016);
    assert!(!done);
    assert!(ds.settling_id.is_some());
}

#[test]
fn second_drag_starts_with_clean_state() {
    let mut ds = DragState::default();
    ds.item_springs
        .insert(7, Spring::new(900.0, 45.0, 1.0, 0.0));
    ds.settling_id = Some(1);
    ds.reset();
    assert!(ds.item_springs.is_empty());
    assert!(ds.dragging_id.is_none());
    assert!(ds.settling_id.is_none());
    assert!((ds.scale_spring.pos - 1.0).abs() < f32::EPSILON);
}

#[test]
fn slot_size_derived_from_nat_tops() {
    let ds = DragState {
        nat_tops: vec![100.0, 171.0, 242.0, 313.0],
        ..DragState::default()
    };
    assert!((ds.slot_size() - 71.0).abs() < f32::EPSILON);
}

#[test]
fn calculate_shift_uses_actual_slot_size() {
    assert!((calculate_shift(0, 2, 1, 71.0) - (-71.0)).abs() < f32::EPSILON);
    assert!((calculate_shift(0, 2, 2, 71.0) - (-71.0)).abs() < f32::EPSILON);
    assert_eq!(calculate_shift(0, 2, 3, 71.0), 0.0);
    assert!((calculate_shift(3, 1, 1, 71.0) - 71.0).abs() < f32::EPSILON);
    assert!((calculate_shift(3, 1, 2, 71.0) - 71.0).abs() < f32::EPSILON);
    assert_eq!(calculate_shift(3, 1, 0, 71.0), 0.0);
}

#[test]
fn begin_settle_carries_y_velocity_into_spring() {
    let mut ds = DragState {
        nat_tops: vec![100.0, 171.0, 242.0],
        orig_idx: 0,
        vy: 400.0,
        ..DragState::default()
    };
    // flip_delta_y = 50.0 (card is 50px above its new DOM slot)
    ds.begin_settle(50.0, 0.0);
    assert!((ds.y_return.pos - 50.0).abs() < f32::EPSILON);
    assert!(
        ds.y_return.vel > 0.0,
        "throw momentum must be positive for downward flick"
    );
}

#[test]
fn begin_settle_sets_x_return_from_dx() {
    let mut ds = DragState::default();
    ds.begin_settle(0.0, 42.0);
    assert!((ds.x_return.pos - 42.0).abs() < f32::EPSILON);
    assert_eq!(ds.x_return.target, 0.0);
}

#[test]
fn begin_settle_targets_scale_to_one() {
    let mut ds = DragState::default();
    ds.scale_spring.pos = 1.05;
    ds.scale_spring.target = 1.05;
    ds.begin_settle(0.0, 0.0);
    assert!((ds.scale_spring.target - 1.0).abs() < f32::EPSILON);
}

#[test]
fn velocity_smoothing_works() {
    let mut ds = DragState {
        cur_x: 100.0,
        cur_y: 100.0,
        ..Default::default()
    };
    ds.update_velocity(200.0, 200.0, 1.0 / 60.0);
    assert!(ds.vx > 0.0);
    assert!(ds.vy > 0.0);
    // Velocity should be smoothed, not instantaneous
    let expected_inst = (200.0 - 100.0) / (1.0 / 60.0);
    assert!(ds.vx < expected_inst);
}
