use super::*;

#[test]
fn anim_snaps_to_value() {
    let mut a = Anim::with(500.0, 35.0, 50.0);
    assert!((a.val - 50.0).abs() < f32::EPSILON);
    assert!((a.goal - 50.0).abs() < f32::EPSILON);
    a.snap(100.0);
    assert!((a.val - 100.0).abs() < f32::EPSILON);
    assert!(a.v.abs() < f32::EPSILON);
}

#[test]
fn anim_settles_at_goal() {
    let mut a = Anim::with(500.0, 35.0, 0.0);
    a.val = 200.0;
    a.goal = 0.0;
    for _ in 0..500 {
        a.tick(1.0 / 120.0);
    }
    assert!(a.settled(0.5));
}

#[test]
fn grab_wipe_clears_everything() {
    let mut g = Grab {
        held: Some(42),
        releasing: Some(42),
        confirmed_drag: true,
        offsets: vec![1, -1, 0],
        ..Default::default()
    };
    g.wipe();
    assert!(g.held.is_none());
    assert!(g.releasing.is_none());
    assert!(!g.confirmed_drag);
    assert!(g.offsets.is_empty());
}

#[test]
fn drag_tick_computes_hover_slot() {
    let mut g = Grab {
        held: Some(1),
        confirmed_drag: true,
        slot_h: 63.0,
        ids: vec![10, 20, 30, 40],
        offsets: vec![0; 4],
        home: 0,
        anchor_y: 100.0,
        ptr_y: 100.0 + 63.0 * 2.0,
        ..Default::default()
    };
    g.drag_tick(0.016);
    assert_eq!(g.hover, 2);
}

#[test]
fn offsets_shift_cards_between_home_and_hover() {
    let mut g = Grab {
        held: Some(1),
        confirmed_drag: true,
        slot_h: 63.0,
        ids: vec![10, 20, 30, 40],
        offsets: vec![0; 4],
        home: 0,
        anchor_y: 100.0,
        ptr_y: 100.0 + 63.0 * 2.0,
        ..Default::default()
    };
    g.drag_tick(0.016);
    assert_eq!(g.offset_for(0), 0);
    assert_eq!(g.offset_for(1), -1);
    assert_eq!(g.offset_for(2), -1);
    assert_eq!(g.offset_for(3), 0);
}

#[test]
fn release_clears_offsets_and_sets_releasing() {
    let mut g = Grab {
        held: Some(5),
        confirmed_drag: true,
        slot_h: 63.0,
        ids: vec![5, 6, 7],
        offsets: vec![0, -1, 0],
        home: 0,
        hover: 1,
        anchor_y: 0.0,
        ptr_y: 63.0,
        ..Default::default()
    };
    let result = g.release();
    assert!(result.is_some());
    let (id, from, to) = result.unwrap();
    assert_eq!(id, 5);
    assert_eq!(from, 0);
    assert_eq!(to, 1);
    assert!(g.held.is_none());
    assert_eq!(g.releasing, Some(5));
    assert!(g.offsets.is_empty());
}

#[test]
fn release_no_move_returns_none() {
    let mut g = Grab {
        held: Some(5),
        confirmed_drag: true,
        home: 1,
        hover: 1,
        anchor_y: 0.0,
        ptr_y: 0.0,
        ..Default::default()
    };
    let result = g.release();
    assert!(result.is_none());
    assert_eq!(g.releasing, Some(5));
}

#[test]
fn settle_completes_when_springs_done() {
    let mut g = Grab {
        releasing: Some(1),
        ..Default::default()
    };
    g.fly_y.snap(0.0);
    g.fly_x.snap(0.0);
    g.pop.snap(1.0);
    let done = g.settle_tick(0.016);
    assert!(done);
    assert!(g.releasing.is_none());
}

#[test]
fn settle_not_done_while_flying() {
    let mut g = Grab {
        releasing: Some(1),
        ..Default::default()
    };
    g.fly_y.val = 200.0;
    g.fly_y.goal = 0.0;
    let done = g.settle_tick(0.016);
    assert!(!done);
    assert!(g.releasing.is_some());
}

#[test]
fn velocity_tracking() {
    let mut g = Grab {
        ptr_x: 100.0,
        ptr_y: 100.0,
        ..Default::default()
    };
    g.track(200.0, 200.0);
    assert!(g.vx > 0.0);
    assert!(g.vy > 0.0);
}