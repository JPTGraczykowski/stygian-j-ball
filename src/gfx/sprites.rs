//! All artwork, generated at startup: hand-placed string-art icons plus procedural
//! textures (noise, bevels, dithering). No external image files are needed.

use super::canvas::{CLEAR, Img, Rgba, dither, rgb};
use super::palette::*;
use crate::consts::{BRICK_H, BRICK_W};
use crate::game::{BrickKind, Power};
use crate::rng::{fbm, hash2, hashf};

/// Build an image from rows of characters, looking each up in `map`.
/// `.` and unknown characters are transparent.
pub fn art(rows: &[&str], map: &[(char, Rgba)]) -> Img {
    let h = rows.len() as i32;
    let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0) as i32;
    let mut img = Img::new(w, h);
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if let Some((_, c)) = map.iter().find(|(k, _)| *k == ch) {
                img.set(x as i32, y as i32, *c);
            }
        }
    }
    img
}

/// Build a left-right symmetric image from its left half.
pub fn art_mirrored(half: &[&str], map: &[(char, Rgba)]) -> Img {
    let rows: Vec<String> = half
        .iter()
        .map(|r| {
            let rev: String = r.chars().rev().collect();
            format!("{r}{rev}")
        })
        .collect();
    let refs: Vec<&str> = rows.iter().map(|s| s.as_str()).collect();
    art(&refs, map)
}

/// Add a one-pixel outline around all opaque pixels.
pub fn outlined(src: &Img, c: Rgba) -> Img {
    let mut out = Img::new(src.w + 2, src.h + 2);
    for y in -1..=src.h {
        for x in -1..=src.w {
            if src.get(x, y).a != 0 {
                continue;
            }
            let near = [(-1, 0), (1, 0), (0, -1), (0, 1)]
                .iter()
                .any(|(dx, dy)| src.get(x + dx, y + dy).a != 0);
            if near {
                out.set(x + 1, y + 1, c);
            }
        }
    }
    out.blit(src, 1, 1);
    out
}

// ---------------------------------------------------------------------------
// Bricks (18x8). Right column and bottom row are mortar, so neighbours tile.

const BW: i32 = BRICK_W as i32;
const BH: i32 = BRICK_H as i32;

fn bevel_brick(img: &mut Img, ramp: &[Rgba]) {
    let n = ramp.len();
    for x in 0..BW {
        img.set(x, BH - 1, INK);
    }
    for y in 0..BH {
        img.set(BW - 1, y, INK);
    }
    for x in 0..BW - 1 {
        img.set(x, 0, ramp[n - 1]);
        img.set(x, BH - 2, ramp[1]);
    }
    for y in 0..BH - 1 {
        img.set(0, y, ramp[n - 2]);
        img.set(BW - 2, y, ramp[1]);
    }
    img.set(0, BH - 2, ramp[2]);
    img.set(BW - 2, 0, ramp[2]);
}

fn noise_fill(img: &mut Img, ramp: &[Rgba], lo: f32, hi: f32, scale: f32, seed: u32) {
    for y in 0..BH {
        for x in 0..BW {
            let v = lo + (hi - lo) * fbm(x as f32 * scale, y as f32 * scale * 1.6, 3, seed);
            img.set(x, y, dither(ramp, v, x, y));
        }
    }
}

