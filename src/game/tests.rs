use super::*;

fn empty_game() -> Game {
    let mut g = Game::new(7);
    g.bricks = vec![None; COLS * ROWS];
    g.events.clear();
    g
}

fn free_ball(g: &mut Game, pos: Vec2, dir: Vec2) {
    g.balls = vec![Ball {
        pos,
        dir: dir.normalize(),
        stuck: None,
        stuck_time: 0.0,
    }];
}

fn run(g: &mut Game, seconds: f32) {
    let steps = (seconds / FIXED_DT) as usize;
    for _ in 0..steps {
        g.step(FIXED_DT);
    }
}

/// A distant brick keeps the level from counting as cleared.
fn add_anchor(g: &mut Game) {
    g.bricks[cell_index(0, ROWS - 1)] = Some(Brick::new(BrickKind::Iron));
    g.bricks[cell_index(15, 0)] = Some(Brick::new(BrickKind::Stone(0)));
}

#[test]
fn paddle_centre_sends_ball_straight_up() {
    let d = paddle_bounce_dir(0.0, 16.0);
    assert!(d.x.abs() < 1e-6);
    assert!((d.y + 1.0).abs() < 1e-6);
}

#[test]
fn paddle_edges_give_steep_mirrored_angles() {
    let l = paddle_bounce_dir(-20.0, 16.0);
    let r = paddle_bounce_dir(20.0, 16.0);
    assert!(l.x < -0.85 && r.x > 0.85);
    assert!((l.x + r.x).abs() < 1e-6);
    assert!(l.y < 0.0 && r.y < 0.0);
    // Offset is monotonic: further from centre, flatter angle.
    let a = paddle_bounce_dir(4.0, 16.0);
    let b = paddle_bounce_dir(10.0, 16.0);
    assert!(b.x > a.x);
}

#[test]
fn clamp_angle_prevents_flat_travel() {
    let d = clamp_angle(vec2(1.0, 0.0));
    let min = MIN_ANGLE_DEG.to_radians().sin();
    assert!((d.y.abs() - min).abs() < 1e-5);
    assert!((d.length() - 1.0).abs() < 1e-5);
    let d = clamp_angle(vec2(-1.0, 0.01));
    assert!(d.x < 0.0 && d.y > 0.0);
    // Steep vectors are untouched.
    let s = vec2(0.6, -0.8);
    assert!((clamp_angle(s) - s).length() < 1e-6);
}

#[test]
fn ball_reflects_off_side_and_top_walls() {
    let mut g = empty_game();
    add_anchor(&mut g);
    free_ball(&mut g, vec2(5.0, 100.0), vec2(-1.0, -1.0));
    run(&mut g, 0.1);
    assert!(g.balls[0].dir.x > 0.0, "should bounce off the left wall");
    free_ball(&mut g, vec2(150.0, 6.0), vec2(0.5, -1.0));
    run(&mut g, 0.1);
    assert!(g.balls[0].dir.y > 0.0, "should bounce off the ceiling");
    free_ball(&mut g, vec2(PF_W - 4.0, 100.0), vec2(1.0, 0.5));
    run(&mut g, 0.1);
    assert!(g.balls[0].dir.x < 0.0, "should bounce off the right wall");
}

#[test]
fn ball_bounces_off_paddle_using_strike_offset() {
    let mut g = empty_game();
    add_anchor(&mut g);
    g.paddle.x = 100.0;
    free_ball(&mut g, vec2(110.0, PADDLE_Y - 20.0), vec2(0.0, 1.0));
    run(&mut g, 0.3);
    let b = g.balls[0];
    assert!(b.dir.y < 0.0, "ball should be heading up");
    assert!(
        b.dir.x > 0.3,
        "right-of-centre hit should send it right: {:?}",
        b.dir
    );
    assert!(
        g.events
            .iter()
            .any(|e| matches!(e, Event::PaddleHit { .. }))
    );
}

#[test]
fn ball_missing_paddle_is_lost_and_costs_a_life() {
    let mut g = empty_game();
    add_anchor(&mut g);
    g.paddle.x = 30.0;
    free_ball(&mut g, vec2(250.0, PADDLE_Y - 10.0), vec2(0.0, 1.0));
    let lives = g.lives;
    run(&mut g, 0.5);
    assert!(matches!(g.phase, Phase::Dying(_)));
    run(&mut g, 2.0);
    assert_eq!(g.phase, Phase::Play);
    assert_eq!(g.lives, lives - 1);
    assert_eq!(g.balls.len(), 1);
    assert!(
        g.balls[0].stuck.is_some(),
        "new ball is served on the paddle"
    );
}

