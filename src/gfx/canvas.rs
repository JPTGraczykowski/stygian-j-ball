//! A tiny software framebuffer: every frame is composed pixel-by-pixel, VGA style.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub const CLEAR: Rgba = Rgba {
    r: 0,
    g: 0,
    b: 0,
    a: 0,
};

pub const fn rgb(hex: u32) -> Rgba {
    Rgba {
        r: (hex >> 16) as u8,
        g: (hex >> 8) as u8,
        b: hex as u8,
        a: 255,
    }
}

impl Rgba {
    pub fn scale(self, f: f32) -> Rgba {
        let s = |c: u8| (c as f32 * f).clamp(0.0, 255.0) as u8;
        Rgba {
            r: s(self.r),
            g: s(self.g),
            b: s(self.b),
            a: self.a,
        }
    }

    pub fn lerp(self, o: Rgba, t: f32) -> Rgba {
        let l = |a: u8, b: u8| {
            (a as f32 + (b as f32 - a as f32) * t)
                .round()
                .clamp(0.0, 255.0) as u8
        };
        Rgba {
            r: l(self.r, o.r),
            g: l(self.g, o.g),
            b: l(self.b, o.b),
            a: l(self.a, o.a),
        }
    }

    pub fn with_alpha(self, a: u8) -> Rgba {
        Rgba { a, ..self }
    }

    pub fn luma(self) -> f32 {
        (0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32) / 255.0
    }
}

/// 4x4 ordered-dither thresholds in (0, 1).
const BAYER4: [[f32; 4]; 4] = [
    [0.5 / 16.0, 8.5 / 16.0, 2.5 / 16.0, 10.5 / 16.0],
    [12.5 / 16.0, 4.5 / 16.0, 14.5 / 16.0, 6.5 / 16.0],
    [3.5 / 16.0, 11.5 / 16.0, 1.5 / 16.0, 9.5 / 16.0],
    [15.5 / 16.0, 7.5 / 16.0, 13.5 / 16.0, 5.5 / 16.0],
];

pub fn bayer(x: i32, y: i32) -> f32 {
    BAYER4[(y & 3) as usize][(x & 3) as usize]
}

/// Pick a colour from a dark-to-light ramp with ordered dithering between steps.
pub fn dither(ramp: &[Rgba], v: f32, x: i32, y: i32) -> Rgba {
    let n = ramp.len();
    let f = v.clamp(0.0, 1.0) * (n - 1) as f32;
    let i = f.floor() as usize;
    if i + 1 >= n {
        return ramp[n - 1];
    }
    if f - i as f32 > bayer(x, y) {
        ramp[i + 1]
    } else {
        ramp[i]
    }
}

#[derive(Clone, Debug)]
pub struct Img {
    pub w: i32,
    pub h: i32,
    pub px: Vec<Rgba>,
}

impl Img {
    pub fn new(w: i32, h: i32) -> Img {
        Img {
            w,
            h,
            px: vec![CLEAR; (w * h) as usize],
        }
    }

