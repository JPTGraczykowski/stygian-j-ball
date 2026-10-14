//! Headless renderer: plays scripted scenes and writes PNG screenshots.
//!
//!     cargo run --release --bin snapshot -- [out_dir]

use stygian::app::{App, FrameInput, Screen, autopilot};
use stygian::consts::*;
use stygian::fx::Fx;
use stygian::game::{Phase, Power};
use stygian::gfx::canvas::Img;
use stygian::gfx::render::Renderer;

fn save(img: &Img, path: &str, scale: u32) {
    let (w, h) = (img.w as u32 * scale, img.h as u32 * scale);
    let mut buf = image::RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let p = img.get((x / scale) as i32, (y / scale) as i32);
            buf.put_pixel(x, y, image::Rgba([p.r, p.g, p.b, 255]));
        }
    }
    buf.save(path).expect("write png");
    println!("wrote {path}");
}

fn run(app: &mut App, fx: &mut Fx, secs: f32, ai: bool) {
    let dt = 1.0 / 60.0;
    for _ in 0..(secs / dt) as usize {
        let mut input = FrameInput::default();
        if ai && app.screen == Screen::Playing {
            input.game = autopilot(&app.game);
        }
        let was_demo = app.showing_demo();
        let (events, _) = app.update(dt, &input);
        if was_demo != app.showing_demo() {
            fx.clear();
        }
        for e in &events {
            fx.handle(e);
        }
        let g = if app.showing_demo() {
            &app.demo
        } else {
            &app.game
        };
        fx.update(dt, Some(g));
    }
}

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "shots".into());
    std::fs::create_dir_all(&out).unwrap();
    let mut r = Renderer::new();
    let mut fx = Fx::new();
    let mut img = Img::new(SCREEN_W, SCREEN_H);
    let mut app = App::new(None, 42);

    run(&mut app, &mut fx, 4.0, false);
    r.draw(&mut img, &app, &mut fx);
    save(&img, &format!("{out}/title.png"), 3);

    app.screen = Screen::Lore;
    r.draw(&mut img, &app, &mut fx);
    save(&img, &format!("{out}/lore.png"), 3);
    app.screen = Screen::Title;

    // Start a game and play a little with the autopilot.
    let i = FrameInput {
        confirm: true,
        ..Default::default()
    };
    app.update(1.0 / 60.0, &i);
    fx.clear();
    run(&mut app, &mut fx, 0.5, false);
    r.draw(&mut img, &app, &mut fx);
    save(&img, &format!("{out}/serve.png"), 3);

    run(&mut app, &mut fx, 9.0, true);
    r.draw(&mut img, &app, &mut fx);
    save(&img, &format!("{out}/play.png"), 3);

    // Showcase effects on a later level.
    app.game.load_level(9);
    app.game.release_balls();
    app.game.apply_power(Power::Fire, app.game.paddle_pos());
    app.game.apply_power(Power::Crossbow, app.game.paddle_pos());
    app.game.apply_power(Power::Split, app.game.paddle_pos());
    app.game.apply_power(Power::Expand, app.game.paddle_pos());
    run(&mut app, &mut fx, 3.3, true);
    r.draw(&mut img, &app, &mut fx);
    save(&img, &format!("{out}/fire.png"), 3);

    app.game.load_level(12);
    app.game.release_balls();
    app.game.apply_power(Power::Phantom, app.game.paddle_pos());
    app.game.apply_power(Power::Grab, app.game.paddle_pos());
    run(&mut app, &mut fx, 2.0, true);
    r.draw(&mut img, &app, &mut fx);
    save(&img, &format!("{out}/phantom.png"), 3);

    app.game.load_level(4);
    app.game.release_balls();
    run(&mut app, &mut fx, 4.0, true);
    r.draw(&mut img, &app, &mut fx);
    save(&img, &format!("{out}/crystal.png"), 3);

    app.screen = Screen::Paused;
    r.draw(&mut img, &app, &mut fx);
    save(&img, &format!("{out}/paused.png"), 3);
    app.screen = Screen::Playing;

    app.game.lives = 1;
    app.game.balls.clear();
    run(&mut app, &mut fx, 2.0, false);
    r.draw(&mut img, &app, &mut fx);
    assert!(matches!(app.game.phase, Phase::GameOver) || app.screen != Screen::Playing);
    save(&img, &format!("{out}/gameover.png"), 3);

    // Name entry and the hall of heroes.
    app.screen = Screen::EnterName;
    app.name = "Iolo".into();
    r.draw(&mut img, &app, &mut fx);
    save(&img, &format!("{out}/name.png"), 3);
    app.insert_score("Iolo", app.game.score, 13);
    app.insert_score("Dupre", 4210, 6);
    app.insert_score("Shamino", 870, 2);
    app.highlight = Some(1);
    app.screen = Screen::Scores;
    r.draw(&mut img, &app, &mut fx);
    save(&img, &format!("{out}/scores.png"), 3);

    // Victory banner.
    app.game = stygian::game::Game::new(5);
    app.game.phase = Phase::Victory;
    app.screen = Screen::Playing;
    r.draw(&mut img, &app, &mut fx);
    save(&img, &format!("{out}/victory.png"), 3);
}