#[test]
fn game_over_when_last_life_lost() {
    let mut g = empty_game();
    add_anchor(&mut g);
    g.lives = 1;
    g.paddle.x = 30.0;
    free_ball(&mut g, vec2(250.0, PADDLE_Y), vec2(0.0, 1.0));
    run(&mut g, 3.0);
    assert_eq!(g.phase, Phase::GameOver);
    assert!(g.events.contains(&Event::GameOver));
}

#[test]
fn hitting_brick_from_below_reflects_vertically_and_breaks_it() {
    let mut g = empty_game();
    add_anchor(&mut g);
    let idx = cell_index(5, 4);
    g.bricks[idx] = Some(Brick::new(BrickKind::Stone(0)));
    let c = cell_center(idx);
    free_ball(&mut g, vec2(c.x, c.y + 20.0), vec2(0.2, -1.0));
    let dx_before = g.balls[0].dir.x;
    run(&mut g, 0.3);
    assert!(g.bricks[idx].is_none(), "brick should be destroyed");
    assert!(g.balls[0].dir.y > 0.0, "ball should now travel down");
    assert!(
        (g.balls[0].dir.x - dx_before).abs() < 1e-4,
        "horizontal component kept"
    );
    assert_eq!(g.score, BrickKind::Stone(0).points());
}

#[test]
fn hitting_brick_side_reflects_horizontally() {
    let mut g = empty_game();
    add_anchor(&mut g);
    let idx = cell_index(8, 6);
    g.bricks[idx] = Some(Brick::new(BrickKind::Iron));
    let (l, t, _, b) = cell_rect(idx);
    free_ball(&mut g, vec2(l - 15.0, (t + b) * 0.5), vec2(1.0, 0.3));
    run(&mut g, 0.15);
    assert!(g.balls[0].dir.x < 0.0, "should bounce back to the left");
    assert!(g.bricks[idx].is_some(), "iron is indestructible");
}

#[test]
fn row_of_bricks_acts_as_flat_surface() {
    // Striking the seam between two bricks must not produce a corner deflection.
    let mut g = empty_game();
    add_anchor(&mut g);
    let a = cell_index(4, 5);
    let b = cell_index(5, 5);
    g.bricks[a] = Some(Brick::new(BrickKind::Iron));
    g.bricks[b] = Some(Brick::new(BrickKind::Iron));
    let (_, _, seam_x, bottom) = cell_rect(a);
    let dir = vec2(0.5, -1.0).normalize();
    free_ball(&mut g, vec2(seam_x - 4.0, bottom + 8.0), dir);
    run(&mut g, 0.12);
    let d = g.balls[0].dir;
    assert!(d.y > 0.0);
    assert!(
        (d.x - dir.x).abs() < 1e-4,
        "x must be preserved on a flat face, got {d:?}"
    );
}

#[test]
fn multi_hit_bricks_need_several_hits() {
    let mut g = empty_game();
    add_anchor(&mut g);
    let idx = cell_index(3, 3);
    g.bricks[idx] = Some(Brick::new(BrickKind::Crystal));
    g.hit_brick(idx);
    g.hit_brick(idx);
    assert_eq!(g.bricks[idx].unwrap().hp, 1);
    g.hit_brick(idx);
    assert!(g.bricks[idx].is_none());
}

#[test]
fn rune_explosion_chains_but_spares_iron() {
    let mut g = empty_game();
    add_anchor(&mut g);
    let r1 = cell_index(5, 5);
    let r2 = cell_index(6, 5);
    let stone = cell_index(7, 6);
    let iron = cell_index(4, 4);
    g.bricks[r1] = Some(Brick::new(BrickKind::Rune));
    g.bricks[r2] = Some(Brick::new(BrickKind::Rune));
    g.bricks[stone] = Some(Brick::new(BrickKind::Crystal));
    g.bricks[iron] = Some(Brick::new(BrickKind::Iron));
    g.balls.clear();
    g.balls.push(Ball {
        pos: vec2(10.0, 150.0),
        dir: vec2(0.0, -1.0),
        stuck: Some(0.0),
        stuck_time: 0.0,
    });
    g.destroy_brick(r1);
    run(&mut g, 0.5);
    assert!(g.bricks[r2].is_none(), "neighbouring rune detonates");
    assert!(
        g.bricks[stone].is_none(),
        "chain reaches the second rune's neighbours"
    );
    assert!(g.bricks[iron].is_some(), "iron survives explosions");
}