pub fn brick(kind: BrickKind, damage: u8, variant: u32) -> Img {
    let mut img = Img::new(BW, BH);
    let seed = variant * 977 + 13;
    match kind {
        BrickKind::Stone(m) => {
            let ramp: &[Rgba] = match m {
                0 => &GREY_STONE,
                1 => &SANDSTONE,
                2 => &GREY_STONE,
                3 => &CLAY,
                4 => &WOOD,
                _ => &SLATE,
            };
            noise_fill(&mut img, ramp, 0.25, 0.85, 0.45, seed);
            match m {
                1 => {
                    // Sandstone strata.
                    for x in 0..BW {
                        let y = 3 + ((x as f32 * 0.4 + variant as f32).sin() * 0.8).round() as i32;
                        img.set(x, y, ramp[1]);
                    }
                }
                2 => {
                    // Moss creeping over the top of grey stone.
                    for y in 0..BH {
                        for x in 0..BW {
                            let n = fbm(x as f32 * 0.5, y as f32 * 0.5, 2, seed + 5);
                            if n + (1.0 - y as f32 / 5.0) * 0.45 > 0.78 {
                                let v = 0.3 + 0.6 * n;
                                img.set(x, y, dither(&MOSS, v, x, y));
                            }
                        }
                    }
                }
                3 => {
                    // Small clay bricks with mortar.
                    for x in 0..BW {
                        img.set(x, 3, CLAY[0]);
                    }
                    for y in 0..3 {
                        img.set(8, y, CLAY[0]);
                    }
                    for y in 4..BH {
                        img.set(4, y, CLAY[0]);
                        img.set(13, y, CLAY[0]);
                    }
                }
                4 => {
                    // Wood planks: grain lines and nail heads.
                    for y in 0..BH {
                        for x in 0..BW {
                            let g = ((y as f32 * 2.2
                                + (x as f32 * 0.35 + variant as f32).sin() * 0.9)
                                as i32)
                                % 3;
                            if g == 0 && hashf(x, y, seed) > 0.35 {
                                img.set(x, y, WOOD[1]);
                            }
                        }
                    }
                    img.set(2, 3, IRON[5]);
                    img.set(BW - 4, 3, IRON[5]);
                }
                5 => {
                    for k in 0..3 {
                        let x0 = 3 + k * 5 + (variant as i32 % 3);
                        img.line(x0, 1, x0 + 3, BH - 2, SLATE[1]);
                    }
                }
                _ => {
                    // Pits in grey stone.
                    for k in 0..3 {
                        let h = hash2(k, variant as i32, 7);
                        img.set(2 + (h % 13) as i32, 1 + ((h >> 8) % 5) as i32, ramp[0]);
                    }
                }
            }
            bevel_brick(&mut img, ramp);
        }
        BrickKind::Bound => {
            noise_fill(&mut img, &WOOD, 0.35, 0.8, 0.5, seed);
            for y in 0..BH {
                for x in 0..BW {
                    if (y * 3 + (x / 5)) % 4 == 0 && hashf(x, y, seed) > 0.4 {
                        img.set(x, y, WOOD[2]);
                    }
                }
            }
            bevel_brick(&mut img, &WOOD);
            for sx in [3, 13] {
                for y in 0..BH - 1 {
                    let v = 0.35 + 0.5 * (1.0 - y as f32 / 7.0);
                    img.set(sx, y, dither(&IRON, v + 0.1, sx, y));
                    img.set(sx + 1, y, dither(&IRON, v - 0.15, sx + 1, y));
                }
                img.set(sx, 2, IRON[6]);
                img.set(sx, 5, IRON[6]);
            }
            if damage >= 1 {
                // One strap snapped, wood splintered.
                img.set(13, 3, WOOD[0]);
                img.set(14, 3, WOOD[0]);
                img.set(13, 4, CLEAR.with_alpha(0));
                crack(&mut img, seed + 1, WOOD[0], 2);
            }
        }
        BrickKind::Crystal => {
            for y in 0..BH {
                for x in 0..BW {
                    // Faceted: diagonal bands of light.
                    let facet = ((x + y * 2) / 5) % 3;
                    let v = 0.35
                        + facet as f32 * 0.18
                        + 0.15 * fbm(x as f32 * 0.3, y as f32 * 0.3, 2, seed);
                    img.set(x, y, dither(&CRYSTAL, v, x, y));
                }
            }
            bevel_brick(&mut img, &CRYSTAL);
            img.set(4, 2, CRYSTAL[6]);
            img.set(12, 4, CRYSTAL[6]);
            if damage >= 1 {
                crack(&mut img, seed + 3, CRYSTAL[6], 2);
            }
            if damage >= 2 {
                crack(&mut img, seed + 9, CRYSTAL[0], 3);
                crack(&mut img, seed + 11, CRYSTAL[6], 2);
            }
        }
        BrickKind::Iron => {
            for y in 0..BH {
                for x in 0..BW {
                    let shine = 1.0 - ((x as f32 - y as f32 * 2.0 - 4.0).abs() / 10.0).min(1.0);
                    let v = 0.3
                        + 0.25 * (1.0 - y as f32 / 7.0)
                        + 0.3 * shine
                        + 0.05 * hashf(x, y, seed);
                    img.set(x, y, dither(&IRON, v, x, y));
                }
            }
            bevel_brick(&mut img, &IRON);
            for (rx, ry) in [(2, 2), (BW - 4, 2), (2, BH - 4), (BW - 4, BH - 4)] {
                img.set(rx, ry, IRON[6]);
                img.set(rx + 1, ry + 1, IRON[1]);
            }
        }
        BrickKind::Rune => {
            // `damage` doubles as the glow animation frame here.
            noise_fill(&mut img, &GREY_STONE, 0.05, 0.45, 0.5, seed);
            bevel_brick(&mut img, &GREY_STONE[..4]);
            let glow = [0.55, 0.7, 0.85, 1.0, 0.85, 0.7][damage as usize % 6];
            let rune = ["#.#..", ".###.", "..#..", ".###.", "#.#.#"];
            // Faint halo first, then the rune itself.
            for (y, row) in rune.iter().enumerate() {
                for (x, c) in row.chars().enumerate() {
                    if c == '#' {
                        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                            let px = 6 + x as i32 + dx;
                            let py = 1 + y as i32 + dy;
                            if img.get(px, py) != INK {
                                img.set(px, py, dither(&RUNE_GLOW, glow * 0.35, px, py));
                            }
                        }
                    }
                }
            }
            for (y, row) in rune.iter().enumerate() {
                for (x, c) in row.chars().enumerate() {
                    if c == '#' {
                        let px = 6 + x as i32;
                        let py = 1 + y as i32;
                        img.set(px, py, dither(&RUNE_GLOW, glow, px, py));
                    }
                }
            }
        }
        BrickKind::Gold => {
            // A treasure chest: wooden body, gold bands and lock.
            noise_fill(&mut img, &WOOD, 0.35, 0.75, 0.5, seed);
            bevel_brick(&mut img, &WOOD);
            for x in 0..BW - 1 {
                img.set(x, 2, dither(&GOLD, 0.8, x, 2));
            }
            for y in 0..BH - 1 {
                for sx in [1, BW - 3] {
                    img.set(sx, y, dither(&GOLD, 0.7 - y as f32 * 0.05, sx, y));
                }
            }
            for (x, y) in [(8, 3), (9, 3), (8, 4), (9, 4)] {
                img.set(x, y, GOLD[5]);
            }
            img.set(8, 5, GOLD[2]);
            img.set(9, 5, GOLD[2]);
            img.set(8, 4, INK);
            img.set(4, 1, GOLD[6]);
        }
    }
    img
}

