//! Hand-drawn variable-width bitmap font. Cells are 9 rows tall: capitals use rows
//! 0-6 (baseline on row 6), lowercase descenders reach rows 7-8.

use super::canvas::{Img, Rgba};
use std::sync::OnceLock;

pub const LINE_H: i32 = 10;
const SPACE_W: i32 = 3;

const GLYPHS: &str = r#"
:A
.###.
#...#
#...#
#####
#...#
#...#
#...#
:B
####.
#...#
#...#
####.
#...#
#...#
####.
:C
.###.
#...#
#....
#....
#....
#...#
.###.
:D
####.
#...#
#...#
#...#
#...#
#...#
####.
:E
#####
#....
#....
####.
#....
#....
#####
:F
#####
#....
#....
####.
#....
#....
#....
:G
.###.
#...#
#....
#.###
#...#
#...#
.####
:H
#...#
#...#
#...#
#####
#...#
#...#
#...#
:I
###
.#.
.#.
.#.
.#.
.#.
###
:J
..###
...#.
...#.
...#.
#..#.
#..#.
.##..
:K
#...#
#..#.
#.#..
##...
#.#..
#..#.
#...#
:L
#....
#....
#....
#....
#....
#....
#####
:M
#...#
##.##
#.#.#
#.#.#
#...#
#...#
#...#
:N
#...#
##..#
#.#.#
#..##
#...#
#...#
#...#
:O
.###.
#...#
#...#
#...#
#...#
#...#
.###.
:P
####.
#...#
#...#
####.
#....
#....
#....
:Q
.###.
#...#
#...#
#...#
#.#.#
#..#.
.##.#
:R
####.
#...#
#...#
####.
#.#..
#..#.
#...#
:S
.####
#....
#....
.###.
....#
....#
####.
:T
#####
..#..
..#..
..#..
..#..
..#..
..#..
:U
#...#
#...#
#...#
#...#
#...#
#...#
.###.
:V
#...#
#...#
#...#
#...#
#...#
.#.#.
..#..
:W
#...#
#...#
#...#
#.#.#
#.#.#
##.##
#...#
:X
#...#
#...#
.#.#.
..#..
.#.#.
#...#
#...#
:Y
#...#
#...#
.#.#.
..#..
..#..
..#..
..#..
:Z
#####
....#
...#.
..#..
.#...
#....
#####
:a
.....
.....
.###.
....#
.####
#...#
.####
:b
#....
#....
####.
#...#
#...#
#...#
####.
:c
....
....
.###
#...
#...
#...
.###
:d
....#
....#
.####
#...#
#...#
#...#
.####
:e
.....
.....
.###.
#...#
#####
#....
.###.
:f
..##
.#..
###.
.#..
.#..
.#..
.#..
:g
.....
.....
.####
#...#
#...#
#...#
.####
....#
.###.
:h
#....
#....
####.
#...#
#...#
#...#
#...#
:i
#
.
#
#
#
#
#
:j
..#
...
..#
..#
..#
..#
..#
#.#
.#.
:k
#...
#...
#..#
#.#.
##..
#.#.
#..#
:l
#.
#.
#.
#.
#.
#.
.#
:m
.....
.....
##.#.
#.#.#
#.#.#
#.#.#
#.#.#
:n
....
....
###.
#..#
#..#
#..#
#..#
:o
....
....
.##.
#..#
#..#
#..#
.##.
:p
....
....
###.
#..#
#..#
#..#
###.
#...
#...
:q
....
....
.###
#..#
#..#
#..#
.###
...#
...#
:r
...
...
#.#
##.
#..
#..
#..
:s
....
....
.###
#...
.##.
...#
###.
:t
.#..
.#..
###.
.#..
.#..
.#..
..##
:u
....
....
#..#
#..#
#..#
#..#
.###
:v
.....
.....
#...#
#...#
#...#
.#.#.
..#..
:w
.....
.....
#...#
#...#
#.#.#
#.#.#
.#.#.
:x
....
....
#..#
#..#
.##.
#..#
#..#
:y
....
....
#..#
#..#
#..#
#..#
.###
...#
.##.
:z
....
....
####
...#
.##.
#...
####
:0
.###.
#...#
#..##
#.#.#
##..#
#...#
.###.
:1
.#.
##.
.#.
.#.
.#.
.#.
###
:2
.###.
#...#
....#
..##.
.#...
#....
#####
:3
####.
....#
....#
.###.
....#
....#
####.
:4
...#.
..##.
.#.#.
#..#.
#####
...#.
...#.
:5
#####
#....
####.
....#
....#
#...#
.###.
:6
.###.
#....
#....
####.
#...#
#...#
.###.
:7
#####
....#
...#.
..#..
.#...
.#...
.#...
:8
.###.
#...#
#...#
.###.
#...#
#...#
.###.
:9
.###.
#...#
#...#
.####
....#
....#
.###.
:.
.
.
.
.
.
.
#
:,
..
..
..
..
..
..
.#
#.
:!
#
#
#
#
#
.
#
:?
.###.
#...#
....#
..##.
..#..
.....
..#..
:'
#
#
:"
#.#
#.#
::
.
.
#
.
.
#
:;
..
..
.#
..
..
.#
#.
:-
...
...
...
###
:+
...
...
.#.
###
.#.
:/
....#
...#.
...#.
..#..
.#...
.#...
#....
:(
.#
#.
#.
#.
#.
#.
.#
:)
#.
.#
.#
.#
.#
.#
#.
:*
.....
#.#.#
.###.
#####
.###.
#.#.#
:=
...
...
###
...
###
"#;

