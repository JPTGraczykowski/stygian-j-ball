//! The playfield backdrop: a first-person dungeon corridor, ray-cast once at startup
//! (stone walls, flagstone floor, beamed ceiling, a dark archway in the distance) and
//! re-lit every frame by flickering torches and the light of the orbs, then quantised
//! to the palette with ordered dithering.

use super::canvas::{Img, bayer, dither};
use super::palette::{BLACK, WALL_COOL, WALL_WARM, WOOD};
use crate::rng::{fbm, hashf};

const FOCAL: f32 = 1.25;
const HALF_W: f32 = 1.7;
const FLOOR_Y: f32 = -1.0;
const CEIL_Y: f32 = 1.1;
const END_Z: f32 = 11.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Material {
    Stone,
    Wood,
    Void,
}

pub struct Torch {
    /// Screen position of the flame base, in playfield pixels.
    pub x: i32,
    pub y: i32,
    pub scale: f32,
    pub seed: u32,
}

/// A dynamic point light in playfield pixel space.
#[derive(Clone, Copy)]
pub struct Light {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub intensity: f32,
    /// Cold (magical) light rather than warm firelight.
    pub cool: bool,
}

pub struct Dungeon {
    pub w: i32,
    pub h: i32,
    albedo: Vec<f32>,
    material: Vec<Material>,
    ambient: Vec<f32>,
    torch_maps: Vec<Vec<f32>>,
    pub torches: Vec<Torch>,
    warm: Vec<f32>,
    cool: Vec<f32>,
}

fn stone_blocks(u: f32, v: f32, bw: f32, bh: f32, seed: u32) -> f32 {
    let row = (v / bh).floor();
    let off = if (row as i32) & 1 == 0 { 0.0 } else { bw * 0.5 };
    let col = ((u + off) / bw).floor();
    let fu = (u + off) / bw - col;
    let fv = v / bh - row;
    let mortar_u = 0.05 * 0.62 / bw;
    if fu < mortar_u || fv < 0.08 {
        return 0.1;
    }
    let base = 0.5 + 0.25 * hashf(col as i32, row as i32, seed);
    let grain = (fbm(u * 13.0, v * 13.0, 3, seed + 1) - 0.5) * 0.35;
    let bevel = if fv > 0.86 {
        0.12
    } else if fv < 0.16 {
        -0.1
    } else {
        0.0
    };
    let edge = if fu > 0.95 { -0.08 } else { 0.0 };
    (base + grain + bevel + edge).clamp(0.0, 1.0)
}

/// Cast a ray through screen point (sx, sy); returns (albedo, material, hit point).
fn cast(sx: f32, sy: f32) -> (f32, Material, [f32; 3]) {
    let mut t = END_Z / FOCAL;
    let mut surface = 0;
    if sx.abs() > 1e-6 {
        let ts = HALF_W / sx.abs();
        if ts < t {
            t = ts;
            surface = 1;
        }
    }
    if sy < -1e-6 {
        let tf = FLOOR_Y / sy;
        if tf < t {
            t = tf;
            surface = 2;
        }
    } else if sy > 1e-6 {
        let tc = CEIL_Y / sy;
        if tc < t {
            t = tc;
            surface = 3;
        }
    }
    let p = [sx * t, sy * t, FOCAL * t];
    let (a, m) = match surface {
        1 => (stone_blocks(p[2], p[1], 0.62, 0.32, 11), Material::Stone),
        2 => {
            let a = stone_blocks(p[0] + 7.0, p[2], 0.75, 0.6, 23) * 0.8;
            (a, Material::Stone)
        }
        3 => {
            let f = (p[2] / 0.9).fract();
            if f < 0.16 {
                let g = fbm(p[0] * 3.0, p[2] * 20.0, 2, 5);
                let edge = if f < 0.025 || f > 0.135 { 0.45 } else { 1.0 };
                ((0.4 + 0.45 * g) * edge, Material::Wood)
            } else {
                (
                    stone_blocks(p[0] + 3.0, p[2], 0.5, 0.5, 31) * 0.6,
                    Material::Stone,
                )
            }
        }
        _ => {
            // End wall with a dark archway leading deeper into the abyss.
            let arch_top = 0.25;
            let in_arch = p[0].abs() < 0.55
                && (p[1] < arch_top
                    || (p[0] * p[0] + (p[1] - arch_top) * (p[1] - arch_top)) < 0.55 * 0.55);
            if in_arch {
                (0.0, Material::Void)
            } else {
                (
                    stone_blocks(p[0] + 5.0, p[1], 0.62, 0.32, 41),
                    Material::Stone,
                )
            }
        }
    };
    (a, m, p)
}

