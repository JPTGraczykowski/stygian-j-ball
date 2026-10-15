//! Composes a full 384x240 frame: carved-stone UI frame, the dungeon playfield with
//! bricks, orbs and effects, the message scroll, and the side panel (gargoyle,
//! score plaque, ankhs of life, health/mana/speed flasks, active runes and compass).

use super::canvas::{Img, Rgba, dither, rgb};
use super::dungeon::{Dungeon, Light};
use super::font::{
    self, LINE_H, draw_text, draw_text_big, draw_text_centered, draw_text_shadow, text_width,
};
use super::palette::*;
use super::sprites::{self, BallStyle, PaddleLook};
use crate::app::{App, Hotspot, MAX_SCORES, Screen};
use crate::consts::*;
use crate::fx::{Fx, PKind};
use crate::game::{BrickKind, Game, Phase, Power, cell_rect};
use crate::rng::{fbm, hash2, hashf};

pub const PANEL_X: i32 = 302;
pub const PANEL_W: i32 = 78;
pub const SCROLL_X: i32 = 6;
pub const SCROLL_Y: i32 = 203;
pub const SCROLL_W: i32 = 292;
pub const SCROLL_H: i32 = 34;
const COMPASS: (i32, i32, i32) = (341, 211, 22);
/// Left edges of the health, mana and speed flasks.
const FLASK_X: [i32; 3] = [PANEL_X + 7, PANEL_X + 31, PANEL_X + 55];
const PW: i32 = PF_W as i32;
const PH: i32 = PF_H as i32;

pub struct Renderer {
    frame: Img,
    dungeon: Dungeon,
    pf: Img,
    bricks: Vec<Img>,
    balls: [Img; 3],
    bolt: Img,
    icons: Vec<Img>,
    ankh: Img,
    gargoyle_eyes: Vec<(i32, i32)>,
    gargoyle_pos: (i32, i32),
    sconce: Img,
    lights: Vec<Light>,
    shadow: Vec<bool>,
}

/// Index into the brick sprite cache.
fn brick_slot(kind: BrickKind, damage: u8, variant: u32, frame: usize) -> usize {
    match kind {
        BrickKind::Stone(m) => m as usize * 3 + variant as usize % 3,
        BrickKind::Bound => 18 + damage.min(1) as usize,
        BrickKind::Crystal => 20 + damage.min(2) as usize,
        BrickKind::Iron => 23,
        BrickKind::Gold => 24,
        BrickKind::Rune => 25 + frame % 6,
    }
}

fn build_brick_cache() -> Vec<Img> {
    let mut v = Vec::new();
    for m in 0..6u8 {
        for var in 0..3 {
            v.push(sprites::brick(BrickKind::Stone(m), 0, var));
        }
    }
    v.push(sprites::brick(BrickKind::Bound, 0, 1));
    v.push(sprites::brick(BrickKind::Bound, 1, 1));
    for d in 0..3 {
        v.push(sprites::brick(BrickKind::Crystal, d, 2));
    }
    v.push(sprites::brick(BrickKind::Iron, 0, 0));
    v.push(sprites::brick(BrickKind::Gold, 0, 0));
    for f in 0..6 {
        v.push(sprites::brick(BrickKind::Rune, f, 4));
    }
    v
}

pub fn roman(mut n: u32) -> String {
    const T: [(u32, &str); 13] = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut s = String::new();
    for (v, r) in T {
        while n >= v {
            s.push_str(r);
            n -= v;
        }
    }
    s
}

// ---------------------------------------------------------------------------
// Static frame.

/// Carved stone blocks covering the whole image.
pub fn frame_stone(img: &mut Img) {
    for y in 0..img.h {
        let row = y / 12;
        let fy = y % 12;
        let off = (row * 11) % 26;
        for x in 0..img.w {
            let col = (x + off) / 26;
            let fx = (x + off) % 26;
            let v = if fy == 0 || fx == 0 {
                0.06
            } else {
                let base = 0.42 + 0.18 * hashf(col, row, 77);
                let n = (fbm(x as f32 * 0.35, y as f32 * 0.35, 3, 5) - 0.5) * 0.3;
                let bevel = match (fy, fx) {
                    (1, _) => 0.25,
                    (11, _) => -0.22,
                    (_, 1) => 0.12,
                    (_, 25) => -0.12,
                    _ => 0.0,
                };
                base + n + bevel
            };
            img.set(x, y, dither(&FRAME, v, x, y));
        }
    }
}

/// A recessed area: shadowed top-left inner edge, lit bottom-right edge.
fn inset(img: &mut Img, x: i32, y: i32, w: i32, h: i32, fill: bool) {
    if fill {
        for yy in y..y + h {
            for xx in x..x + w {
                let v = 0.12 + 0.12 * fbm(xx as f32 * 0.3, yy as f32 * 0.3, 2, 9);
                img.set(xx, yy, dither(&FRAME, v, xx, yy));
            }
        }
    }
    for d in 1..=2 {
        let (dark, light) = if d == 1 {
            (FRAME[0], FRAME[6])
        } else {
            (FRAME[1], FRAME[5])
        };
        img.hline(x - d, x + w - 1 + d, y - d, dark);
        img.vline(x - d, y - d, y + h - 1 + d, dark);
        img.hline(x - d + 1, x + w - 1 + d, y + h - 1 + d, light);
        img.vline(x + w - 1 + d, y - d + 1, y + h - 1 + d, light);
    }
}