/// Draw a short random-walk crack.
fn crack(img: &mut Img, seed: u32, c: Rgba, len: i32) {
    let mut x = 3 + (hash2(1, 2, seed) % 12) as i32;
    let mut y = 1 + (hash2(3, 4, seed) % 4) as i32;
    for i in 0..len * 3 {
        img.set(x, y, c);
        let h = hash2(i, 9, seed);
        x += (h % 3) as i32 - 1;
        y += ((h >> 4) % 3) as i32 - 1;
        x = x.clamp(1, BW - 3);
        y = y.clamp(1, BH - 3);
    }
}

// ---------------------------------------------------------------------------
// Balls, bolts, paddle.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BallStyle {
    Orb,
    Fire,
    Ghost,
}

pub fn ball(style: BallStyle) -> Img {
    let ramp: &[Rgba] = match style {
        BallStyle::Orb => &ORB,
        BallStyle::Fire => &FIRE[2..],
        BallStyle::Ghost => &GHOST,
    };
    let mut img = Img::new(5, 5);
    for y in 0..5 {
        for x in 0..5 {
            if (x == 0 || x == 4) && (y == 0 || y == 4) {
                continue;
            }
            let d = ((x as f32 - 1.4).powi(2) + (y as f32 - 1.4).powi(2)).sqrt();
            let v = 1.0 - d / 4.2;
            img.set(x, y, dither(ramp, v, x, y));
        }
    }
    img
}

pub fn bolt() -> Img {
    art(
        &[".m.", "mMm", ".w.", ".w.", ".w.", "f.f"],
        &[('m', IRON[5]), ('M', IRON[6]), ('w', WOOD[4]), ('f', WHITE)],
    )
}

pub struct PaddleLook {
    pub crossbow: bool,
    pub grab: bool,
    pub time: f32,
}

