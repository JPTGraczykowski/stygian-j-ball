//! Game rules and physics: a faithful take on DX-Ball's ball/paddle/brick behaviour.
//!
//! * The paddle follows the mouse directly; the ball's rebound angle depends only on
//!   where it strikes the paddle (centre = straight up, edges = steep angles).
//! * The ball moves at one shared speed that slowly creeps upward during play.
//! * Bricks reflect the ball off the face that was struck; adjacent bricks behave as one
//!   flat surface. Corners deflect along the corner normal.
//! * Power-ups fall from broken bricks and are caught with the paddle.
//!
//! This module is pure logic (no rendering, no audio) so it can be unit tested.

use crate::consts::*;
use crate::levels::LEVELS;
use crate::rng::Rng;
use macroquad::math::{Vec2, vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrickKind {
    /// Plain one-hit stone; the value picks the material (0..6).
    Stone(u8),
    /// Iron-bound wood, two hits.
    Bound,
    /// Crystal, three hits.
    Crystal,
    /// Indestructible iron.
    Iron,
    /// Fire rune: explodes, destroying its neighbours.
    Rune,
    /// Treasure: always drops a boon.
    Gold,
}

impl BrickKind {
    pub fn from_char(c: char) -> Option<BrickKind> {
        Some(match c {
            '1'..='6' => BrickKind::Stone(c as u8 - b'1'),
            'B' => BrickKind::Bound,
            'C' => BrickKind::Crystal,
            'I' => BrickKind::Iron,
            'R' => BrickKind::Rune,
            'G' => BrickKind::Gold,
            _ => return None,
        })
    }

    pub fn max_hp(self) -> u8 {
        match self {
            BrickKind::Bound => 2,
            BrickKind::Crystal => 3,
            _ => 1,
        }
    }

    pub fn points(self) -> u64 {
        match self {
            BrickKind::Stone(_) => 10,
            BrickKind::Bound => 25,
            BrickKind::Crystal => 40,
            BrickKind::Iron => 0,
            BrickKind::Rune => 20,
            BrickKind::Gold => 100,
        }
    }

    pub fn breakable(self) -> bool {
        self != BrickKind::Iron
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Brick {
    pub kind: BrickKind,
    pub hp: u8,
    /// Seconds since last hit, for the hit flash.
    pub flash: f32,
}

impl Brick {
    pub fn new(kind: BrickKind) -> Brick {
        Brick {
            kind,
            hp: kind.max_hp(),
            flash: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Power {
    Expand,
    Shrink,
    Split,
    Legion,
    Fire,
    Phantom,
    Haste,
    Slow,
    Grab,
    Crossbow,
    Life,
    Skull,
    Quake,
    Zap,
    Warp,
}

impl Power {
    pub const ALL: [Power; 15] = [
        Power::Expand,
        Power::Shrink,
        Power::Split,
        Power::Legion,
        Power::Fire,
        Power::Phantom,
        Power::Haste,
        Power::Slow,
        Power::Grab,
        Power::Crossbow,
        Power::Life,
        Power::Skull,
        Power::Quake,
        Power::Zap,
        Power::Warp,
    ];

    /// Relative drop frequency.
    pub fn weight(self) -> u32 {
        match self {
            Power::Expand => 10,
            Power::Shrink => 6,
            Power::Split => 9,
            Power::Legion => 3,
            Power::Fire => 5,
            Power::Phantom => 3,
            Power::Haste => 6,
            Power::Slow => 6,
            Power::Grab => 5,
            Power::Crossbow => 6,
            Power::Life => 2,
            Power::Skull => 3,
            Power::Quake => 3,
            Power::Zap => 4,
            Power::Warp => 1,
        }
    }

    pub fn good(self) -> bool {
        !matches!(
            self,
            Power::Shrink | Power::Haste | Power::Skull | Power::Quake
        )
    }

    pub fn name(self) -> &'static str {
        match self {
            Power::Expand => "Potion of Girth",
            Power::Shrink => "Potion of Withering",
            Power::Split => "Wand of Splitting",
            Power::Legion => "Scroll of Legion",
            Power::Fire => "Rune of Flame",
            Power::Phantom => "Rune of Passage",
            Power::Haste => "Potion of Haste",
            Power::Slow => "Hourglass of Ages",
            Power::Grab => "Pitch of Holding",
            Power::Crossbow => "Crossbow",
            Power::Life => "Ankh of Life",
            Power::Skull => "Skull of Doom",
            Power::Quake => "Rune of Tremors",
            Power::Zap => "Rune of Frailty",
            Power::Warp => "Moongate",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Power::Expand => "You quaff a Potion of Girth. Your plank grows!",
            Power::Shrink => "A Potion of Withering! Your plank shrivels.",
            Power::Split => "The Wand of Splitting cleaves the orb in three!",
            Power::Legion => "You read the Scroll of Legion. Eight orbs!",
            Power::Fire => "The Rune of Flame sets the orb ablaze!",
            Power::Phantom => "Rune of Passage: the orb walks through stone.",
            Power::Haste => "A Potion of Haste! The orb quickens.",
            Power::Slow => "The Hourglass of Ages slows the orb.",
            Power::Grab => "Pitch of Holding: your plank turns sticky.",
            Power::Crossbow => "You take up a Crossbow. Click to loose bolts!",
            Power::Life => "An Ankh of Life! You gain another chance.",
            Power::Skull => "The Skull of Doom! Darkness takes you...",
            Power::Quake => "The Rune of Tremors shakes the walls down!",
            Power::Zap => "Rune of Frailty: every wall weakens.",
            Power::Warp => "A Moongate opens! You step through...",
        }
    }

    fn pick(rng: &mut Rng, good_only: bool) -> Power {
        let pool = Power::ALL
            .iter()
            .filter(|p| !good_only || (p.good() && **p != Power::Warp));
        let total: u32 = pool.clone().map(|p| p.weight()).sum();
        let mut roll = rng.below(total);
        for p in pool {
            if roll < p.weight() {
                return *p;
            }
            roll -= p.weight();
        }
        Power::Expand
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ball {
    pub pos: Vec2,
    /// Unit direction; actual velocity is `dir * Game::speed`.
    pub dir: Vec2,
    /// When stuck to the paddle: horizontal offset from the paddle centre.
    pub stuck: Option<f32>,
    pub stuck_time: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Paddle {
    /// Centre x.
    pub x: f32,
    pub size: usize,
    /// Current (animated) width; eases toward `PADDLE_SIZES[size]`.
    pub width: f32,
}

impl Paddle {
    pub fn half(&self) -> f32 {
        self.width * 0.5
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Drop {
    pub power: Power,
    pub pos: Vec2,
    pub age: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bolt {
    pub pos: Vec2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Effects {
    pub fire: f32,
    pub phantom: f32,
    pub crossbow: f32,
    pub grab: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Play,
    /// Level cleared; counts down to the next level.
    Cleared(f32),
    /// Last ball lost; counts down to the next serve.
    Dying(f32),
    GameOver,
    Victory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sfx {
    Paddle,
    Wall,
    BrickHit,
    BrickBreak,
    Clang,
    Explosion,
    PowerGood,
    PowerBad,
    Bolt,
    Launch,
    LifeLost,
    LevelClear,
    GameOver,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Sfx(Sfx),
    Message(String),
    BrickHit { pos: Vec2, kind: BrickKind },
    BrickBroken { pos: Vec2, kind: BrickKind },
    Explosion { pos: Vec2, big: bool },
    PaddleHit { pos: Vec2 },
    PowerCollected { pos: Vec2, power: Power },
    BallLost { pos: Vec2 },
    Shake(f32),
    LevelStart,
    LevelCleared,
    GameOver,
    Victory,
}

/// Per-frame player input, in playfield coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Input {
    /// Absolute paddle target from the mouse, if the mouse moved.
    pub paddle_x: Option<f32>,
    pub left: bool,
    pub right: bool,
    /// Fire pressed this frame (launch / shoot).
    pub fire: bool,
    /// Fire held (crossbow autofire).
    pub fire_held: bool,
}

#[derive(Clone, Debug)]
pub struct Game {
    pub level: usize,
    /// Completed passes through all levels; each pass is faster.
    pub cycle: u32,
    pub bricks: Vec<Option<Brick>>,
    pub balls: Vec<Ball>,
    pub paddle: Paddle,
    pub drops: Vec<Drop>,
    pub bolts: Vec<Bolt>,
    pub score: u64,
    pub lives: u32,
    pub speed: f32,
    pub effects: Effects,
    pub phase: Phase,
    pub events: Vec<Event>,
    pub time: f32,
    pub level_time: f32,
    pub rng: Rng,
    pending_explosions: Vec<(usize, f32)>,
    since_progress: f32,
    bolt_cooldown: f32,
    accumulator: f32,
    /// Keyboard paddle target, kept between frames.
    key_x: Option<f32>,
}

pub fn cell_index(col: usize, row: usize) -> usize {
    row * COLS + col
}

pub fn cell_rect(idx: usize) -> (f32, f32, f32, f32) {
    let col = idx % COLS;
    let row = idx / COLS;
    let x = col as f32 * BRICK_W;
    let y = BRICK_TOP + row as f32 * BRICK_H;
    (x, y, x + BRICK_W, y + BRICK_H)
}

pub fn cell_center(idx: usize) -> Vec2 {
    let (l, t, r, b) = cell_rect(idx);
    vec2((l + r) * 0.5, (t + b) * 0.5)
}

/// Direction leaving the paddle given the strike offset from its centre.
pub fn paddle_bounce_dir(offset: f32, half_width: f32) -> Vec2 {
    let t = (offset / (half_width + BALL_R * 0.5)).clamp(-1.0, 1.0);
    let a = (t * MAX_BOUNCE_DEG).to_radians();
    vec2(a.sin(), -a.cos())
}

/// Keep the ball from travelling too flat (endless side-to-side bouncing).
pub fn clamp_angle(dir: Vec2) -> Vec2 {
    let d = dir.normalize_or(vec2(0.0, -1.0));
    let min = MIN_ANGLE_DEG.to_radians().sin();
    if d.y.abs() >= min {
        return d;
    }
    let sy = if d.y < 0.0 {
        -1.0
    } else if d.y > 0.0 {
        1.0
    } else {
        -1.0
    };
    let sx = if d.x < 0.0 { -1.0 } else { 1.0 };
    vec2(sx * (1.0 - min * min).sqrt(), sy * min)
}

fn rotate(v: Vec2, deg: f32) -> Vec2 {
    let (s, c) = deg.to_radians().sin_cos();
    vec2(v.x * c - v.y * s, v.x * s + v.y * c)
}

fn circle_hits_rect(p: Vec2, r: f32, (l, t, rr, b): (f32, f32, f32, f32)) -> bool {
    let cx = p.x.clamp(l, rr);
    let cy = p.y.clamp(t, b);
    let dx = p.x - cx;
    let dy = p.y - cy;
    dx * dx + dy * dy < r * r
}

enum Side {
    /// Struck the top or bottom face; sign is the y direction to leave in.
    Vertical(f32),
    /// Struck the left or right face; sign is the x direction to leave in.
    Horizontal(f32),
    Corner(Vec2),
    Inside,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        let mut g = Game {
            level: 0,
            cycle: 0,
            bricks: vec![None; COLS * ROWS],
            balls: Vec::new(),
            paddle: Paddle {
                x: PF_W * 0.5,
                size: DEFAULT_PADDLE_SIZE,
                width: PADDLE_SIZES[DEFAULT_PADDLE_SIZE],
            },
            drops: Vec::new(),
            bolts: Vec::new(),
            score: 0,
            lives: START_LIVES,
            speed: SPEED_START,
            effects: Effects::default(),
            phase: Phase::Play,
            events: Vec::new(),
            time: 0.0,
            level_time: 0.0,
            rng: Rng::new(seed),
            pending_explosions: Vec::new(),
            since_progress: 0.0,
            bolt_cooldown: 0.0,
            accumulator: 0.0,
            key_x: None,
        };
        g.load_level(0);
        g
    }

    pub fn level_name(&self) -> &'static str {
        LEVELS[self.level].name
    }

    pub fn base_speed(&self) -> f32 {
        SPEED_START + SPEED_PER_LEVEL * (self.level.min(10) as f32) + 25.0 * self.cycle as f32
    }

    pub fn load_level(&mut self, level: usize) {
        self.level = level % LEVELS.len();
        self.bricks = vec![None; COLS * ROWS];
        for (r, row) in LEVELS[self.level].rows.iter().enumerate().take(ROWS) {
            for (c, ch) in row.chars().enumerate().take(COLS) {
                self.bricks[cell_index(c, r)] = BrickKind::from_char(ch).map(Brick::new);
            }
        }
        self.drops.clear();
        self.bolts.clear();
        self.pending_explosions.clear();
        self.effects = Effects::default();
        self.paddle.size = DEFAULT_PADDLE_SIZE;
        self.level_time = 0.0;
        self.phase = Phase::Play;
        self.serve();
        self.events.push(Event::LevelStart);
        self.events.push(Event::Message(format!(
            "Level {}: {}.",
            self.level + 1,
            LEVELS[self.level].name
        )));
    }

    /// Put a fresh ball on the paddle.
    pub fn serve(&mut self) {
        self.speed = self.base_speed();
        self.balls.clear();
        let off = self.paddle.half() * 0.3;
        self.balls.push(Ball {
            pos: vec2(self.paddle.x + off, PADDLE_Y - BALL_R),
            dir: paddle_bounce_dir(off, self.paddle.half()),
            stuck: Some(off),
            stuck_time: 0.0,
        });
        self.since_progress = 0.0;
    }

    /// Centre of the paddle's top surface.
    pub fn paddle_pos(&self) -> Vec2 {
        vec2(self.paddle.x, PADDLE_Y)
    }

    pub fn breakable_left(&self) -> usize {
        self.bricks
            .iter()
            .flatten()
            .filter(|b| b.kind.breakable())
            .count()
    }

    pub fn serving(&self) -> bool {
        self.phase == Phase::Play
            && self.balls.len() == 1
            && self.balls[0].stuck.is_some()
            && !self.effects.grab
    }

    /// Advance the simulation by `dt` seconds of wall time.
    pub fn update(&mut self, dt: f32, input: &Input) {
        if matches!(self.phase, Phase::GameOver | Phase::Victory) {
            return;
        }
        self.apply_paddle_input(dt, input);
        if input.fire && self.phase == Phase::Play {
            self.release_balls();
        }
        if input.fire_held || input.fire {
            self.try_fire_bolts();
        }

        self.accumulator += dt.min(0.1);
        while self.accumulator >= FIXED_DT {
            self.accumulator -= FIXED_DT;
            self.step(FIXED_DT);
            if matches!(self.phase, Phase::GameOver | Phase::Victory) {
                break;
            }
        }
    }

    fn apply_paddle_input(&mut self, dt: f32, input: &Input) {
        let hw = self.paddle.half();
        if let Some(x) = input.paddle_x {
            self.key_x = None;
            self.paddle.x = x;
        } else if input.left || input.right {
            let dir = input.right as i32 as f32 - input.left as i32 as f32;
            let x = self.key_x.unwrap_or(self.paddle.x) + dir * PADDLE_KEY_SPEED * dt;
            self.key_x = Some(x.clamp(hw, PF_W - hw));
            self.paddle.x = x;
        }
        self.paddle.x = self.paddle.x.clamp(hw, PF_W - hw);
    }

    pub fn release_balls(&mut self) {
        let hw = self.paddle.half();
        let mut released = false;
        for b in &mut self.balls {
            if let Some(off) = b.stuck.take() {
                b.dir = paddle_bounce_dir(off, hw);
                b.pos.y = PADDLE_Y - BALL_R - 0.5;
                released = true;
            }
        }
        if released {
            self.events.push(Event::Sfx(Sfx::Launch));
        }
    }

    fn try_fire_bolts(&mut self) {
        if self.phase != Phase::Play || self.effects.crossbow <= 0.0 || self.bolt_cooldown > 0.0 {
            return;
        }
        // Don't shoot on the click that launches the serve.
        if self.balls.iter().any(|b| b.stuck.is_some()) && !self.effects.grab {
            return;
        }
        let hw = self.paddle.half();
        for dx in [-hw + 3.0, hw - 3.0] {
            self.bolts.push(Bolt {
                pos: vec2(self.paddle.x + dx, PADDLE_Y - 2.0),
            });
        }
        self.bolt_cooldown = BOLT_COOLDOWN;
        self.events.push(Event::Sfx(Sfx::Bolt));
    }

    /// One fixed simulation step.
    pub fn step(&mut self, h: f32) {
        self.time += h;
        for b in self.bricks.iter_mut().flatten() {
            b.flash += h;
        }
        match self.phase {
            Phase::Play => {}
            Phase::Cleared(t) => {
                let t = t - h;
                if t <= 0.0 {
                    self.next_level();
                } else {
                    self.phase = Phase::Cleared(t);
                }
                return;
            }
            Phase::Dying(t) => {
                let t = t - h;
                if t <= 0.0 {
                    self.after_death();
                } else {
                    self.phase = Phase::Dying(t);
                }
                return;
            }
            Phase::GameOver | Phase::Victory => return,
        }

        self.level_time += h;
        self.since_progress += h;
        self.bolt_cooldown = (self.bolt_cooldown - h).max(0.0);

        // Paddle size eases toward its target.
        let target = PADDLE_SIZES[self.paddle.size];
        let dw = target - self.paddle.width;
        self.paddle.width += dw.clamp(-140.0 * h, 140.0 * h);
        let hw = self.paddle.half();
        self.paddle.x = self.paddle.x.clamp(hw, PF_W - hw);

        // Timed effects.
        let e = &mut self.effects;
        e.fire = (e.fire - h).max(0.0);
        e.phantom = (e.phantom - h).max(0.0);
        e.crossbow = (e.crossbow - h).max(0.0);

        if self.balls.iter().any(|b| b.stuck.is_none()) {
            self.speed = (self.speed + SPEED_RAMP * h).min(SPEED_MAX);
        }

        // Balls, sub-stepped so nothing moves more than a pixel per collision test.
        let travel = self.speed * h;
        let subs = (travel / 1.0).ceil().max(1.0) as usize;
        let sh = h / subs as f32;
        let mut i = 0;
        while i < self.balls.len() {
            let mut alive = true;
            for _ in 0..subs {
                if !self.step_ball(i, sh) {
                    alive = false;
                    break;
                }
            }
            if alive {
                i += 1;
            } else {
                let lost = self.balls.remove(i);
                self.events.push(Event::BallLost { pos: lost.pos });
            }
        }

        self.step_bolts(h);
        self.step_drops(h);
        self.step_explosions(h);

        if self.phase != Phase::Play {
            return;
        }
        if self.balls.is_empty() {
            self.lose_life();
        } else if self.breakable_left() == 0 && self.pending_explosions.is_empty() {
            self.clear_level();
        }
    }

    /// Moves ball `i` by one sub-step. Returns false if it fell out of the playfield.
    fn step_ball(&mut self, i: usize, h: f32) -> bool {
        let mut b = self.balls[i];
        if let Some(off) = b.stuck {
            let hw = self.paddle.half();
            let off = off.clamp(-hw, hw);
            b.stuck = Some(off);
            b.pos = vec2(self.paddle.x + off, PADDLE_Y - BALL_R);
            b.stuck_time += h;
            self.balls[i] = b;
            if self.effects.grab && b.stuck_time > GRAB_RELEASE_TIME {
                self.release_balls();
            }
            return true;
        }

        let prev = b.pos;
        b.pos += b.dir * self.speed * h;

        // Walls.
        let mut wall = false;
        if b.pos.x < BALL_R {
            b.pos.x = BALL_R;
            b.dir.x = b.dir.x.abs();
            wall = true;
        } else if b.pos.x > PF_W - BALL_R {
            b.pos.x = PF_W - BALL_R;
            b.dir.x = -b.dir.x.abs();
            wall = true;
        }
        if b.pos.y < BALL_R {
            b.pos.y = BALL_R;
            b.dir.y = b.dir.y.abs();
            wall = true;
        }
        if wall {
            self.events.push(Event::Sfx(Sfx::Wall));
            b.dir = self.unstick(b.dir);
        }

        self.collide_bricks(&mut b, prev);

        // Paddle: only the top surface counts, like the original.
        let hw = self.paddle.half();
        if b.dir.y > 0.0
            && prev.y + BALL_R <= PADDLE_Y + 1.5
            && b.pos.y + BALL_R >= PADDLE_Y
            && (b.pos.x - self.paddle.x).abs() <= hw + BALL_R
        {
            let off = b.pos.x - self.paddle.x;
            b.pos.y = PADDLE_Y - BALL_R;
            b.dir = paddle_bounce_dir(off, hw);
            self.since_progress = 0.0;
            self.events.push(Event::PaddleHit { pos: b.pos });
            self.events.push(Event::Sfx(Sfx::Paddle));
            if self.effects.grab {
                b.stuck = Some(off.clamp(-hw, hw));
                b.stuck_time = 0.0;
            }
        }

        self.balls[i] = b;
        b.pos.y - BALL_R <= PF_H
    }

    /// Nudges the ball if it has been bouncing without progress for too long.
    fn unstick(&mut self, dir: Vec2) -> Vec2 {
        if self.since_progress < STUCK_TIME {
            return dir;
        }
        self.since_progress = 0.0;
        let deg = self.rng.range(8.0, 16.0) * if self.rng.chance(0.5) { 1.0 } else { -1.0 };
        clamp_angle(rotate(dir, deg))
    }

    fn collide_bricks(&mut self, b: &mut Ball, prev: Vec2) {
        let c0 = ((b.pos.x - BALL_R) / BRICK_W).floor().max(0.0) as usize;
        let c1 = (((b.pos.x + BALL_R) / BRICK_W).floor().max(0.0) as usize).min(COLS - 1);
        let ry0 = (b.pos.y - BALL_R - BRICK_TOP) / BRICK_H;
        let ry1 = (b.pos.y + BALL_R - BRICK_TOP) / BRICK_H;
        if ry1 < 0.0 || ry0 >= ROWS as f32 {
            return;
        }
        let r0 = ry0.floor().max(0.0) as usize;
        let r1 = (ry1.floor().max(0.0) as usize).min(ROWS - 1);

        let mut hits: [(usize, Side); 6] = std::array::from_fn(|_| (0, Side::Inside));
        let mut n = 0;
        for r in r0..=r1 {
            for c in c0..=c1 {
                let idx = cell_index(c, r);
                if self.bricks[idx].is_none() {
                    continue;
                }
                let rect = cell_rect(idx);
                if !circle_hits_rect(b.pos, BALL_R, rect) {
                    continue;
                }
                let (l, t, rr, bt) = rect;
                let in_x = prev.x >= l && prev.x <= rr;
                let in_y = prev.y >= t && prev.y <= bt;
                let side = if in_x && in_y {
                    Side::Inside
                } else if in_x {
                    Side::Vertical(if prev.y < t { -1.0 } else { 1.0 })
                } else if in_y {
                    Side::Horizontal(if prev.x < l { -1.0 } else { 1.0 })
                } else {
                    let cx = if prev.x < l { l } else { rr };
                    let cy = if prev.y < t { t } else { bt };
                    Side::Corner(vec2(cx, cy))
                };
                if n < hits.len() {
                    hits[n] = (idx, side);
                    n += 1;
                }
            }
        }
        if n == 0 {
            return;
        }

        let phantom = self.effects.phantom > 0.0;
        let fire = self.effects.fire > 0.0;
        let mut new_dir = b.dir;
        let mut face = false;
        let mut corner = None;
        for (idx, side) in hits.iter().take(n) {
            let Some(brick) = self.bricks[*idx] else {
                continue;
            };
            let passes = phantom && brick.kind.breakable();
            if !passes {
                match side {
                    Side::Vertical(sy) => {
                        new_dir.y = new_dir.y.abs() * sy;
                        face = true;
                    }
                    Side::Horizontal(sx) => {
                        new_dir.x = new_dir.x.abs() * sx;
                        face = true;
                    }
                    Side::Corner(p) => corner = Some(*p),
                    Side::Inside => {}
                }
            }
            if fire && brick.kind.breakable() {
                self.fire_blast(*idx);
            } else if passes {
                self.destroy_brick(*idx);
            } else {
                self.hit_brick(*idx);
            }
        }

        if face {
            b.dir = clamp_angle(new_dir);
            b.pos = prev;
        } else if let Some(p) = corner {
            let nrm = (prev - p).normalize_or(-b.dir);
            let d = b.dir.dot(nrm);
            if d < 0.0 {
                b.dir = clamp_angle(b.dir - 2.0 * d * nrm);
            }
            b.pos = prev;
        }
        // Iron never breaks, so a ball can loop between iron and walls forever.
        let all_iron = hits
            .iter()
            .take(n)
            .all(|(idx, _)| self.bricks[*idx].is_some_and(|b| b.kind == BrickKind::Iron));
        if (face || corner.is_some()) && all_iron {
            b.dir = self.unstick(b.dir);
        }
    }

    fn hit_brick(&mut self, idx: usize) {
        let Some(brick) = self.bricks[idx].as_mut() else {
            return;
        };
        brick.flash = 0.0;
        let kind = brick.kind;
        let pos = cell_center(idx);
        if kind == BrickKind::Iron {
            self.events.push(Event::Sfx(Sfx::Clang));
            self.events.push(Event::BrickHit { pos, kind });
            return;
        }
        brick.hp = brick.hp.saturating_sub(1);
        if brick.hp == 0 {
            self.destroy_brick(idx);
        } else {
            self.score += 5;
            self.events.push(Event::Sfx(Sfx::BrickHit));
            self.events.push(Event::BrickHit { pos, kind });
        }
    }

    pub fn destroy_brick(&mut self, idx: usize) {
        let Some(brick) = self.bricks[idx].take() else {
            return;
        };
        let pos = cell_center(idx);
        self.score += brick.kind.points();
        self.since_progress = 0.0;
        self.events.push(Event::BrickBroken {
            pos,
            kind: brick.kind,
        });
        self.events.push(Event::Sfx(Sfx::BrickBreak));
        if brick.kind == BrickKind::Rune {
            self.pending_explosions.push((idx, 0.08));
        }
        if brick.kind == BrickKind::Gold {
            let power = Power::pick(&mut self.rng, true);
            self.drops.push(Drop {
                power,
                pos,
                age: 0.0,
            });
        } else if self.rng.chance(DROP_CHANCE) {
            let power = Power::pick(&mut self.rng, false);
            self.drops.push(Drop {
                power,
                pos,
                age: 0.0,
            });
        }
    }

    fn neighbours(idx: usize) -> impl Iterator<Item = usize> {
        let c = (idx % COLS) as i32;
        let r = (idx / COLS) as i32;
        (-1..=1)
            .flat_map(move |dy| (-1..=1).map(move |dx| (c + dx, r + dy)))
            .filter(move |&(x, y)| {
                (x, y) != (c, r) && x >= 0 && y >= 0 && x < COLS as i32 && y < ROWS as i32
            })
            .map(|(x, y)| cell_index(x as usize, y as usize))
    }

    /// Fireball impact: destroys the brick and its neighbours (iron excepted).
    fn fire_blast(&mut self, idx: usize) {
        let pos = cell_center(idx);
        self.destroy_brick(idx);
        for n in Self::neighbours(idx) {
            if self.bricks[n].is_some_and(|b| b.kind.breakable()) {
                self.destroy_brick(n);
            }
        }
        self.events.push(Event::Explosion { pos, big: false });
        self.events.push(Event::Sfx(Sfx::Explosion));
        self.events.push(Event::Shake(1.5));
    }

    fn step_explosions(&mut self, h: f32) {
        if self.pending_explosions.is_empty() {
            return;
        }
        let mut due = Vec::new();
        self.pending_explosions.retain_mut(|(idx, t)| {
            *t -= h;
            if *t <= 0.0 {
                due.push(*idx);
                false
            } else {
                true
            }
        });
        for idx in due {
            self.events.push(Event::Explosion {
                pos: cell_center(idx),
                big: true,
            });
            self.events.push(Event::Sfx(Sfx::Explosion));
            self.events.push(Event::Shake(3.0));
            for n in Self::neighbours(idx) {
                if self.bricks[n].is_some_and(|b| b.kind.breakable()) {
                    self.destroy_brick(n);
                }
            }
        }
    }

    fn step_bolts(&mut self, h: f32) {
        let mut i = 0;
        while i < self.bolts.len() {
            let mut bolt = self.bolts[i];
            bolt.pos.y -= BOLT_SPEED * h;
            let mut gone = bolt.pos.y < 0.0;
            if !gone {
                let col = (bolt.pos.x / BRICK_W).floor();
                let row = ((bolt.pos.y - BRICK_TOP) / BRICK_H).floor();
                if col >= 0.0 && row >= 0.0 && (col as usize) < COLS && (row as usize) < ROWS {
                    let idx = cell_index(col as usize, row as usize);
                    if self.bricks[idx].is_some() {
                        self.hit_brick(idx);
                        gone = true;
                    }
                }
            }
            if gone {
                self.bolts.swap_remove(i);
            } else {
                self.bolts[i] = bolt;
                i += 1;
            }
        }
    }

    fn step_drops(&mut self, h: f32) {
        let hw = self.paddle.half();
        let half = DROP_SIZE * 0.5;
        let mut i = 0;
        while i < self.drops.len() {
            let d = &mut self.drops[i];
            d.pos.y += DROP_SPEED * h;
            d.age += h;
            let caught = d.pos.y + half >= PADDLE_Y
                && d.pos.y - half <= PADDLE_Y + PADDLE_H
                && (d.pos.x - self.paddle.x).abs() <= hw + half;
            let (power, pos) = (d.power, d.pos);
            if caught {
                self.drops.swap_remove(i);
                self.apply_power(power, pos);
                if self.phase != Phase::Play {
                    return;
                }
            } else if pos.y - half > PF_H {
                self.drops.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn apply_power(&mut self, power: Power, pos: Vec2) {
        self.score += 25;
        self.events.push(Event::PowerCollected { pos, power });
        self.events.push(Event::Sfx(if power.good() {
            Sfx::PowerGood
        } else {
            Sfx::PowerBad
        }));
        self.events
            .push(Event::Message(power.message().to_string()));
        match power {
            Power::Expand => self.paddle.size = (self.paddle.size + 1).min(PADDLE_SIZES.len() - 1),
            Power::Shrink => self.paddle.size = self.paddle.size.saturating_sub(1),
            Power::Split => {
                self.release_balls();
                let existing: Vec<Ball> = self.balls.clone();
                for b in existing {
                    for deg in [-25.0, 25.0] {
                        if self.balls.len() >= MAX_BALLS {
                            break;
                        }
                        self.balls.push(Ball {
                            dir: clamp_angle(rotate(b.dir, deg)),
                            ..b
                        });
                    }
                }
            }
            Power::Legion => {
                self.release_balls();
                if let Some(&b) = self.balls.first() {
                    for k in 0..8 {
                        if self.balls.len() >= MAX_BALLS {
                            break;
                        }
                        let a = (22.5 + 45.0 * k as f32).to_radians();
                        let dir = clamp_angle(vec2(a.cos(), a.sin()));
                        self.balls.push(Ball { dir, ..b });
                    }
                    self.balls.remove(0);
                }
            }
            Power::Fire => {
                self.effects.fire = FIRE_TIME;
                self.effects.phantom = 0.0;
            }
            Power::Phantom => {
                self.effects.phantom = PHANTOM_TIME;
                self.effects.fire = 0.0;
            }
            Power::Haste => self.speed = (self.speed + 45.0).min(SPEED_MAX),
            Power::Slow => self.speed = (self.speed - 60.0).max(self.base_speed() * 0.8),
            Power::Grab => self.effects.grab = true,
            Power::Crossbow => self.effects.crossbow = CROSSBOW_TIME,
            Power::Life => self.lives = (self.lives + 1).min(MAX_LIVES),
            Power::Skull => {
                for b in self.balls.drain(..) {
                    self.events.push(Event::BallLost { pos: b.pos });
                }
                self.lose_life();
            }
            Power::Quake => {
                self.quake();
                self.events.push(Event::Shake(4.0));
            }
            Power::Zap => {
                for b in self.bricks.iter_mut().flatten() {
                    if b.kind.breakable() {
                        b.hp = 1;
                        b.flash = 0.0;
                    }
                }
            }
            Power::Warp => {
                self.score += 500;
                self.clear_level();
            }
        }
    }

    /// Every brick with free space below it drops one row.
    pub fn quake(&mut self) {
        for r in (0..ROWS - 1).rev() {
            for c in 0..COLS {
                let from = cell_index(c, r);
                let to = cell_index(c, r + 1);
                if self.bricks[from].is_some() && self.bricks[to].is_none() {
                    self.bricks[to] = self.bricks[from].take();
                }
            }
        }
        self.pending_explosions.clear();
    }

    fn lose_life(&mut self) {
        self.balls.clear();
        self.drops.clear();
        self.bolts.clear();
        self.effects = Effects::default();
        self.phase = Phase::Dying(1.4);
        self.events.push(Event::Sfx(Sfx::LifeLost));
        self.events.push(Event::Shake(2.0));
    }

    fn after_death(&mut self) {
        self.lives = self.lives.saturating_sub(1);
        if self.lives == 0 {
            self.phase = Phase::GameOver;
            self.events.push(Event::GameOver);
            self.events.push(Event::Sfx(Sfx::GameOver));
            self.events.push(Event::Message(
                "Thy quest has ended. The Abyss claims thee.".into(),
            ));
            return;
        }
        self.paddle.size = DEFAULT_PADDLE_SIZE;
        self.phase = Phase::Play;
        self.serve();
        let msg = if self.lives == 1 {
            "The orb is lost! This is thy last chance.".to_string()
        } else {
            format!("The orb is lost! {} chances remain.", self.lives)
        };
        self.events.push(Event::Message(msg));
    }

    fn clear_level(&mut self) {
        let bonus = 500 + 100 * (self.level as u64 + 1) + 1000 * self.cycle as u64;
        self.score += bonus;
        self.balls.clear();
        self.drops.clear();
        self.bolts.clear();
        self.phase = Phase::Cleared(2.6);
        self.events.push(Event::LevelCleared);
        self.events.push(Event::Sfx(Sfx::LevelClear));
        self.events
            .push(Event::Message(format!("The way is clear! Bonus: {bonus}.")));
    }

    fn next_level(&mut self) {
        if self.level + 1 >= LEVELS.len() {
            self.phase = Phase::Victory;
            self.events.push(Event::Victory);
            self.events
                .push(Event::Message("Thou hast conquered the Abyss!".into()));
            return;
        }
        self.load_level(self.level + 1);
    }

    /// After victory, continue with a faster pass through the levels.
    pub fn continue_cycle(&mut self) {
        self.cycle += 1;
        self.load_level(0);
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests;
