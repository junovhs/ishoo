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
    assert!(ds.pending_reorder.is_none());
}

#[test]
fn step_settle_completes_and_returns_pending_reorder() {
    let mut ds = DragState {
        settling_id: Some(1),
        pending_reorder: Some(PendingReorder {
            drag_id: 1,
            target_id: 2,
            insert_after: true,
        }),
        ..Default::default()
    };
    ds.y_return.set(0.0);
    ds.x_return.set(0.0);
    ds.scale_spring.set(1.0);

    let result = ds.step_settle(0.016);
    assert!(result.is_some());
    let reorder = result.unwrap();
    assert_eq!(reorder.drag_id, 1);
    assert_eq!(reorder.target_id, 2);
    assert!(reorder.insert_after);
    assert!(ds.settling_id.is_none());
    assert!(ds.pending_reorder.is_none());
}

#[test]
fn second_drag_starts_with_clean_state() {
    let mut ds = DragState::default();
    ds.item_springs.insert(7, Spring::new(900.0, 45.0, 1.0, 0.0));
    ds.settling_id = Some(1);
    ds.pending_reorder = Some(PendingReorder { drag_id: 1, target_id: 2, insert_after: false });
    ds.reset();
    assert!(ds.item_springs.is_empty());
    assert!(ds.dragging_id.is_none());
    assert!(ds.settling_id.is_none());
    assert!(ds.pending_reorder.is_none());
    assert!((ds.scale_spring.pos - 1.0).abs() < f32::EPSILON);
}

#[test]
fn slot_size_derived_from_nat_tops() {
    let ds = DragState { nat_tops: vec![100.0, 171.0, 242.0, 313.0], ..DragState::default() };
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
    ds.begin_settle(50.0, 0.0, 0);
    assert!((ds.y_return.pos - 50.0).abs() < f32::EPSILON);
    assert!(ds.y_return.vel > 0.0, "throw momentum must be positive for downward flick");
}

#[test]
fn velocity_smoothing_works() {
    let mut ds = DragState {
        cur_x: 100.0,
        cur_y: 100.0,
        ..Default::default()
    };

    // Simulate rapid movement
    ds.update_velocity(200.0, 200.0, 1.0 / 60.0);
    assert!(ds.vx > 0.0);
    assert!(ds.vy > 0.0);

    // Velocity should be smoothed, not instantaneous
    let expected_inst = (200.0 - 100.0) / (1.0 / 60.0);
    assert!(ds.vx < expected_inst);
}