/// The paddle: an iron-capped wooden beam with a glowing gem.
pub fn paddle(width: i32, look: &PaddleLook) -> Img {
    let w = width.max(8);
    let h = 6;
    let mut img = Img::new(w, h + 3);
    let oy = 3; // headroom for crossbow arms
    for y in 0..h {
        for x in 0..w {
            let cap = x < 4 || x >= w - 4;
            let edge = x == 0 || x == w - 1 || y == 0 || y == h - 1;
            let c = if edge {
                INK
            } else if cap {
                let v = 0.85 - y as f32 * 0.13 + if x == 1 || x == w - 2 { 0.1 } else { 0.0 };
                dither(&IRON, v, x, y)
            } else {
                let grain = ((x as f32 * 0.31).sin() + (y as f32 * 1.7)).floor() as i32 % 2 == 0;
                let v = 0.82 - y as f32 * 0.12 - if grain { 0.12 } else { 0.0 };
                dither(&WOOD, v, x, y)
            };
            img.set(x, oy + y, c);
        }
    }
    // Rivets on the caps.
    img.set(2, oy + 2, IRON[6]);
    img.set(w - 3, oy + 2, IRON[6]);
    // Central gem.
    let mid = w / 2;
    let pulse = 0.75 + 0.25 * (look.time * 5.0).sin();
    img.set(mid - 1, oy + 1, dither(&MANA, pulse, mid - 1, 1));
    img.set(mid, oy + 1, MANA[5]);
    img.set(mid - 1, oy + 2, MANA[3]);
    img.set(mid, oy + 2, dither(&MANA, pulse * 0.8, mid, 2));
    if look.grab {
        // Black pitch along the top, with a couple of drips.
        for x in 1..w - 1 {
            img.set(x, oy, rgb(0x16200e));
            if hashf(x, 0, 3) > 0.6 {
                img.set(x, oy + 1, rgb(0x2c3a1c));
            }
        }
    }
    if look.crossbow {
        for x0 in [2, w - 3] {
            img.set(x0, oy - 1, IRON[4]);
            img.set(x0 - 1, oy - 2, IRON[5]);
            img.set(x0 + 1, oy - 2, IRON[5]);
            img.set(x0, oy - 3, IRON[6]);
        }
    }
    img
}

// ---------------------------------------------------------------------------
// Power-up icons (10x10).

const POTION: [&str; 10] = [
    "....kk....",
    "...kcck...",
    "...kggk...",
    "..kgllgk..",
    ".kllllllk.",
    "klhllllldk",
    "klhllllldk",
    "kllllllddk",
    ".kllllddk.",
    "..kkkkkk..",
];

const RUNESTONE: [&str; 10] = [
    "..kkkkkk..",
    ".k443322k.",
    "k44333322k",
    "k43333321k",
    "k43333221k",
    "k33332221k",
    "k33322211k",
    "k32222111k",
    ".k221111k.",
    "..kkkkkk..",
];

fn potion(liquid: &[Rgba]) -> Img {
    art(
        &POTION,
        &[
            ('k', INK),
            ('c', WOOD[4]),
            ('g', rgb(0x8c9cab)),
            ('l', liquid[2]),
            ('h', liquid[liquid.len() - 1]),
            ('d', liquid[0]),
        ],
    )
}

fn runestone(stone: &[Rgba], symbol: &[&str], glow: Rgba) -> Img {
    let mut img = art(
        &RUNESTONE,
        &[
            ('k', INK),
            ('1', stone[0]),
            ('2', stone[1]),
            ('3', stone[2]),
            ('4', stone[3]),
        ],
    );
    for (y, row) in symbol.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            if c == '#' {
                img.set(2 + x as i32, 2 + y as i32, glow);
            }
        }
    }
    img
}