    pub fn filled(w: i32, h: i32, c: Rgba) -> Img {
        Img {
            w,
            h,
            px: vec![c; (w * h) as usize],
        }
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.w && y < self.h
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> Rgba {
        if self.in_bounds(x, y) {
            self.px[(y * self.w + x) as usize]
        } else {
            CLEAR
        }
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: Rgba) {
        if self.in_bounds(x, y) {
            self.px[(y * self.w + x) as usize] = c;
        }
    }

    /// Alpha-blend a colour onto a pixel (uses `c.a`).
    #[inline]
    pub fn blend(&mut self, x: i32, y: i32, c: Rgba) {
        if c.a == 0 || !self.in_bounds(x, y) {
            return;
        }
        if c.a == 255 {
            self.px[(y * self.w + x) as usize] = c;
            return;
        }
        let i = (y * self.w + x) as usize;
        let d = self.px[i];
        let t = c.a as f32 / 255.0;
        let mut o = d.lerp(c, t);
        o.a = d.a.max(c.a);
        self.px[i] = o;
    }

    /// Additive light (scaled by `amount`).
    #[inline]
    pub fn add(&mut self, x: i32, y: i32, c: Rgba, amount: f32) {
        if !self.in_bounds(x, y) {
            return;
        }
        let i = (y * self.w + x) as usize;
        let d = self.px[i];
        let a = |d: u8, s: u8| (d as f32 + s as f32 * amount).min(255.0) as u8;
        self.px[i] = Rgba {
            r: a(d.r, c.r),
            g: a(d.g, c.g),
            b: a(d.b, c.b),
            a: d.a,
        };
    }

    pub fn fill(&mut self, c: Rgba) {
        self.px.fill(c);
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgba) {
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(self.w);
        let y1 = (y + h).min(self.h);
        for yy in y0..y1 {
            for xx in x0..x1 {
                self.blend(xx, yy, c);
            }
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgba) {
        self.hline(x, x + w - 1, y, c);
        self.hline(x, x + w - 1, y + h - 1, c);
        self.vline(x, y, y + h - 1, c);
        self.vline(x + w - 1, y, y + h - 1, c);
    }

    pub fn hline(&mut self, x0: i32, x1: i32, y: i32, c: Rgba) {
        for x in x0.min(x1)..=x0.max(x1) {
            self.blend(x, y, c);
        }
    }

    pub fn vline(&mut self, x: i32, y0: i32, y1: i32, c: Rgba) {
        for y in y0.min(y1)..=y0.max(y1) {
            self.blend(x, y, c);
        }
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: Rgba) {
        let (mut x, mut y) = (x0, y0);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.blend(x, y, c);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, c: Rgba) {
        for y in -r..=r {
            for x in -r..=r {
                if x * x + y * y <= r * r + r / 2 {
                    self.blend(cx + x, cy + y, c);
                }
            }
        }
    }

    /// Copy (with alpha) another image onto this one.
    pub fn blit(&mut self, src: &Img, dx: i32, dy: i32) {
        self.blit_region(src, 0, 0, src.w, src.h, dx, dy);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn blit_region(&mut self, src: &Img, sx: i32, sy: i32, sw: i32, sh: i32, dx: i32, dy: i32) {
        for y in 0..sh {
            let ty = dy + y;
            if ty < 0 || ty >= self.h {
                continue;
            }
            for x in 0..sw {
                let c = src.get(sx + x, sy + y);
                if c.a != 0 {
                    self.blend(dx + x, ty, c);
                }
            }
        }
    }

    /// Blit with a global opacity multiplier and optional horizontal flip.
    pub fn blit_ex(&mut self, src: &Img, dx: i32, dy: i32, opacity: f32, flip_x: bool) {
        for y in 0..src.h {
            for x in 0..src.w {
                let sx = if flip_x { src.w - 1 - x } else { x };
                let c = src.get(sx, y);
                if c.a != 0 {
                    let a = (c.a as f32 * opacity).clamp(0.0, 255.0) as u8;
                    self.blend(dx + x, dy + y, c.with_alpha(a));
                }
            }
        }
    }

    /// Blit additively: the source brightens what is underneath (for glows and fire).
    pub fn blit_add(&mut self, src: &Img, dx: i32, dy: i32, amount: f32) {
        for y in 0..src.h {
            for x in 0..src.w {
                let c = src.get(x, y);
                if c.a != 0 {
                    self.add(dx + x, dy + y, c, amount * c.a as f32 / 255.0);
                }
            }
        }
    }

    /// Blit every opaque pixel in a single colour (silhouettes, flashes).
    pub fn blit_solid(&mut self, src: &Img, dx: i32, dy: i32, c: Rgba) {
        for y in 0..src.h {
            for x in 0..src.w {
                if src.get(x, y).a != 0 {
                    self.blend(dx + x, dy + y, c);
                }
            }
        }
    }

    /// Darken a rectangle (multiplies RGB by `f`).
    pub fn shade_rect(&mut self, x: i32, y: i32, w: i32, h: i32, f: f32) {
        for yy in y.max(0)..(y + h).min(self.h) {
            for xx in x.max(0)..(x + w).min(self.w) {
                let i = (yy * self.w + xx) as usize;
                self.px[i] = self.px[i].scale(f);
            }
        }
    }

    /// Dithered darkening: every other pixel in a checkerboard (classic VGA "shadow").
    pub fn checker_shade(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgba) {
        for yy in y.max(0)..(y + h).min(self.h) {
            for xx in x.max(0)..(x + w).min(self.w) {
                if (xx + yy) & 1 == 0 {
                    self.set(xx, yy, c);
                }
            }
        }
    }

    pub fn sub_image(&self, x: i32, y: i32, w: i32, h: i32) -> Img {
        let mut out = Img::new(w, h);
        out.blit_region(self, x, y, w, h, 0, 0);
        out
    }

    pub fn to_rgba_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.px.len() * 4);
        for p in &self.px {
            out.extend_from_slice(&[p.r, p.g, p.b, 255]);
        }
        out
    }

    pub fn write_rgba_bytes(&self, out: &mut [u8]) {
        for (dst, p) in out.as_chunks_mut::<4>().0.iter_mut().zip(&self.px) {
            dst[0] = p.r;
            dst[1] = p.g;
            dst[2] = p.b;
            dst[3] = 255;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_and_bounds() {
        let mut img = Img::new(4, 3);
        img.set(1, 2, rgb(0x123456));
        assert_eq!(img.get(1, 2), rgb(0x123456));
        img.set(-1, 0, rgb(0xffffff));
        img.set(4, 0, rgb(0xffffff));
        assert_eq!(img.get(9, 9), CLEAR);
    }

    #[test]
    fn blit_respects_transparency() {
        let mut dst = Img::filled(3, 3, rgb(0x000000));
        let mut src = Img::new(2, 2);
        src.set(0, 0, rgb(0xff0000));
        dst.blit(&src, 1, 1);
        assert_eq!(dst.get(1, 1), rgb(0xff0000));
        assert_eq!(dst.get(2, 2), rgb(0x000000));
    }

    #[test]
    fn dither_hits_ramp_ends() {
        let ramp = [rgb(0), rgb(0x808080), rgb(0xffffff)];
        assert_eq!(dither(&ramp, 0.0, 0, 0), ramp[0]);
        assert_eq!(dither(&ramp, 1.0, 3, 3), ramp[2]);
        // Midway between two steps produces a mix.
        let mut hi = 0;
        for y in 0..4 {
            for x in 0..4 {
                if dither(&ramp, 0.75, x, y) == ramp[2] {
                    hi += 1;
                }
            }
        }
        assert_eq!(hi, 8);
    }
}