#[test]
fn fireball_destroys_neighbours() {
    let mut g = empty_game();
    add_anchor(&mut g);
    for c in 4..=6 {
        for r in 4..=6 {
            g.bricks[cell_index(c, r)] = Some(Brick::new(BrickKind::Bound));
        }
    }
    g.effects.fire = 5.0;
    let c = cell_center(cell_index(5, 6));
    free_ball(&mut g, vec2(c.x, c.y + 12.0), vec2(0.0, -1.0));
    run(&mut g, 0.1);
    assert!(g.bricks[cell_index(5, 5)].is_none());
    assert!(g.bricks[cell_index(4, 5)].is_none());
    assert!(g.balls[0].dir.y > 0.0, "fireball still bounces");
}

#[test]
fn phantom_ball_passes_through_bricks() {
    let mut g = empty_game();
    add_anchor(&mut g);
    for r in 3..8 {
        g.bricks[cell_index(5, r)] = Some(Brick::new(BrickKind::Crystal));
    }
    g.effects.phantom = 5.0;
    let c = cell_center(cell_index(5, 7));
    free_ball(&mut g, vec2(c.x, c.y + 10.0), vec2(0.0, -1.0));
    run(&mut g, 0.35);
    for r in 3..8 {
        assert!(g.bricks[cell_index(5, r)].is_none());
    }
    assert!(
        g.balls[0].dir.y < 0.0 || g.balls[0].pos.y < 20.0,
        "kept going up"
    );
}

#[test]
fn no_tunnelling_at_max_speed() {
    let mut g = empty_game();
    add_anchor(&mut g);
    for c in 0..COLS {
        g.bricks[cell_index(c, 8)] = Some(Brick::new(BrickKind::Iron));
    }
    g.speed = SPEED_MAX;
    free_ball(&mut g, vec2(100.0, 160.0), vec2(0.3, -1.0));
    let wall_y = cell_rect(cell_index(0, 8)).1;
    for _ in 0..2000 {
        g.speed = SPEED_MAX;
        g.paddle.x = g.balls[0].pos.x;
        g.step(FIXED_DT);
        assert!(g.balls[0].pos.y > wall_y, "ball tunnelled through iron");
    }
}

#[test]
fn grab_catches_and_releases_ball() {
    let mut g = empty_game();
    add_anchor(&mut g);
    g.effects.grab = true;
    g.paddle.x = 100.0;
    free_ball(&mut g, vec2(104.0, PADDLE_Y - 15.0), vec2(0.0, 1.0));
    run(&mut g, 0.2);
    assert!(g.balls[0].stuck.is_some());
    g.update(
        0.0,
        &Input {
            fire: true,
            ..Default::default()
        },
    );
    assert!(g.balls[0].stuck.is_none());
    assert!(g.balls[0].dir.y < 0.0);
}

#[test]
fn split_and_legion_multiply_balls() {
    let mut g = empty_game();
    add_anchor(&mut g);
    free_ball(&mut g, vec2(100.0, 100.0), vec2(0.3, -1.0));
    g.apply_power(Power::Split, vec2(0.0, 0.0));
    assert_eq!(g.balls.len(), 3);
    g.apply_power(Power::Legion, vec2(0.0, 0.0));
    assert_eq!(g.balls.len(), 10);
    for b in &g.balls {
        assert!((b.dir.length() - 1.0).abs() < 1e-4);
    }
}

#[test]
fn paddle_size_powers_are_bounded() {
    let mut g = empty_game();
    for _ in 0..10 {
        g.apply_power(Power::Expand, Vec2::ZERO);
    }
    assert_eq!(g.paddle.size, PADDLE_SIZES.len() - 1);
    for _ in 0..10 {
        g.apply_power(Power::Shrink, Vec2::ZERO);
    }
    assert_eq!(g.paddle.size, 0);
}

