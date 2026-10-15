//! Renders the browser build's home-screen icons: the gargoyle from the side panel
//! on carved stone, with an orb below.
//!
//!     cargo run --release --bin icons -- [out_dir]

use stygian::gfx::canvas::Img;
use stygian::gfx::palette::FIRE;
use stygian::gfx::render::frame_stone;
use stygian::gfx::sprites::{self, BallStyle};

/// Icon size in art pixels. 192 = 32 * 6 and 512 = 32 * 16 keep the pixels square.
const SIZE: i32 = 32;

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "web".into());
    let mut img = Img::new(SIZE, SIZE);
    frame_stone(&mut img);
    img.shade_rect(0, 0, SIZE, SIZE, 0.7);
    let (garg, eyes) = sprites::gargoyle();
    let (gx, gy) = ((SIZE - garg.w) / 2, 3);
    img.blit(&garg, gx, gy);
    for (x, y) in eyes {
        img.set(gx + x, gy + y, FIRE[5]);
    }
    let orb = sprites::ball(BallStyle::Fire);
    img.blit(&orb, (SIZE - orb.w) / 2, gy + garg.h + 2);

    for px in [192u32, 512] {
        let scale = px / SIZE as u32;
        let mut buf = image::RgbaImage::new(px, px);
        for (x, y, p) in buf.enumerate_pixels_mut() {
            let c = img.get((x / scale) as i32, (y / scale) as i32);
            *p = image::Rgba([c.r, c.g, c.b, 255]);
        }
        let path = format!("{out}/icon-{px}.png");
        buf.save(&path).expect("write png");
        println!("wrote {path}");
    }
}