pub struct Glyph {
    pub w: i32,
    /// Pixel offsets (x, y) within the cell.
    pub pixels: Vec<(i32, i32)>,
}

pub struct Font {
    glyphs: Vec<Option<Glyph>>,
}

fn parse() -> Font {
    let mut glyphs: Vec<Option<Glyph>> = (0..128).map(|_| None).collect();
    let mut current: Option<(char, Vec<&str>)> = None;
    let finish = |cur: Option<(char, Vec<&str>)>, glyphs: &mut Vec<Option<Glyph>>| {
        if let Some((ch, rows)) = cur {
            let mut pixels = Vec::new();
            for (y, row) in rows.iter().enumerate() {
                for (x, c) in row.chars().enumerate() {
                    if c == '#' {
                        pixels.push((x as i32, y as i32));
                    }
                }
            }
            let min_x = pixels.iter().map(|p| p.0).min().unwrap_or(0);
            let max_x = pixels.iter().map(|p| p.0).max().unwrap_or(0);
            for p in &mut pixels {
                p.0 -= min_x;
            }
            if (ch as usize) < glyphs.len() && glyphs[ch as usize].is_none() {
                glyphs[ch as usize] = Some(Glyph {
                    w: max_x - min_x + 1,
                    pixels,
                });
            }
        }
    };
    for line in GLYPHS.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix(':')
            && let Some(ch) = rest.chars().next()
        {
            finish(current.take(), &mut glyphs);
            current = Some((ch, Vec::new()));
            continue;
        }
        if let Some((_, rows)) = current.as_mut() {
            rows.push(line);
        }
    }
    finish(current.take(), &mut glyphs);
    Font { glyphs }
}

pub fn font() -> &'static Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    FONT.get_or_init(parse)
}

impl Font {
    pub fn glyph(&self, c: char) -> Option<&Glyph> {
        self.glyphs.get(c as usize).and_then(|g| g.as_ref())
    }

    fn advance(&self, c: char) -> i32 {
        match self.glyph(c) {
            Some(g) => g.w + 1,
            None => SPACE_W + 1,
        }
    }

    pub fn width(&self, text: &str) -> i32 {
        let w: i32 = text.chars().map(|c| self.advance(c)).sum();
        (w - 1).max(0)
    }
}

pub fn text_width(text: &str) -> i32 {
    font().width(text)
}

/// Draw text with its top-left at (x, y). Returns the width drawn.
pub fn draw_text(img: &mut Img, x: i32, y: i32, text: &str, c: Rgba) -> i32 {
    let f = font();
    let mut cx = x;
    for ch in text.chars() {
        if let Some(g) = f.glyph(ch) {
            for &(px, py) in &g.pixels {
                img.blend(cx + px, y + py, c);
            }
        }
        cx += f.advance(ch);
    }
    cx - x - 1
}

pub fn draw_text_centered(img: &mut Img, cx: i32, y: i32, text: &str, c: Rgba) {
    draw_text(img, cx - text_width(text) / 2, y, text, c);
}