pub fn power_icon(p: Power) -> Img {
    match p {
        Power::Expand => potion(&MOSS[1..]),
        Power::Shrink => potion(&CRYSTAL[1..]),
        Power::Haste => potion(&BLOOD[1..]),
        Power::Slow => art(
            &[
                "kkkkkkkkkk",
                "kwwwwwwwwk",
                ".kssssssk.",
                "..kssssk..",
                "...kssk...",
                "...kgsk...",
                "..kggsgk..",
                ".kgsssssk.",
                "kwwwwwwwwk",
                "kkkkkkkkkk",
            ],
            &[
                ('k', INK),
                ('w', WOOD[4]),
                ('s', MANA[4]),
                ('g', rgb(0x50606c)),
            ],
        ),
        Power::Legion => art(
            &[
                ".kkkkkkkk.",
                "kPppppppPk",
                ".kpllllpk.",
                ".kppppppk.",
                ".kpllllpk.",
                ".kppppppk.",
                ".kplrrlpk.",
                ".kpprrppk.",
                "kPppppppPk",
                ".kkkkkkkk.",
            ],
            &[
                ('k', INK),
                ('P', PARCHMENT[1]),
                ('p', PARCHMENT[5]),
                ('l', SCROLL_INK_FADED),
                ('r', BLOOD[4]),
            ],
        ),
        Power::Life => art(
            &[
                "...kkkk...",
                "..kyyyyk..",
                "..kyk.yk..",
                "..kyyyyk..",
                "kkkkyykkkk",
                "kYyyyyyyyk",
                "kkkkyykkkk",
                "...kyyk...",
                "...kyyk...",
                "...kkkk...",
            ],
            &[('k', INK), ('y', GOLD[4]), ('Y', GOLD[6])],
        ),
        Power::Skull => art(
            &[
                "..kkkkkk..",
                ".kbbbbbbk.",
                "kbBbbbbbbk",
                "kbkkbbkkbk",
                "kbkrbbkrbk",
                "kbbbbbbbbk",
                ".kbbkkbbk.",
                "..kbbbbk..",
                "..kbkkbk..",
                "...kkkk...",
            ],
            &[
                ('k', INK),
                ('b', rgb(0xd8ccb0)),
                ('B', WHITE),
                ('r', BLOOD[5]),
            ],
        ),
        Power::Crossbow => art(
            &[
                "....kk....",
                "...kmmk...",
                "kk.kwwk.kk",
                "kik.ww.kik",
                ".kissssik.",
                "..k.ww.k..",
                "....ww....",
                "...kwwk...",
                "...kwwk...",
                "....kk....",
            ],
            &[
                ('k', INK),
                ('m', IRON[6]),
                ('i', IRON[4]),
                ('w', WOOD[4]),
                ('s', PARCHMENT[6]),
            ],
        ),
        Power::Warp => art(
            &[
                "...kkkk...",
                "..kbbbbk..",
                ".kbBccBbk.",
                ".kbcwwcbk.",
                ".kbcwwcbk.",
                ".kbcwwcbk.",
                ".kbcwwcbk.",
                ".kbBccBbk.",
                "..kbbbbk..",
                "...kkkk...",
            ],
            &[
                ('k', INK),
                ('b', MANA[2]),
                ('B', MANA[3]),
                ('c', MANA[4]),
                ('w', MANA[5]),
            ],
        ),
        Power::Fire => runestone(
            &GREY_STONE,
            &["...#..", "..##..", "..###.", ".####.", ".####.", "..##.."],
            FIRE[4],
        ),
        Power::Phantom => runestone(
            &GREY_STONE,
            &["......", ".####.", "#.##.#", "#.##.#", ".####.", "......"],
            GHOST[3],
        ),
        Power::Grab => runestone(
            &MOSS,
            &["#....#", "#....#", "#....#", "#....#", ".#..#.", "..##.."],
            rgb(0xe8f0a0),
        ),
        Power::Quake => runestone(
            &SANDSTONE,
            &["..#...", "...#..", "..#...", ".##...", "...##.", "....#."],
            INK,
        ),
        Power::Zap => runestone(
            &SLATE,
            &["...##.", "..##..", ".####.", "..##..", ".##...", ".#...."],
            GOLD[6],
        ),
        Power::Split => runestone(
            &SLATE,
            &["..##..", "..##..", "......", "##..##", "##..##", "......"],
            ORB[3],
        ),
    }
}

// ---------------------------------------------------------------------------
// HUD ornaments.

pub fn ankh() -> Img {
    let a = art(
        &[
            ".yyy.", ".y.y.", ".yyy.", "yyyyy", "..y..", "..y..", "..y..",
        ],
        &[('y', GOLD[4])],
    );
    let mut a = outlined(&a, INK);
    // Highlight.
    a.set(2, 1, GOLD[6]);
    a.set(1, 4, GOLD[6]);
    a
}