impl Dungeon {
    pub fn new(w: i32, h: i32) -> Dungeon {
        let n = (w * h) as usize;
        let mut albedo = vec![0.0; n];
        let mut material = vec![Material::Stone; n];
        let mut ambient = vec![0.0; n];
        let mut points = vec![[0.0f32; 3]; n];
        let half_h = h as f32 * 0.5;
        let to_screen = |px: f32, py: f32| ((px - w as f32 * 0.5) / half_h, (half_h - py) / half_h);
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                // 2x2 supersampling for the albedo; centre ray for geometry.
                let mut acc = 0.0;
                for (ox, oy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
                    let (sx, sy) = to_screen(x as f32 + ox, y as f32 + oy);
                    acc += cast(sx, sy).0;
                }
                let (sx, sy) = to_screen(x as f32 + 0.5, y as f32 + 0.5);
                let (_, m, p) = cast(sx, sy);
                albedo[i] = acc * 0.25;
                material[i] = m;
                points[i] = p;
                ambient[i] = 0.32 * (-0.2 * (p[2] - 1.0)).exp();
            }
        }

        let world_torches = [
            [-HALF_W + 0.06, -0.12, 2.2],
            [HALF_W - 0.06, -0.12, 2.2],
            [-HALF_W + 0.06, -0.12, 5.6],
            [HALF_W - 0.06, -0.12, 5.6],
        ];
        let mut torch_maps = Vec::new();
        let mut torches = Vec::new();
        for (k, t) in world_torches.iter().enumerate() {
            let mut map = vec![0.0; n];
            for i in 0..n {
                let p = points[i];
                let d2 = (p[0] - t[0]).powi(2) + (p[1] - t[1]).powi(2) + (p[2] - t[2]).powi(2);
                let fog = (-0.1 * p[2]).exp();
                map[i] = 1.05 / (1.0 + 2.2 * d2) * fog;
            }
            torch_maps.push(map);
            let sx = FOCAL * t[0] / t[2];
            let sy = FOCAL * t[1] / t[2];
            torches.push(Torch {
                x: (sx * half_h + w as f32 * 0.5).round() as i32,
                y: (half_h - sy * half_h).round() as i32,
                scale: 2.0 / t[2],
                seed: k as u32 * 17 + 3,
            });
        }

        Dungeon {
            w,
            h,
            albedo,
            material,
            ambient,
            torch_maps,
            torches,
            warm: vec![0.0; n],
            cool: vec![0.0; n],
        }
    }

    /// Flicker factor for torch `k` at time `t`.
    pub fn flicker(&self, k: usize, t: f32) -> f32 {
        let s = self.torches[k].seed as f32;
        0.82 + 0.1 * (t * 9.3 + s).sin()
            + 0.06 * (t * 23.7 + s * 2.1).sin()
            + 0.04 * (t * 3.1 + s).sin()
    }

    /// Render the lit corridor into `out` at (ox, oy).
    pub fn render(
        &mut self,
        out: &mut Img,
        ox: i32,
        oy: i32,
        t: f32,
        lights: &[Light],
        brightness: f32,
    ) {
        self.warm.fill(0.0);
        self.cool.fill(0.0);
        for l in lights {
            let buf = if l.cool {
                &mut self.cool
            } else {
                &mut self.warm
            };
            let r = l.radius;
            let x0 = ((l.x - r).floor() as i32).max(0);
            let x1 = ((l.x + r).ceil() as i32).min(self.w - 1);
            let y0 = ((l.y - r).floor() as i32).max(0);
            let y1 = ((l.y + r).ceil() as i32).min(self.h - 1);
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let d = ((x as f32 - l.x).powi(2) + (y as f32 - l.y).powi(2)).sqrt() / r;
                    if d < 1.0 {
                        let f = (1.0 - d) * (1.0 - d);
                        buf[(y * self.w + x) as usize] += f * l.intensity;
                    }
                }
            }
        }
        let flick: Vec<f32> = (0..self.torches.len())
            .map(|k| self.flicker(k, t))
            .collect();
        for y in 0..self.h {
            for x in 0..self.w {
                let i = (y * self.w + x) as usize;
                let c = match self.material[i] {
                    Material::Void => BLACK,
                    m => {
                        let mut warm = self.ambient[i] + self.warm[i];
                        for (k, map) in self.torch_maps.iter().enumerate() {
                            warm += map[i] * flick[k];
                        }
                        let cool = self.cool[i];
                        let total = warm + cool;
                        let v = (self.albedo[i] * total * brightness).min(1.0);
                        if m == Material::Wood {
                            dither(&WOOD[..5], v * 1.1, x, y)
                        } else if cool > 0.0 && cool / total > bayer(x + 1, y + 2) {
                            dither(&WALL_COOL, v, x, y)
                        } else {
                            dither(&WALL_WARM, v, x, y)
                        }
                    }
                };
                out.set(ox + x, oy + y, c);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corridor_geometry() {
        // Centre of screen looks down the corridor into the archway.
        let (_, m, p) = cast(0.0, 0.0);
        assert!(m == Material::Void);
        assert!((p[2] - END_Z).abs() < 1e-3);
        // Far left hits the left wall, bottom hits the floor.
        let (_, _, p) = cast(-1.5, 0.0);
        assert!((p[0] + HALF_W).abs() < 1e-3);
        let (_, _, p) = cast(0.0, -1.0);
        assert!((p[1] - FLOOR_Y).abs() < 1e-3);
    }

    #[test]
    fn torches_project_on_screen() {
        let d = Dungeon::new(96, 64);
        assert_eq!(d.torches.len(), 4);
        for t in &d.torches {
            assert!(t.x > 0 && t.x < 96 && t.y > 0 && t.y < 64);
        }
    }
}