fn stud(img: &mut Img, x: i32, y: i32) {
    img.set(x, y, GOLD[5]);
    img.set(x + 1, y, GOLD[4]);
    img.set(x, y + 1, GOLD[4]);
    img.set(x + 1, y + 1, GOLD[2]);
    img.set(x + 2, y + 1, INK);
    img.set(x + 1, y + 2, INK);
    img.set(x + 2, y + 2, INK);
}

fn parchment_scroll(img: &mut Img) {
    let (x0, y0, w, h) = (SCROLL_X + 6, SCROLL_Y + 1, SCROLL_W - 12, SCROLL_H - 2);
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            let n = fbm(x as f32 * 0.08, y as f32 * 0.15, 4, 61);
            let stain = fbm(x as f32 * 0.03, y as f32 * 0.06, 2, 62);
            let edge = ((y - y0).min(y0 + h - 1 - y) as f32 / 3.0).min(1.0);
            let v = 0.55 + 0.35 * n - if stain > 0.62 { 0.15 } else { 0.0 } - (1.0 - edge) * 0.3;
            img.set(x, y, dither(&PARCHMENT, v, x, y));
        }
    }
    img.hline(x0, x0 + w - 1, y0 - 1, INK);
    img.hline(x0, x0 + w - 1, y0 + h, INK);
    // Rolled ends.
    for (rx, flip) in [(SCROLL_X, false), (SCROLL_X + SCROLL_W - 6, true)] {
        for y in SCROLL_Y - 2..SCROLL_Y + SCROLL_H + 2 {
            for dx in 0..6 {
                let t = if flip { 5 - dx } else { dx } as f32 / 5.0;
                let v =
                    0.25 + 0.7 * (t * std::f32::consts::PI).sin() * 0.9 + 0.08 * hashf(dx, y, 4);
                let c = if dx == 0 || dx == 5 {
                    INK
                } else {
                    dither(&PARCHMENT, v, rx + dx, y)
                };
                img.set(rx + dx, y, c);
            }
            if (y - SCROLL_Y) % 7 == 3 {
                img.set(rx + if flip { 1 } else { 4 }, y, PARCHMENT[1]);
            }
        }
        for dx in 0..6 {
            img.set(rx + dx, SCROLL_Y - 3, INK);
            img.set(rx + dx, SCROLL_Y + SCROLL_H + 2, INK);
        }
        // Wooden knobs.
        for (ky, kh) in [(SCROLL_Y - 5, 2), (SCROLL_Y + SCROLL_H + 3, 2)] {
            for yy in ky..ky + kh {
                for dx in 1..5 {
                    img.set(
                        rx + dx,
                        yy,
                        dither(&WOOD, 0.4 + 0.15 * dx as f32 / 4.0, rx + dx, yy),
                    );
                }
            }
        }
    }
}

fn compass_face(img: &mut Img) {
    let (cx, cy, r) = COMPASS;
    for y in -r - 1..=r + 1 {
        for x in -r - 1..=r + 1 {
            let d = ((x * x + y * y) as f32).sqrt();
            let px = cx + x;
            let py = cy + y;
            if d <= r as f32 - 3.0 {
                let n = fbm(px as f32 * 0.2, py as f32 * 0.2, 2, 8);
                let v = 0.3 + 0.35 * n - d / r as f32 * 0.2;
                img.set(px, py, dither(&PARCHMENT, v, px, py));
            } else if d <= r as f32 {
                // Brass ring lit from the top-left.
                let a = (-(x + y) as f32 / (d + 0.01)) * 0.5 + 0.5;
                let v = 0.25 + 0.6 * a + if d > r as f32 - 1.0 { -0.15 } else { 0.0 };
                img.set(px, py, dither(&GOLD, v, px, py));
            } else if d <= r as f32 + 1.0 {
                img.set(px, py, INK);
            }
        }
    }
    for k in 0..16 {
        let a = k as f32 * std::f32::consts::TAU / 16.0;
        let len = if k % 4 == 0 {
            4.0
        } else if k % 2 == 0 {
            2.5
        } else {
            1.5
        };
        let r0 = r as f32 - 4.0;
        for s in 0..len as i32 {
            let rr = r0 - s as f32;
            img.set(
                cx + (a.sin() * rr).round() as i32,
                cy - (a.cos() * rr).round() as i32,
                PARCHMENT[0],
            );
        }
    }
    draw_text(img, cx - 2, cy - r + 6, "N", BLOOD[3]);
    draw_text(img, cx - 2, cy + r - 13, "S", SCROLL_INK);
    draw_text(img, cx - r + 6, cy - 3, "W", SCROLL_INK);
    draw_text(img, cx + r - 10, cy - 3, "E", SCROLL_INK);
}

fn engrave(img: &mut Img, x: i32, y: i32, text: &str) {
    font::draw_text_engraved(img, x, y, text, FRAME[0], FRAME[5], FRAME[2]);
}

