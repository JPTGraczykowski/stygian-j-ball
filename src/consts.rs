//! Layout and tuning constants shared by game logic and rendering.
//!
//! All gameplay coordinates are in *playfield space*: (0,0) is the top-left of the
//! playfield, one unit equals one pixel of the 384x240 virtual screen.

/// Virtual screen size (the software framebuffer).
pub const SCREEN_W: i32 = 384;
pub const SCREEN_H: i32 = 240;

/// Playfield placement on the virtual screen.
pub const PF_X: i32 = 8;
pub const PF_Y: i32 = 8;
pub const PF_W: f32 = 288.0;
pub const PF_H: f32 = 192.0;

/// Brick grid.
pub const COLS: usize = 16;
pub const ROWS: usize = 18;
pub const BRICK_W: f32 = 18.0;
pub const BRICK_H: f32 = 8.0;
pub const BRICK_TOP: f32 = 12.0;

/// Paddle.
pub const PADDLE_Y: f32 = 178.0;
pub const PADDLE_H: f32 = 6.0;
/// Paddle widths by size step; DEFAULT_PADDLE_SIZE indexes this.
pub const PADDLE_SIZES: [f32; 6] = [16.0, 24.0, 32.0, 42.0, 54.0, 68.0];
pub const DEFAULT_PADDLE_SIZE: usize = 2;
pub const PADDLE_KEY_SPEED: f32 = 330.0;

/// Ball.
pub const BALL_R: f32 = 2.5;
/// Maximum deflection from vertical when the ball strikes the paddle's very edge.
pub const MAX_BOUNCE_DEG: f32 = 64.0;
/// The ball never travels flatter than this angle above the horizontal.
pub const MIN_ANGLE_DEG: f32 = 15.0;
pub const SPEED_START: f32 = 125.0;
pub const SPEED_PER_LEVEL: f32 = 7.0;
/// Ball speed creeps upward while in play (px/s per second), as in DX-Ball.
pub const SPEED_RAMP: f32 = 0.9;
pub const SPEED_MAX: f32 = 270.0;
pub const MAX_BALLS: usize = 24;

/// Simulation.
pub const FIXED_DT: f32 = 1.0 / 240.0;

/// Falling power-ups and bolts.
pub const DROP_SPEED: f32 = 50.0;
pub const DROP_SIZE: f32 = 10.0;
pub const DROP_CHANCE: f32 = 0.17;
pub const BOLT_SPEED: f32 = 270.0;
pub const BOLT_COOLDOWN: f32 = 0.28;

/// Timed effects (seconds).
pub const FIRE_TIME: f32 = 18.0;
pub const PHANTOM_TIME: f32 = 9.0;
pub const CROSSBOW_TIME: f32 = 22.0;
pub const GRAB_RELEASE_TIME: f32 = 3.0;

pub const START_LIVES: u32 = 3;
pub const MAX_LIVES: u32 = 9;

/// If nothing productive happens for this long, wall bounces get nudged to break loops.
pub const STUCK_TIME: f32 = 7.0;
