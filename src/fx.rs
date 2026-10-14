//! Purely cosmetic effects driven by game events: debris, sparks, embers, smoke,
//! transient light, screen shake and floating text. Positions are in playfield space.

use crate::consts::*;
use crate::game::{BrickKind, Event, Game, Power};
use crate::gfx::canvas::Rgba;
use crate::gfx::dungeon::Light;
use crate::gfx::palette::*;
use crate::rng::Rng;
use macroquad::math::{Vec2, vec2};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PKind {
    /// Falls with gravity and bounces on the floor.
    Debris,
    /// Fast, bright, short-lived; additive.
    Spark,
    /// Rising fire; additive.
    Ember,
    /// Slow dark puff.
    Smoke,
    /// Sparkle that drifts upward; additive.
    Magic,
}

#[derive(Clone, Copy)]
pub struct Particle {
    pub kind: PKind,
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub color: Rgba,
}

#[derive(Clone, Copy)]
pub struct Flash {
    pub light: Light,
    pub life: f32,
    pub max_life: f32,
}

pub struct FloatText {
    pub text: String,
    pub pos: Vec2,
    pub life: f32,
    pub good: bool,
}

pub struct Fx {
    pub particles: Vec<Particle>,
    pub flashes: Vec<Flash>,
    pub texts: Vec<FloatText>,
    pub shake: f32,
    pub screen_flash: f32,
    /// Gargoyle eye glow, 0..1.
    pub eye_glow: f32,
    pub paddle_squash: f32,
    rng: Rng,
    trail_timer: f32,
}

fn brick_colors(kind: BrickKind) -> [Rgba; 3] {
    match kind {
        BrickKind::Stone(0) | BrickKind::Stone(2) => [GREY_STONE[1], GREY_STONE[3], GREY_STONE[4]],
        BrickKind::Stone(1) => [SANDSTONE[2], SANDSTONE[3], SANDSTONE[4]],
        BrickKind::Stone(3) => [CLAY[2], CLAY[3], CLAY[4]],
        BrickKind::Stone(4) | BrickKind::Bound => [WOOD[2], WOOD[3], WOOD[4]],
        BrickKind::Stone(_) => [SLATE[2], SLATE[3], SLATE[4]],
        BrickKind::Crystal => [CRYSTAL[3], CRYSTAL[5], CRYSTAL[6]],
        BrickKind::Iron => [IRON[3], IRON[5], IRON[6]],
        BrickKind::Rune => [GREY_STONE[1], RUNE_GLOW[3], RUNE_GLOW[4]],
        BrickKind::Gold => [GOLD[3], GOLD[5], GOLD[6]],
    }
}

impl Default for Fx {
    fn default() -> Self {
        Self::new()
    }
}

impl Fx {
    pub fn new() -> Fx {
        Fx {
            particles: Vec::new(),
            flashes: Vec::new(),
            texts: Vec::new(),
            shake: 0.0,
            screen_flash: 0.0,
            eye_glow: 0.0,
            paddle_squash: 0.0,
            rng: Rng::new(99),
            trail_timer: 0.0,
        }
    }

    pub fn clear(&mut self) {
        self.particles.clear();
        self.flashes.clear();
        self.texts.clear();
        self.shake = 0.0;
        self.screen_flash = 0.0;
    }

    fn spawn(&mut self, kind: PKind, pos: Vec2, vel: Vec2, life: f32, color: Rgba) {
        if self.particles.len() < 1500 {
            self.particles.push(Particle {
                kind,
                pos,
                vel,
                life,
                max_life: life,
                color,
            });
        }
    }

    fn flash(&mut self, pos: Vec2, radius: f32, intensity: f32, cool: bool, life: f32) {
        self.flashes.push(Flash {
            light: Light {
                x: pos.x,
                y: pos.y,
                radius,
                intensity,
                cool,
            },
            life,
            max_life: life,
        });
    }