fn build_frame(gargoyle: &Img) -> (Img, (i32, i32)) {
    let mut f = Img::new(SCREEN_W, SCREEN_H);
    frame_stone(&mut f);
    // Playfield recess.
    inset(&mut f, PF_X, PF_Y, PW, PH, false);
    for (x, y) in [
        (1, 1),
        (PF_X + PW + 1, 1),
        (1, PF_Y + PH + 1),
        (PF_X + PW + 1, PF_Y + PH + 1),
    ] {
        stud(&mut f, x + 1, y + 1);
    }
    parchment_scroll(&mut f);

    // Side panel.
    let gx = PANEL_X + (PANEL_W - gargoyle.w) / 2;
    let gy = 3;
    f.blit(gargoyle, gx, gy);
    inset(&mut f, PANEL_X + 2, 27, PANEL_W - 4, 48, true);
    engrave(&mut f, PANEL_X + 5, 30, "Level");
    engrave(&mut f, PANEL_X + 5, 41, "Score");
    engrave(&mut f, PANEL_X + 5, 63, "Best");
    inset(&mut f, PANEL_X + 2, 80, PANEL_W - 4, 12, true);
    for fx in FLASK_X {
        inset(&mut f, fx - 2, 97, 20, 30, true);
    }
    inset(&mut f, PANEL_X + 2, 133, PANEL_W - 4, 50, true);
    compass_face(&mut f);
    (f, (gx, gy))
}

// ---------------------------------------------------------------------------
// Dynamic HUD pieces.

fn draw_flask(img: &mut Img, x: i32, y: i32, frac: f32, ramp: &[Rgba], t: f32) {
    let (cx, cy, r) = (7.5f32, 17.5f32, 7.6f32);
    let level = 25.0 - frac.clamp(0.0, 1.0) * 22.0;
    for py in 0..26 {
        for px in 0..16 {
            let fx = px as f32 + 0.5;
            let fy = py as f32 + 0.5;
            let in_body = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt() <= r;
            let in_neck = (5..=10).contains(&px) && (2..=11).contains(&py);
            let in_cork = (5..=10).contains(&px) && py < 3;
            let (sx, sy) = (x + px, y + py);
            if in_cork {
                img.set(
                    sx,
                    sy,
                    if px == 5 || px == 10 || py == 0 {
                        INK
                    } else {
                        WOOD[3]
                    },
                );
                continue;
            }
            if !(in_body || in_neck) {
                continue;
            }
            let body_edge = in_body && ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt() > r - 1.0;
            let neck_edge = in_neck && !in_body && (px == 5 || px == 10);
            if body_edge && !(in_neck && py < 12) || neck_edge {
                img.set(sx, sy, INK);
                continue;
            }
            let wave = (t * 3.0 + px as f32 * 0.7).sin() * 0.6;
            if fy > level + wave {
                let dxn = ((fx - cx) / r).abs();
                let v = 0.3 + 0.55 * (1.0 - dxn) + if fy < level + wave + 1.2 { 0.25 } else { 0.0 };
                img.set(sx, sy, dither(ramp, v, sx, sy));
            } else {
                img.set(sx, sy, dither(&[rgb(0x0a0b10), rgb(0x161a22)], 0.3, sx, sy));
            }
        }
    }
    // Glass gloss.
    for (gx, gy) in [(3, 14), (3, 15), (4, 13), (4, 16), (7, 4), (7, 5)] {
        img.blend(x + gx, y + gy, WHITE.with_alpha(170));
    }
}

fn draw_needle(img: &mut Img, dir: (f32, f32)) {
    let (cx, cy, r) = COMPASS;
    let len = r as f32 - 6.0;
    let (dx, dy) = dir;
    let tip = (cx as f32 + dx * len, cy as f32 + dy * len);
    let tail = (cx as f32 - dx * len * 0.7, cy as f32 - dy * len * 0.7);
    let (px, py) = (-dy, dx);
    for o in [-0.6f32, 0.0, 0.6] {
        let ox = (px * o).round() as i32;
        let oy = (py * o).round() as i32;
        img.line(
            cx + ox,
            cy + oy,
            tail.0.round() as i32,
            tail.1.round() as i32,
            IRON[4],
        );
        img.line(
            cx + ox,
            cy + oy,
            tip.0.round() as i32,
            tip.1.round() as i32,
            BLOOD[4],
        );
    }
    img.set(tip.0.round() as i32, tip.1.round() as i32, BLOOD[5]);
    img.set(cx, cy, GOLD[6]);
    img.set(cx + 1, cy, GOLD[3]);
    img.set(cx, cy + 1, GOLD[3]);
}

/// Darken the playfield behind overlay text with a dithered veil.
fn veil(img: &mut Img, y: i32, h: i32) {
    img.checker_shade(0, y, PW, h, INK);
    img.shade_rect(0, y, PW, h, 0.6);
}

fn banner(img: &mut Img, y: i32, text: &str, ramp: &[Rgba]) {
    let w = text_width(text) * 2;
    veil(img, y - 4, 26);
    draw_text_big(img, (PW - w) / 2, y, text, 2, ramp, INK);
}

/// A touch-screen button: a dark plate with a bevelled stone rim.
fn button(img: &mut Img, h: &Hotspot) {
    img.checker_shade(h.x, h.y, h.w, h.h, INK);
    img.shade_rect(h.x, h.y, h.w, h.h, 0.5);
    img.rect(h.x, h.y, h.w, h.h, FRAME[2]);
    img.hline(h.x + 1, h.x + h.w - 2, h.y + 1, FRAME[6]);
    img.vline(h.x + 1, h.y + 1, h.y + h.h - 2, FRAME[6]);
    let tw = text_width(&h.label);
    draw_text_shadow(
        img,
        h.x + (h.w - tw) / 2,
        h.y + 4,
        &h.label,
        PARCHMENT[6],
        INK,
    );
}