/// Text with a one-pixel drop shadow, for legibility over busy backgrounds.
pub fn draw_text_shadow(img: &mut Img, x: i32, y: i32, text: &str, c: Rgba, shadow: Rgba) -> i32 {
    draw_text(img, x + 1, y + 1, text, shadow);
    draw_text(img, x, y, text, c)
}

/// "Carved" text: dark top-left edge, light bottom-right edge, like letters cut into stone.
pub fn draw_text_engraved(
    img: &mut Img,
    x: i32,
    y: i32,
    text: &str,
    dark: Rgba,
    light: Rgba,
    face: Rgba,
) {
    draw_text(img, x - 1, y - 1, text, dark);
    draw_text(img, x + 1, y + 1, text, light);
    draw_text(img, x, y, text, face);
}

/// Large title text: each font pixel becomes a `scale`x`scale` block shaded with a
/// vertical ramp, with a black outline and a drop shadow.
pub fn draw_text_big(
    img: &mut Img,
    x: i32,
    y: i32,
    text: &str,
    scale: i32,
    ramp: &[Rgba],
    outline: Rgba,
) {
    let f = font();
    let mut mask = Img::new(f.width(text) * scale + 4, 9 * scale + 4);
    let mut cx = 2;
    for ch in text.chars() {
        if let Some(g) = f.glyph(ch) {
            for &(px, py) in &g.pixels {
                for sy in 0..scale {
                    for sx in 0..scale {
                        mask.set(
                            cx + px * scale + sx,
                            2 + py * scale + sy,
                            Rgba {
                                r: 255,
                                g: 255,
                                b: 255,
                                a: 255,
                            },
                        );
                    }
                }
            }
        }
        cx += f.advance(ch) * scale;
    }
    let h7 = (7 * scale) as f32;
    // Shadow, then outline, then the shaded face.
    for my in 0..mask.h {
        for mx in 0..mask.w {
            if mask.get(mx, my).a == 0 {
                continue;
            }
            img.set(x + mx - 2 + 2, y + my - 2 + 2, outline.scale(0.6));
            for (ox, oy) in [(-1, 0), (1, 0), (0, -1), (0, 1), (1, 1)] {
                if mask.get(mx + ox, my + oy).a == 0 {
                    img.set(x + mx - 2 + ox, y + my - 2 + oy, outline);
                }
            }
        }
    }
    for my in 0..mask.h {
        for mx in 0..mask.w {
            if mask.get(mx, my).a == 0 {
                continue;
            }
            let v = 1.0 - ((my - 2) as f32 / h7).clamp(0.0, 1.0);
            // Bevel: pixels on an upper edge catch the light.
            let edge = mask.get(mx, my - 1).a == 0;
            let low = mask.get(mx, my + 1).a == 0;
            let v = if edge {
                1.0
            } else if low {
                v * 0.5
            } else {
                0.25 + v * 0.65
            };
            img.set(
                x + mx - 2,
                y + my - 2,
                super::canvas::dither(ramp, v, mx, my),
            );
        }
    }
}

/// Greedy word wrap to `max_w` pixels.
pub fn wrap(text: &str, max_w: i32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if text_width(&candidate) <= max_w || line.is_empty() {
            line = candidate;
        } else {
            lines.push(std::mem::take(&mut line));
            line = word.to_string();
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_all_printable_glyphs_used_by_the_game() {
        let f = font();
        for c in ('A'..='Z').chain('a'..='z').chain('0'..='9') {
            assert!(f.glyph(c).is_some(), "missing glyph {c:?}");
        }
        for c in ".,!?':;-+/()*=\"".chars() {
            assert!(f.glyph(c).is_some(), "missing glyph {c:?}");
        }
    }

    #[test]
    fn widths_are_sane() {
        assert_eq!(text_width("I"), 3);
        assert_eq!(text_width("i"), 1);
        assert!(text_width("Hello World") > text_width("Hello"));
        assert_eq!(text_width(""), 0);
    }

    #[test]
    fn wrap_respects_width() {
        let lines = wrap(
            "The quick brown fox jumps over the lazy dog again and again",
            80,
        );
        assert!(lines.len() > 1);
        for l in &lines {
            assert!(text_width(l) <= 80, "{l}");
        }
    }
}