    pub fn handle(&mut self, e: &Event) {
        match e {
            Event::BrickBroken { pos, kind } => {
                let cols = brick_colors(*kind);
                for _ in 0..14 {
                    let p = *pos + vec2(self.rng.range(-8.0, 8.0), self.rng.range(-3.0, 3.0));
                    let v = vec2(self.rng.range(-50.0, 50.0), self.rng.range(-70.0, 10.0));
                    let c = cols[self.rng.below(3) as usize];
                    let life = self.rng.range(0.5, 1.1);
                    self.spawn(PKind::Debris, p, v, life, c);
                }
                for _ in 0..4 {
                    let p = *pos + vec2(self.rng.range(-6.0, 6.0), 0.0);
                    let v = vec2(self.rng.range(-8.0, 8.0), self.rng.range(-14.0, -4.0));
                    let life = self.rng.range(0.4, 0.8);
                    self.spawn(PKind::Smoke, p, v, life, FRAME[3]);
                }
                match kind {
                    BrickKind::Crystal => {
                        for _ in 0..8 {
                            let v = vec2(self.rng.range(-60.0, 60.0), self.rng.range(-60.0, 20.0));
                            self.spawn(PKind::Magic, *pos, v, 0.6, CRYSTAL[6]);
                        }
                        self.flash(*pos, 28.0, 0.5, true, 0.3);
                    }
                    BrickKind::Gold => {
                        for _ in 0..12 {
                            let v = vec2(self.rng.range(-50.0, 50.0), self.rng.range(-80.0, -10.0));
                            self.spawn(PKind::Magic, *pos, v, 0.9, GOLD[6]);
                        }
                        self.texts.push(FloatText {
                            text: "+100".into(),
                            pos: *pos,
                            life: 1.0,
                            good: true,
                        });
                    }
                    _ => {}
                }
            }
            Event::BrickHit { pos, kind } => {
                let n = if *kind == BrickKind::Iron { 6 } else { 3 };
                for _ in 0..n {
                    let v = vec2(self.rng.range(-70.0, 70.0), self.rng.range(-70.0, 70.0));
                    let c = if *kind == BrickKind::Iron {
                        GOLD[6]
                    } else {
                        brick_colors(*kind)[2]
                    };
                    let life = self.rng.range(0.12, 0.3);
                    self.spawn(PKind::Spark, *pos, v, life, c);
                }
                if *kind == BrickKind::Iron {
                    self.flash(*pos, 14.0, 0.35, false, 0.12);
                }
            }
            Event::Explosion { pos, big } => {
                let (n, r) = if *big { (40, 60.0) } else { (18, 34.0) };
                for _ in 0..n {
                    let a = self.rng.range(0.0, std::f32::consts::TAU);
                    let s = self.rng.range(15.0, if *big { 110.0 } else { 70.0 });
                    let v = vec2(a.cos() * s, a.sin() * s - 20.0);
                    let c = FIRE[3 + self.rng.below(4) as usize];
                    let life = self.rng.range(0.3, 0.7);
                    self.spawn(PKind::Ember, *pos, v, life, c);
                }
                for _ in 0..n / 3 {
                    let p = *pos + vec2(self.rng.range(-10.0, 10.0), self.rng.range(-6.0, 6.0));
                    let v = vec2(self.rng.range(-12.0, 12.0), self.rng.range(-25.0, -8.0));
                    let life = self.rng.range(0.7, 1.3);
                    self.spawn(PKind::Smoke, p, v, life, FRAME[2]);
                }
                self.flash(
                    *pos,
                    r,
                    if *big { 1.6 } else { 1.0 },
                    false,
                    if *big { 0.5 } else { 0.3 },
                );
            }
            Event::PaddleHit { pos } => {
                self.paddle_squash = 1.0;
                for _ in 0..3 {
                    let v = vec2(self.rng.range(-30.0, 30.0), self.rng.range(-30.0, -10.0));
                    self.spawn(PKind::Spark, *pos + vec2(0.0, 2.0), v, 0.15, ORB[3]);
                }
            }
            Event::PowerCollected { pos, power } => {
                let good = power.good();
                let c = if good { MANA[5] } else { BLOOD[5] };
                for _ in 0..16 {
                    let v = vec2(self.rng.range(-40.0, 40.0), self.rng.range(-90.0, -20.0));
                    let life = self.rng.range(0.5, 1.0);
                    self.spawn(PKind::Magic, *pos, v, life, c);
                }
                self.flash(*pos, 40.0, 0.8, good, 0.4);
                self.texts.push(FloatText {
                    text: power.name().into(),
                    pos: *pos,
                    life: 1.6,
                    good,
                });
                if *power == Power::Skull {
                    self.screen_flash = 1.0;
                }
                if *power == Power::Warp {
                    self.screen_flash = 0.8;
                }
            }
            Event::BallLost { pos } => {
                for _ in 0..10 {
                    let v = vec2(self.rng.range(-30.0, 30.0), self.rng.range(-60.0, -20.0));
                    self.spawn(PKind::Magic, vec2(pos.x, PF_H - 2.0), v, 0.6, ORB[2]);
                }
            }
            Event::Shake(s) => self.shake = self.shake.max(*s),
            Event::GameOver => self.eye_glow = 1.0,
            Event::LevelCleared => self.screen_flash = 0.5,
            Event::Sfx(crate::game::Sfx::LifeLost) => self.eye_glow = 1.0,
            _ => {}
        }
    }

