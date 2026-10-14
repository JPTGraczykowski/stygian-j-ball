//! Measures the cost of rendering one frame.
use stygian::app::App;
use stygian::consts::*;
use stygian::fx::Fx;
use stygian::gfx::{canvas::Img, render::Renderer};

fn main() {
    let mut r = Renderer::new();
    let mut fx = Fx::new();
    let mut img = Img::new(SCREEN_W, SCREEN_H);
    let app = App::new(None, 1);
    let n = 200;
    let t = std::time::Instant::now();
    for _ in 0..n {
        r.draw(&mut img, &app, &mut fx);
    }
    println!("{:.2} ms per frame", t.elapsed().as_secs_f64() * 1000.0 / n as f64);
}