/// "Tap" when playing by touch, "Click" otherwise.
fn tap(app: &App) -> &'static str {
    if app.touch { "Tap" } else { "Click" }
}

fn blink(t: f32) -> bool {
    (t * 2.2).fract() < 0.65
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    pub fn new() -> Renderer {
        let (garg, eyes) = sprites::gargoyle();
        let (frame, gargoyle_pos) = build_frame(&garg);
        Renderer {
            frame,
            dungeon: Dungeon::new(PW, PH),
            pf: Img::new(PW, PH),
            bricks: build_brick_cache(),
            balls: [
                sprites::ball(BallStyle::Orb),
                sprites::ball(BallStyle::Fire),
                sprites::ball(BallStyle::Ghost),
            ],
            bolt: sprites::bolt(),
            icons: Power::ALL.iter().map(|p| sprites::power_icon(*p)).collect(),
            ankh: sprites::ankh(),
            gargoyle_eyes: eyes,
            gargoyle_pos,
            sconce: sprites::torch_sconce(),
            lights: Vec::new(),
            shadow: vec![false; (PW * PH) as usize],
        }
    }

    fn icon(&self, p: Power) -> &Img {
        let i = Power::ALL.iter().position(|q| *q == p).unwrap_or(0);
        &self.icons[i]
    }

    pub fn draw(&mut self, out: &mut Img, app: &App, fx: &mut Fx) {
        out.px.copy_from_slice(&self.frame.px);
        let t = app.time;
        let game = app.active_game();
        self.draw_playfield(game, app, fx, t);
        let (sx, sy) = fx.shake_offset();
        for y in 0..PH {
            for x in 0..PW {
                let c = self.pf.get(x - sx, y - sy);
                out.set(PF_X + x, PF_Y + y, if c.a == 0 { BLACK } else { c });
            }
        }
        self.draw_scroll(out, app);
        self.draw_panel(out, app, fx, t);
        for h in app.hotspots() {
            if !h.label.is_empty() {
                button(out, &h);
            }
        }
    }

    fn draw_playfield(&mut self, game: &Game, app: &App, fx: &Fx, t: f32) {
        let fire = game.effects.fire > 0.0;
        let ghost = game.effects.phantom > 0.0;

        // Lights.
        self.lights.clear();
        let nb = game.balls.len().max(1) as f32;
        for b in &game.balls {
            self.lights.push(Light {
                x: b.pos.x,
                y: b.pos.y,
                radius: if fire { 54.0 } else { 46.0 },
                intensity: if fire { 1.1 } else { 0.95 } / nb.sqrt(),
                cool: !fire,
            });
        }
        for d in &game.drops {
            self.lights.push(Light {
                x: d.pos.x,
                y: d.pos.y,
                radius: 14.0,
                intensity: 0.35,
                cool: d.power.good(),
            });
        }
        if game.effects.crossbow > 0.0 {
            for b in &game.bolts {
                self.lights.push(Light {
                    x: b.pos.x,
                    y: b.pos.y,
                    radius: 8.0,
                    intensity: 0.4,
                    cool: false,
                });
            }
        }
        // The gem on the paddle glows faintly.
        self.lights.push(Light {
            x: game.paddle.x,
            y: PADDLE_Y,
            radius: 26.0,
            intensity: 0.25,
            cool: true,
        });
        fx.lights(&mut self.lights);
        let brightness = if app.showing_demo() { 0.8 } else { 1.0 };
        self.dungeon
            .render(&mut self.pf, 0, 0, t, &self.lights, brightness);

        // Torches.
        for (k, torch) in self.dungeon.torches.iter().enumerate() {
            if torch.scale > 0.6 {
                self.pf.blit(&self.sconce, torch.x - 3, torch.y);
                sprites::draw_flame(
                    &mut self.pf,
                    torch.x,
                    torch.y - 1,
                    t,
                    torch.seed,
                    torch.scale.min(1.0),
                );
            } else {
                self.pf.set(torch.x, torch.y + 1, WOOD[2]);
                self.pf.set(torch.x, torch.y + 2, WOOD[1]);
                sprites::draw_flame(
                    &mut self.pf,
                    torch.x,
                    torch.y,
                    t,
                    torch.seed,
                    torch.scale.max(0.4),
                );
            }
            let _ = k;
        }

        // Brick shadows (cast down-right onto the wall), then bricks.
        self.shadow.fill(false);
        for (idx, b) in game.bricks.iter().enumerate() {
            if b.is_some() {
                let (l, tp, _, _) = cell_rect(idx);
                for y in tp as i32 + 2..tp as i32 + 2 + BRICK_H as i32 {
                    for x in l as i32 + 2..l as i32 + 2 + BRICK_W as i32 {
                        if x < PW && y < PH {
                            self.shadow[(y * PW + x) as usize] = true;
                        }
                    }
                }
            }
        }
        for (i, s) in self.shadow.iter().enumerate() {
            if *s {
                self.pf.px[i] = self.pf.px[i].scale(0.45);
            }
        }
        for (idx, b) in game.bricks.iter().enumerate() {
            let Some(b) = b else { continue };
            let (l, tp, _, _) = cell_rect(idx);
            let variant = hash2(idx as i32, 0, 5) % 3;
            let frame = (t * 8.0) as usize + idx;
            let slot = brick_slot(b.kind, b.kind.max_hp() - b.hp, variant, frame);
            let (x, y) = (l as i32, tp as i32);
            self.pf.blit(&self.bricks[slot], x, y);
            if b.flash < 0.09 {
                let a = ((1.0 - b.flash / 0.09) * 200.0) as u8;
                let c = if b.kind == BrickKind::Iron {
                    GOLD[6]
                } else {
                    WHITE
                };
                self.pf.fill_rect(
                    x,
                    y,
                    BRICK_W as i32 - 1,
                    BRICK_H as i32 - 1,
                    c.with_alpha(a),
                );
            }
        }

        // Falling power-ups.
        for d in &game.drops {
            let bob = ((d.age * 6.0).sin() * 1.2).round() as i32;
            let (x, y) = (d.pos.x.round() as i32 - 5, d.pos.y.round() as i32 - 5 + bob);
            self.pf.shade_rect(x + 2, y + 3, 10, 10, 0.5);
            let icon = self.icon(d.power).clone();
            self.pf.blit(&icon, x, y);
            if (d.age * 3.0).fract() < 0.15 {
                let c = if d.power.good() { MANA[5] } else { BLOOD[5] };
                self.pf.add(
                    x + 1 + (hash2((d.age * 3.0) as i32, 1, 3) % 8) as i32,
                    y + 1,
                    c,
                    1.0,
                );
            }
        }

        for b in &game.bolts {
            self.pf.blit(
                &self.bolt,
                b.pos.x.round() as i32 - 1,
                b.pos.y.round() as i32 - 2,
            );
        }

        // Paddle (flickers away while dying).
        let dying = matches!(game.phase, Phase::Dying(_) | Phase::GameOver);
        if !dying || (t * 12.0).fract() < 0.3 && matches!(game.phase, Phase::Dying(d) if d > 0.8) {
            let w = game.paddle.width.round() as i32;
            let look = PaddleLook {
                crossbow: game.effects.crossbow > 0.0,
                grab: game.effects.grab,
                time: t,
            };
            let pimg = sprites::paddle(w, &look);
            let squash = (fx.paddle_squash * 1.5).round() as i32;
            let px = (game.paddle.x - w as f32 / 2.0).round() as i32;
            // Soft shadow on the floor beneath the plank.
            self.pf
                .checker_shade(px + 2, PADDLE_Y as i32 + 7, w, 2, INK);
            self.pf.blit(&pimg, px, PADDLE_Y as i32 - 3 + squash.min(1));
        }

        // Orbs.
        let style = if fire {
            1
        } else if ghost {
            2
        } else {
            0
        };
        let halo = if fire {
            FIRE[4]
        } else if ghost {
            GHOST[2]
        } else {
            ORB[1]
        };
        for b in &game.balls {
            let (x, y) = (
                (b.pos.x - 2.5).round() as i32,
                (b.pos.y - 2.5).round() as i32,
            );
            for (hx, hy) in [
                (1, -1),
                (2, -1),
                (3, -1),
                (-1, 1),
                (-1, 2),
                (-1, 3),
                (5, 1),
                (5, 2),
                (5, 3),
                (1, 5),
                (2, 5),
                (3, 5),
            ] {
                self.pf.add(x + hx, y + hy, halo, 0.45);
            }
            if ghost {
                self.pf.blit_ex(&self.balls[2], x, y, 0.75, false);
            } else {
                self.pf.blit(&self.balls[style], x, y);
            }
        }

        // Particles.
        for p in &fx.particles {
            let (x, y) = (p.pos.x.round() as i32, p.pos.y.round() as i32);
            let k = (p.life / p.max_life).clamp(0.0, 1.0);
            match p.kind {
                PKind::Debris => {
                    self.pf.set(x, y, p.color);
                    if k > 0.5 {
                        self.pf.set(x + 1, y, p.color.scale(0.7));
                    }
                }
                PKind::Spark | PKind::Ember => self.pf.add(x, y, p.color, 0.5 + k),
                PKind::Smoke => self.pf.blend(x, y, p.color.with_alpha((k * 150.0) as u8)),
                PKind::Magic => {
                    self.pf.add(x, y, p.color, k * 1.5);
                    if k > 0.6 {
                        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                            self.pf.add(x + dx, y + dy, p.color, k * 0.4);
                        }
                    }
                }
            }
        }
        for ft in fx.texts.iter().filter(|_| !app.showing_demo()) {
            let a = (ft.life.min(1.0) * 255.0) as u8;
            let c = if ft.good { GOLD[6] } else { BLOOD[5] };
            let w = text_width(&ft.text);
            let x = (ft.pos.x as i32 - w / 2).clamp(1, PW - w - 2);
            draw_text(
                &mut self.pf,
                x + 1,
                ft.pos.y as i32 - 12,
                &ft.text,
                INK.with_alpha(a),
            );
            draw_text(
                &mut self.pf,
                x,
                ft.pos.y as i32 - 13,
                &ft.text,
                c.with_alpha(a),
            );
        }

        self.draw_overlays(game, app, t);

        if fx.screen_flash > 0.0 {
            let a = (fx.screen_flash.min(1.0) * 180.0) as u8;
            self.pf.fill_rect(0, 0, PW, PH, rgb(0xfff0d0).with_alpha(a));
        }
    }

    fn draw_overlays(&mut self, game: &Game, app: &App, t: f32) {
        let pf = &mut self.pf;
        match app.screen {
            Screen::Title => {
                veil(pf, 14, 66);
                draw_text_big(
                    pf,
                    (PW - text_width("STYGIAN") * 3) / 2,
                    18,
                    "STYGIAN",
                    3,
                    &GOLD,
                    INK,
                );
                draw_text_big(
                    pf,
                    (PW - text_width("J-BALL") * 3) / 2,
                    48,
                    "J-BALL",
                    3,
                    &GOLD,
                    INK,
                );
                draw_text_shadow(
                    pf,
                    (PW - text_width("a DX-Ball tribute")) / 2,
                    84,
                    "a DX-Ball tribute",
                    PARCHMENT[5],
                    INK,
                );
                veil(pf, 104, 58);
                if blink(t) {
                    let msg = format!("{} to begin thy descent", tap(app));
                    draw_text_centered(pf, PW / 2, 109, &msg, GOLD[6]);
                }
                // By touch, these are buttons (drawn over the whole frame later).
                if !app.touch {
                    draw_text_centered(pf, PW / 2, 124, "L - Tome of Lore", PARCHMENT[5]);
                    draw_text_centered(pf, PW / 2, 135, "H - Hall of Heroes", PARCHMENT[5]);
                }
                // A browser tab can't be closed from inside the game.
                if !cfg!(target_arch = "wasm32") && !app.touch {
                    draw_text_centered(pf, PW / 2, 146, "Esc - Flee", PARCHMENT[5]);
                }
                let best = app.scores.first().map(|s| s.score).unwrap_or(0);
                if best > 0 {
                    draw_text_shadow(pf, 4, PH - 11, &format!("Best: {best}"), GOLD[4], INK);
                }
            }
            Screen::Lore => {
                pf.shade_rect(0, 0, PW, PH, 0.18);
                draw_text_big(
                    pf,
                    (PW - text_width("TOME OF LORE") * 2) / 2,
                    4,
                    "TOME OF LORE",
                    2,
                    &GOLD,
                    INK,
                );
                let desc = |p: Power| match p {
                    Power::Expand => "Widens the plank",
                    Power::Shrink => "Narrows the plank",
                    Power::Split => "Three orbs",
                    Power::Legion => "Eight orbs",
                    Power::Fire => "Orb blasts stone",
                    Power::Phantom => "Orb passes stone",
                    Power::Haste => "Orb speeds up",
                    Power::Slow => "Orb slows down",
                    Power::Grab => "Plank holds orb",
                    Power::Crossbow if app.touch => "Tap to shoot",
                    Power::Crossbow => "Click to shoot",
                    Power::Life => "An extra life",
                    Power::Skull => "Lose a life!",
                    Power::Quake => "Walls drop down",
                    Power::Zap => "All walls weaken",
                    Power::Warp => "Skip the level",
                };
                for (i, p) in Power::ALL.iter().enumerate() {
                    let col = i as i32 / 8;
                    let row = i as i32 % 8;
                    let (x, y) = (8 + col * 142, 26 + row * 19);
                    let icon = self.icons[i].clone();
                    pf.blit(&icon, x, y + 2);
                    let nc = if p.good() { PARCHMENT[6] } else { BLOOD[5] };
                    draw_text(pf, x + 14, y, p.name(), nc);
                    draw_text(pf, x + 14, y + 9, desc(*p), FRAME[6]);
                }
                let (move_hint, act_hint) = if app.touch {
                    ("Drag: move the plank", "Tap: act  Right panel: pause")
                } else {
                    ("Mouse/arrows: move", "Click/space: act  P: pause")
                };
                draw_text(pf, 150, 26 + 7 * 19 + 1, move_hint, FRAME[7]);
                draw_text(pf, 150, 26 + 7 * 19 + 10, act_hint, FRAME[7]);
                if !app.touch {
                    draw_text(pf, 8, PH - 10, "M music   S sound   F fullscreen", FRAME[7]);
                }
            }
            Screen::Scores => {
                pf.shade_rect(0, 0, PW, PH, 0.18);
                draw_text_big(
                    pf,
                    (PW - text_width("HALL OF HEROES") * 2) / 2,
                    6,
                    "HALL OF HEROES",
                    2,
                    &GOLD,
                    INK,
                );
                if app.scores.is_empty() {
                    draw_text_centered(pf, PW / 2, 80, "No deeds are yet recorded.", PARCHMENT[5]);
                }
                for (i, s) in app.scores.iter().enumerate().take(MAX_SCORES) {
                    let y = 34 + i as i32 * 16;
                    let hl = app.highlight == Some(i);
                    let c = if hl {
                        if blink(t) { GOLD[6] } else { GOLD[4] }
                    } else {
                        PARCHMENT[5]
                    };
                    draw_text_shadow(pf, 24, y, &format!("{}.", i + 1), c, INK);
                    draw_text_shadow(pf, 40, y, &s.name, c, INK);
                    draw_text_shadow(
                        pf,
                        150,
                        y,
                        &format!("Lvl {}", roman(s.level.max(1))),
                        c,
                        INK,
                    );
                    let sc = s.score.to_string();
                    draw_text_shadow(pf, PW - 24 - text_width(&sc), y, &sc, c, INK);
                }
                let msg = format!("{} to return", tap(app));
                draw_text_centered(pf, PW / 2, PH - 12, &msg, FRAME[7]);
            }
            Screen::Paused => banner(pf, 80, "PAUSED", &GOLD),
            Screen::EnterName => {
                veil(pf, 50, 92);
                draw_text_big(
                    pf,
                    (PW - text_width("THOU ART WORTHY") * 2) / 2,
                    56,
                    "THOU ART WORTHY",
                    2,
                    &GOLD,
                    INK,
                );
                draw_text_centered(
                    pf,
                    PW / 2,
                    84,
                    &format!("Score: {}", game.score),
                    PARCHMENT[6],
                );
                draw_text_centered(pf, PW / 2, 98, "Inscribe thy name:", PARCHMENT[5]);
                let name = format!("{}{}", app.name, if blink(t) { "_" } else { " " });
                let w = text_width(&name).max(60);
                pf.fill_rect(PW / 2 - w / 2 - 4, 110, w + 8, 13, PARCHMENT[4]);
                pf.rect(PW / 2 - w / 2 - 5, 109, w + 10, 15, INK);
                draw_text(
                    pf,
                    PW / 2 - text_width(&app.name) / 2,
                    112,
                    &name,
                    SCROLL_INK,
                );
                let msg = if app.touch {
                    "Tap to inscribe thy name"
                } else {
                    "Enter to confirm"
                };
                draw_text_centered(pf, PW / 2, 128, msg, FRAME[7]);
            }
            Screen::Ending(_) | Screen::Playing => match game.phase {
                Phase::Cleared(_) => banner(pf, 84, "THE WAY IS CLEAR", &GOLD),
                Phase::GameOver => {
                    veil(pf, 98, 26);
                    banner(pf, 76, "THY QUEST", &BLOOD);
                    draw_text_big(
                        pf,
                        (PW - text_width("HAS ENDED") * 2) / 2,
                        100,
                        "HAS ENDED",
                        2,
                        &BLOOD,
                        INK,
                    );
                }
                Phase::Victory => {
                    banner(pf, 70, "VICTORY!", &GOLD);
                    veil(pf, 98, 26);
                    draw_text_centered(
                        pf,
                        PW / 2,
                        100,
                        "Thou hast conquered the Abyss.",
                        PARCHMENT[6],
                    );
                    if blink(t) {
                        let msg = if app.touch {
                            "Tap to descend again."
                        } else {
                            "Click to descend again. Esc to rest."
                        };
                        draw_text_centered(pf, PW / 2, 111, msg, GOLD[6]);
                    }
                }
                Phase::Play => {
                    if game.level_time < 2.5 {
                        let title = format!("Level {}", roman(game.level as u32 + 1));
                        let a = ((2.5 - game.level_time).min(1.0) * 255.0) as u8;
                        let w = text_width(&title) * 2;
                        draw_text_big(pf, (PW - w) / 2, 116, &title, 2, &GOLD, INK);
                        let name = game.level_name();
                        let nw = text_width(name);
                        draw_text(pf, (PW - nw) / 2 + 1, 136, name, INK.with_alpha(a));
                        draw_text(pf, (PW - nw) / 2, 135, name, PARCHMENT[6].with_alpha(a));
                    }
                    if game.serving() && game.level_time > 0.4 && blink(t) {
                        let msg = format!("{} to loose the orb", tap(app));
                        let w = text_width(&msg);
                        pf.checker_shade(PW / 2 - w / 2 - 3, 154, w + 6, 11, INK);
                        draw_text_shadow(pf, PW / 2 - w / 2, 156, &msg, PARCHMENT[6], INK);
                    }
                }
                _ => {}
            },
        }
    }

    fn draw_scroll(&self, out: &mut Img, app: &App) {
        let max_w = SCROLL_W - 26;
        let mut lines: Vec<(String, bool)> = Vec::new();
        let n = app.log.len();
        for (i, (msg, age)) in app.log.iter().enumerate() {
            let newest = i + 1 == n;
            // Typewriter reveal for the newest message.
            let shown: String = if newest {
                msg.chars().take((age * 90.0) as usize + 1).collect()
            } else {
                msg.clone()
            };
            for l in font::wrap(&shown, max_w) {
                lines.push((l, newest));
            }
        }
        let start = lines.len().saturating_sub(3);
        for (k, (l, newest)) in lines[start..].iter().enumerate() {
            let c = if *newest {
                SCROLL_INK
            } else {
                SCROLL_INK_FADED
            };
            draw_text(out, SCROLL_X + 13, SCROLL_Y + 3 + k as i32 * LINE_H, l, c);
        }
    }

    fn draw_panel(&self, out: &mut Img, app: &App, fx: &Fx, t: f32) {
        let game = app.active_game();
        let right = PANEL_X + PANEL_W - 5;

        // Gargoyle eyes: smoulder normally, blaze when the orb is lost.
        let idle = 0.25 + 0.1 * (t * 1.7).sin();
        let glow = idle.max(fx.eye_glow);
        let (gx, gy) = self.gargoyle_pos;
        for &(ex, ey) in &self.gargoyle_eyes {
            out.set(gx + ex, gy + ey, dither(&BLOOD, glow, ex, ey));
            if glow > 0.6 {
                out.add(gx + ex, gy + ey - 1, BLOOD[4], glow - 0.6);
            }
        }

        let lvl = roman(game.level as u32 + 1 + game.cycle * crate::levels::LEVELS.len() as u32);
        draw_text_shadow(out, right - text_width(&lvl), 30, &lvl, ENGRAVED_GOLD, INK);
        let score = game.score.to_string();
        draw_text_shadow(out, right - text_width(&score), 52, &score, GOLD[6], INK);
        let best = app
            .scores
            .first()
            .map(|s| s.score)
            .unwrap_or(0)
            .max(if app.showing_demo() { 0 } else { game.score });
        let best = best.to_string();
        draw_text_shadow(out, right - text_width(&best), 63, &best, GOLD[3], INK);

        // Ankhs of life.
        let lives = game.lives.min(MAX_LIVES) as i32;
        let total_w = lives * 8 - 1;
        let x0 = PANEL_X + PANEL_W / 2 - total_w / 2;
        for i in 0..lives {
            out.blit(&self.ankh, x0 + i * 8 - 1, 81);
        }

        // Red flask: health, full at the starting lives. Blue flask: remaining magic.
        // Green flask: the orb's speed.
        let health = game.lives as f32 / START_LIVES as f32;
        draw_flask(out, FLASK_X[0], 99, health, &BLOOD, t);
        let lives_txt = game.lives.to_string();
        let lx = FLASK_X[0] + 8 - text_width(&lives_txt) / 2;
        draw_text_shadow(out, lx, 112, &lives_txt, WHITE, INK);
        let e = &game.effects;
        let mana = (e.fire / FIRE_TIME)
            .max(e.phantom / PHANTOM_TIME)
            .max(e.crossbow / CROSSBOW_TIME);
        draw_flask(out, FLASK_X[1], 99, mana, &MANA, t + 1.3);
        let base = game.base_speed() * 0.8;
        let speed_frac = (game.speed - base) / (SPEED_MAX - base);
        draw_flask(out, FLASK_X[2], 99, speed_frac, &VERDANT, t + 2.6);

        // Active effects with timers.
        let mut effects: Vec<(Power, f32)> = Vec::new();
        if e.fire > 0.0 {
            effects.push((Power::Fire, e.fire / FIRE_TIME));
        }
        if e.phantom > 0.0 {
            effects.push((Power::Phantom, e.phantom / PHANTOM_TIME));
        }
        if e.crossbow > 0.0 {
            effects.push((Power::Crossbow, e.crossbow / CROSSBOW_TIME));
        }
        if e.grab {
            effects.push((Power::Grab, 1.0));
        }
        if effects.is_empty() {
            let msg = if game.balls.len() > 1 {
                format!("{} orbs", game.balls.len())
            } else {
                "No magic".into()
            };
            draw_text_centered(out, PANEL_X + PANEL_W / 2, 154, &msg, FRAME[4]);
        }
        for (k, (p, frac)) in effects.iter().enumerate() {
            let y = 135 + k as i32 * 12;
            out.blit(self.icon(*p), PANEL_X + 4, y);
            let (bx, bw) = (PANEL_X + 17, PANEL_W - 22);
            out.fill_rect(bx, y + 3, bw, 4, INK);
            let fill = (bw as f32 * frac).round() as i32;
            let ramp: &[Rgba] = if *p == Power::Grab {
                &MOSS
            } else if *p == Power::Fire {
                &FIRE
            } else {
                &MANA
            };
            for x in 0..fill {
                for yy in 0..4 {
                    let v = 0.4 + 0.5 * (1.0 - yy as f32 / 3.0);
                    out.set(bx + x, y + 3 + yy, dither(ramp, v, x, yy));
                }
            }
        }
        if game.balls.len() > 1 && !effects.is_empty() && effects.len() < 4 {
            let msg = format!("{} orbs", game.balls.len());
            draw_text_centered(
                out,
                PANEL_X + PANEL_W / 2,
                135 + effects.len() as i32 * 12 + 1,
                &msg,
                FRAME[5],
            );
        }

        // Compass needle follows the first free orb.
        let dir = game
            .balls
            .iter()
            .find(|b| b.stuck.is_none())
            .map(|b| (b.dir.x, b.dir.y))
            .unwrap_or_else(|| {
                let a = (t * 1.3).sin() * 0.25;
                (a.sin(), -a.cos())
            });
        draw_needle(out, dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roman_numerals() {
        assert_eq!(roman(1), "I");
        assert_eq!(roman(4), "IV");
        assert_eq!(roman(9), "IX");
        assert_eq!(roman(14), "XIV");
        assert_eq!(roman(40), "XL");
    }

    #[test]
    fn renders_every_screen_without_panicking() {
        let mut r = Renderer::new();
        let mut fx = Fx::new();
        let mut out = Img::new(SCREEN_W, SCREEN_H);
        let mut app = App::new(None, 5);
        for touch in [false, true] {
            app.touch = touch;
            for screen in [
                Screen::Title,
                Screen::Lore,
                Screen::Scores,
                Screen::Playing,
                Screen::Paused,
                Screen::EnterName,
            ] {
                app.screen = screen;
                r.draw(&mut out, &app, &mut fx);
            }
        }
        // The playfield is not just black.
        let lit = (0..PH)
            .filter(|y| out.get(PF_X + 140, PF_Y + y).luma() > 0.05)
            .count();
        assert!(lit > 10);
    }
}