    /// Advance effects; `game` is used for continuous effects such as ball trails.
    pub fn update(&mut self, dt: f32, game: Option<&Game>) {
        for p in &mut self.particles {
            p.life -= dt;
            match p.kind {
                PKind::Debris => {
                    p.vel.y += 260.0 * dt;
                    p.vel.x *= 1.0 - 0.6 * dt;
                    if p.pos.y >= PF_H - 1.0 && p.vel.y > 0.0 {
                        p.vel.y *= -0.35;
                        p.vel.x *= 0.6;
                        p.pos.y = PF_H - 1.0;
                    }
                }
                PKind::Spark => p.vel *= 1.0 - 4.0 * dt,
                PKind::Ember => {
                    p.vel *= 1.0 - 3.0 * dt;
                    p.vel.y -= 30.0 * dt;
                }
                PKind::Smoke => p.vel *= 1.0 - 1.5 * dt,
                PKind::Magic => {
                    p.vel *= 1.0 - 2.0 * dt;
                    p.vel.y -= 10.0 * dt;
                }
            }
            p.pos += p.vel * dt;
        }
        self.particles
            .retain(|p| p.life > 0.0 && p.pos.x > -10.0 && p.pos.x < PF_W + 10.0);
        for f in &mut self.flashes {
            f.life -= dt;
        }
        self.flashes.retain(|f| f.life > 0.0);
        for t in &mut self.texts {
            t.life -= dt;
            t.pos.y -= 14.0 * dt;
        }
        self.texts.retain(|t| t.life > 0.0);
        self.shake = (self.shake - dt * 10.0).max(0.0);
        self.screen_flash = (self.screen_flash - dt * 2.5).max(0.0);
        self.eye_glow = (self.eye_glow - dt * 0.5).max(0.0);
        self.paddle_squash = (self.paddle_squash - dt * 8.0).max(0.0);

        // Fire and phantom orbs leave trails.
        if let Some(g) = game {
            self.trail_timer -= dt;
            if self.trail_timer <= 0.0 {
                self.trail_timer = 1.0 / 60.0;
                let fire = g.effects.fire > 0.0;
                let ghost = g.effects.phantom > 0.0;
                if fire || ghost {
                    for b in g.balls.clone() {
                        let jitter = vec2(self.rng.range(-1.0, 1.0), self.rng.range(-1.0, 1.0));
                        if fire {
                            let c = FIRE[3 + self.rng.below(3) as usize];
                            self.spawn(PKind::Ember, b.pos + jitter, -b.dir * 12.0, 0.3, c);
                        } else {
                            self.spawn(PKind::Magic, b.pos + jitter, -b.dir * 8.0, 0.35, GHOST[2]);
                        }
                    }
                }
            }
        }
    }

    /// Light sources contributed by effects, for the dungeon renderer.
    pub fn lights(&self, out: &mut Vec<Light>) {
        for f in &self.flashes {
            let k = f.life / f.max_life;
            out.push(Light {
                intensity: f.light.intensity * k,
                ..f.light
            });
        }
    }

    pub fn shake_offset(&mut self) -> (i32, i32) {
        if self.shake <= 0.0 {
            return (0, 0);
        }
        let s = self.shake;
        (
            self.rng.range(-s, s).round() as i32,
            self.rng.range(-s, s).round() as i32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particles_expire() {
        let mut fx = Fx::new();
        fx.handle(&Event::BrickBroken {
            pos: vec2(50.0, 50.0),
            kind: BrickKind::Crystal,
        });
        fx.handle(&Event::Explosion {
            pos: vec2(50.0, 50.0),
            big: true,
        });
        assert!(!fx.particles.is_empty());
        for _ in 0..300 {
            fx.update(1.0 / 60.0, None);
        }
        assert!(fx.particles.is_empty());
        assert!(fx.flashes.is_empty());
    }
}