#[test]
fn drops_are_caught_by_paddle() {
    let mut g = empty_game();
    add_anchor(&mut g);
    g.paddle.x = 50.0;
    g.drops.push(Drop {
        power: Power::Life,
        pos: vec2(52.0, 120.0),
        age: 0.0,
    });
    let lives = g.lives;
    run(&mut g, 2.0);
    assert_eq!(g.lives, lives + 1);
    assert!(g.drops.is_empty());
}

#[test]
fn quake_drops_columns_by_one_row() {
    let mut g = empty_game();
    g.bricks[cell_index(2, 0)] = Some(Brick::new(BrickKind::Stone(1)));
    g.bricks[cell_index(2, 1)] = Some(Brick::new(BrickKind::Stone(2)));
    g.quake();
    assert!(g.bricks[cell_index(2, 0)].is_none());
    assert_eq!(
        g.bricks[cell_index(2, 1)].unwrap().kind,
        BrickKind::Stone(1)
    );
    assert_eq!(
        g.bricks[cell_index(2, 2)].unwrap().kind,
        BrickKind::Stone(2)
    );
}

#[test]
fn clearing_all_breakable_bricks_advances_level() {
    let mut g = Game::new(3);
    for (i, b) in g.bricks.iter_mut().enumerate() {
        if b.is_some_and(|b| b.kind.breakable()) && i != 40 {
            *b = None;
        }
    }
    g.bricks[40] = Some(Brick::new(BrickKind::Stone(0)));
    g.release_balls();
    g.destroy_brick(40);
    run(&mut g, 0.05);
    assert!(matches!(g.phase, Phase::Cleared(_)));
    run(&mut g, 3.0);
    assert_eq!(g.level, 1);
    assert_eq!(g.phase, Phase::Play);
}

#[test]
fn speed_ramps_up_while_playing() {
    let mut g = empty_game();
    add_anchor(&mut g);
    free_ball(&mut g, vec2(100.0, 100.0), vec2(0.3, -1.0));
    let s0 = g.speed;
    for _ in 0..(10.0 / FIXED_DT) as usize {
        g.paddle.x = g.balls[0].pos.x;
        g.step(FIXED_DT);
    }
    assert!(g.speed > s0 + 5.0, "speed {s0} -> {}", g.speed);
    assert!(g.speed <= SPEED_MAX);
}

#[test]
fn crossbow_bolts_break_bricks() {
    let mut g = empty_game();
    add_anchor(&mut g);
    g.paddle.x = 100.0;
    let hw = g.paddle.half();
    let idx_l = cell_index(((100.0 - hw + 3.0) / BRICK_W) as usize, 5);
    g.bricks[idx_l] = Some(Brick::new(BrickKind::Stone(0)));
    free_ball(&mut g, vec2(250.0, 60.0), vec2(0.0, -1.0));
    g.effects.crossbow = 5.0;
    g.update(
        FIXED_DT,
        &Input {
            fire: true,
            fire_held: true,
            ..Default::default()
        },
    );
    assert_eq!(g.bolts.len(), 2);
    run(&mut g, 1.0);
    assert!(g.bricks[idx_l].is_none());
}

/// Autopilot plays many levels: nothing panics, balls never escape the playfield
/// sideways or upward, and levels do get cleared.
#[test]
fn autopilot_soak_test() {
    let mut g = Game::new(12345);
    let mut levels_cleared = 0;
    let mut t = 0.0;
    while t < 900.0 {
        let target = g
            .balls
            .iter()
            .filter(|b| b.stuck.is_none())
            .max_by(|a, b| a.pos.y.total_cmp(&b.pos.y))
            .map(|b| b.pos.x - 3.0 * b.dir.x.signum());
        let input = Input {
            paddle_x: target,
            fire: true,
            fire_held: true,
            ..Default::default()
        };
        g.update(1.0 / 60.0, &input);
        for e in g.take_events() {
            if e == Event::LevelCleared {
                levels_cleared += 1;
            }
        }
        for b in &g.balls {
            assert!(
                b.pos.x >= 0.0 && b.pos.x <= PF_W && b.pos.y >= 0.0,
                "ball escaped: {b:?}"
            );
            assert!(b.pos.is_finite() && b.dir.is_finite());
        }
        match g.phase {
            Phase::GameOver => {
                g = Game::new(t as u64);
            }
            Phase::Victory => g.continue_cycle(),
            _ => {}
        }
        t += 1.0 / 60.0;
    }
    assert!(
        levels_cleared >= 2,
        "autopilot only cleared {levels_cleared} levels"
    );
}