/// Gargoyle head that crowns the side panel. Eye pixels are returned separately so
/// the renderer can make them glow.
pub fn gargoyle() -> (Img, Vec<(i32, i32)>) {
    let half = [
        "kk..........",
        "k4k.........",
        ".k4k........",
        ".k54k...kkkk",
        "..k54kkk4444",
        "..k554445555",
        "...k45555555",
        "...k4kkk5555",
        "..kk4keek555",
        ".k44kkeek455",
        ".k444kkkk455",
        "..k44444k555",
        "...k4444k3k5",
        "...k444kkkkk",
        "...k44kwkwkw",
        "...k44k22222",
        "....k4kwkwkw",
        "....k44kkkkk",
        ".....kk44444",
        ".......kkkkk",
    ];
    let map = [
        ('k', INK),
        ('2', rgb(0x1a0806)),
        ('3', FRAME[3]),
        ('4', FRAME[5]),
        ('5', FRAME[6]),
        ('w', rgb(0xc8bca0)),
        ('e', rgb(0x200000)),
    ];
    let mut img = art_mirrored(&half, &map);
    // Shade: light from the top-left.
    for y in 0..img.h {
        for x in 0..img.w {
            let c = img.get(x, y);
            if c == FRAME[6] && x > 13 {
                img.set(x, y, FRAME[5]);
            } else if c == FRAME[5] && x > 15 && (x + y) % 2 == 0 {
                img.set(x, y, FRAME[4]);
            }
        }
    }
    let mut eyes = Vec::new();
    for y in 0..img.h {
        for x in 0..img.w {
            if img.get(x, y) == rgb(0x200000) {
                eyes.push((x, y));
            }
        }
    }
    (img, eyes)
}

pub fn torch_sconce() -> Img {
    art(
        &[
            ".kwwk.", ".kwWk.", ".kwwk.", "kiiiik", ".kwwk.", ".kwwk.", "..kk..", "..ik..",
            ".iik..",
        ],
        &[('k', INK), ('w', WOOD[3]), ('W', WOOD[5]), ('i', IRON[4])],
    )
}

/// Animated torch flame, drawn procedurally each frame.
pub fn draw_flame(img: &mut Img, cx: i32, base_y: i32, t: f32, seed: u32, scale: f32) {
    let height = (7.0 * scale) as i32;
    let half_w = (2.5 * scale).max(1.5);
    for dy in 0..height {
        let fy = dy as f32 / height as f32; // 0 at the base
        let sway = (t * 7.0 + fy * 3.0 + seed as f32).sin() * fy * 1.2 * scale;
        let w = half_w * (1.0 - fy * 0.85);
        for dx in -(half_w as i32 + 1)..=(half_w as i32 + 1) {
            let x = dx as f32 - sway;
            let n = fbm(dx as f32 * 0.6, dy as f32 * 0.5 - t * 9.0, 2, seed);
            let edge = 1.0 - (x.abs() / (w + 0.3)).min(1.0);
            let v = edge * (1.2 - fy * 0.9) + (n - 0.5) * 0.6;
            if v > 0.12 {
                let px = cx + dx;
                let py = base_y - dy;
                img.set(px, py, dither(&FIRE, v.min(1.0), px, py));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icons_are_ten_by_ten() {
        for p in Power::ALL {
            let i = power_icon(p);
            assert_eq!((i.w, i.h), (10, 10), "{p:?}");
            assert!(i.px.iter().any(|c| c.a != 0));
        }
    }

    #[test]
    fn bricks_have_full_size_and_mortar() {
        for kind in [
            BrickKind::Stone(0),
            BrickKind::Stone(1),
            BrickKind::Stone(2),
            BrickKind::Stone(3),
            BrickKind::Stone(4),
            BrickKind::Stone(5),
            BrickKind::Bound,
            BrickKind::Crystal,
            BrickKind::Iron,
            BrickKind::Rune,
            BrickKind::Gold,
        ] {
            let b = brick(kind, 0, 1);
            assert_eq!((b.w, b.h), (18, 8));
            assert_eq!(b.get(17, 7), INK, "{kind:?} mortar corner");
        }
    }

    #[test]
    fn gargoyle_is_symmetric_with_eyes() {
        let (g, eyes) = gargoyle();
        assert_eq!(g.w, 24);
        assert!(eyes.len() >= 4);
    }

    #[test]
    fn paddle_matches_width() {
        let p = paddle(
            32,
            &PaddleLook {
                crossbow: true,
                grab: true,
                time: 0.0,
            },
        );
        assert_eq!(p.w, 32);
    }
}
