//! Window, input and audio glue. The game itself renders into a 384x240 software
//! framebuffer which is uploaded as a texture and scaled up with nearest filtering.

use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound, stop_sound};
use macroquad::prelude::*;
use std::collections::HashMap;
use stygian::app::{App, FrameInput, Screen, autopilot, default_scores_path};
use stygian::audio;
use stygian::consts::{PF_X, SCREEN_H, SCREEN_W};
use stygian::fx::Fx;
use stygian::game::Sfx;
use stygian::gfx::canvas::Img;
use stygian::gfx::render::Renderer;

fn conf() -> Conf {
    Conf {
        window_title: "Stygian J-Ball".to_owned(),
        window_width: SCREEN_W * 4,
        window_height: SCREEN_H * 4,
        window_resizable: true,
        high_dpi: false,
        ..Default::default()
    }
}

fn sfx_volume(s: Sfx) -> f32 {
    match s {
        Sfx::Wall => 0.35,
        Sfx::BrickHit | Sfx::Paddle => 0.7,
        _ => 0.85,
    }
}

/// Command-line options (mostly for testing and recording).
#[derive(Default)]
struct Opts {
    mute: bool,
    autoplay: bool,
    screenshot: Option<String>,
    after: f32,
}

fn parse_opts() -> Opts {
    let mut o = Opts {
        after: 3.0,
        ..Default::default()
    };
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--mute" => o.mute = true,
            "--autoplay" => o.autoplay = true,
            "--screenshot" => o.screenshot = args.next(),
            "--after" => o.after = args.next().and_then(|s| s.parse().ok()).unwrap_or(3.0),
            "-h" | "--help" => {
                println!(
                    "stygian-j-ball [--mute] [--autoplay] [--screenshot FILE.png [--after SECONDS]]"
                );
                println!("  --mute        start with sound and music off");
                println!("  --autoplay    start a game immediately and let the paddle play itself");
                println!(
                    "  --screenshot  save a screenshot after --after seconds (default 3) and quit"
                );
                std::process::exit(0);
            }
            other => eprintln!("ignoring unknown argument {other:?}"),
        }
    }
    o
}

struct Layout {
    scale: f32,
    ox: f32,
    oy: f32,
}

fn layout() -> Layout {
    let (sw, sh) = (screen_width(), screen_height());
    let scale = (sw / SCREEN_W as f32).min(sh / SCREEN_H as f32);
    Layout {
        scale,
        ox: ((sw - SCREEN_W as f32 * scale) / 2.0).floor(),
        oy: ((sh - SCREEN_H as f32 * scale) / 2.0).floor(),
    }
}

#[macroquad::main(conf)]
async fn main() {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(7);
    let opts = parse_opts();
    let mut app = App::new(default_scores_path(), seed);
    if opts.mute {
        app.sound_on = false;
        app.music_on = false;
    }
    let mut elapsed = 0.0f32;
    let mut renderer = Renderer::new();
    let mut fx = Fx::new();
    let mut frame = Img::new(SCREEN_W, SCREEN_H);
    let mut bytes = frame.to_rgba_bytes();
    let texture = Texture2D::from_rgba8(SCREEN_W as u16, SCREEN_H as u16, &bytes);
    texture.set_filter(FilterMode::Nearest);

    // Synthesise all audio up front.
    let mut sounds: HashMap<Sfx, Sound> = HashMap::new();
    for s in audio::ALL_SFX {
        if let Ok(snd) = load_sound_from_bytes(&audio::wav(&audio::synth(s))).await {
            sounds.insert(s, snd);
        }
    }
    let music = load_sound_from_bytes(&audio::wav(&audio::music()))
        .await
        .ok();
    let mut music_playing = false;

    // "Sharp bilinear" scaling: blow the frame up by an integer factor with nearest
    // filtering, then smoothly scale that to the window. Crisp pixels at any size.
    let mut upscaled: Option<(u32, RenderTarget)> = None;

    let mut last_mouse = mouse_position();
    let mut mouse_mode = true;
    let mut fullscreen = false;

    loop {
        let dt = get_frame_time().min(0.1);
        let lay = layout();

        // ---- Input ----
        let mut input = FrameInput::default();
        let mouse = mouse_position();
        let moved = (mouse.0 - last_mouse.0).abs() > 0.01 || (mouse.1 - last_mouse.1).abs() > 0.01;
        last_mouse = mouse;
        let left = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
        let right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
        if left || right {
            mouse_mode = false;
        } else if moved {
            mouse_mode = true;
        }
        if mouse_mode {
            input.game.paddle_x = Some((mouse.0 - lay.ox) / lay.scale - PF_X as f32);
        }
        input.game.left = left;
        input.game.right = right;
        let click = is_mouse_button_pressed(MouseButton::Left);
        let space = is_key_pressed(KeyCode::Space);
        input.game.fire = click || space || is_key_pressed(KeyCode::Up);
        input.game.fire_held =
            is_mouse_button_down(MouseButton::Left) || is_key_down(KeyCode::Space);

        if opts.autoplay {
            if app.screen == Screen::Title {
                input.confirm = true;
            } else if app.screen == Screen::Playing {
                input.game = autopilot(&app.game);
            }
        }

        let typing = app.screen == Screen::EnterName;
        let enter = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter);
        input.confirm |= enter || (!typing && (click || space));
        input.back = is_key_pressed(KeyCode::Escape);
        input.backspace = is_key_pressed(KeyCode::Backspace);
        while let Some(c) = get_char_pressed() {
            if typing && !c.is_control() {
                input.typed.push(c);
            }
        }
        if !typing {
            input.pause = is_key_pressed(KeyCode::P);
            input.lore = is_key_pressed(KeyCode::L) || is_key_pressed(KeyCode::F1);
            input.scores = is_key_pressed(KeyCode::H);
            input.toggle_music = is_key_pressed(KeyCode::M);
            input.toggle_sound = is_key_pressed(KeyCode::S);
            if is_key_pressed(KeyCode::F) || is_key_pressed(KeyCode::F11) {
                fullscreen = !fullscreen;
                set_fullscreen(fullscreen);
            }
        }

        // ---- Update ----
        let was_demo = app.showing_demo();
        let (events, from_demo) = app.update(dt, &input);
        if was_demo != app.showing_demo() {
            fx.clear();
        }
        if app.quit {
            break;
        }
        let mut played: Vec<Sfx> = Vec::new();
        for e in &events {
            fx.handle(e);
            if let stygian::game::Event::Sfx(s) = e
                && app.sound_on
                && !from_demo
                && !played.contains(s)
            {
                played.push(*s);
                if let Some(snd) = sounds.get(s) {
                    play_sound(
                        snd,
                        PlaySoundParams {
                            looped: false,
                            volume: sfx_volume(*s),
                        },
                    );
                }
            }
        }
        let paused = matches!(app.screen, Screen::Paused | Screen::EnterName);
        if !paused {
            fx.update(dt, Some(app.active_game()));
        }

        if let Some(m) = &music {
            if app.music_on && !music_playing {
                play_sound(
                    m,
                    PlaySoundParams {
                        looped: true,
                        volume: 0.45,
                    },
                );
                music_playing = true;
            } else if !app.music_on && music_playing {
                stop_sound(m);
                music_playing = false;
            }
        }

        show_mouse(!(app.screen == Screen::Playing));

        // ---- Draw ----
        renderer.draw(&mut frame, &app, &mut fx);
        frame.write_rgba_bytes(&mut bytes);
        texture.update_from_bytes(SCREEN_W as u32, SCREEN_H as u32, &bytes);
        let k = (lay.scale.ceil() as u32).clamp(1, 8);
        if upscaled.as_ref().is_none_or(|(uk, _)| *uk != k) {
            let rt = render_target(SCREEN_W as u32 * k, SCREEN_H as u32 * k);
            rt.texture.set_filter(FilterMode::Linear);
            upscaled = Some((k, rt));
        }
        let (_, rt) = upscaled.as_ref().unwrap();
        let (uw, uh) = ((SCREEN_W as u32 * k) as f32, (SCREEN_H as u32 * k) as f32);
        set_camera(&Camera2D {
            render_target: Some(rt.clone()),
            ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, uw, uh))
        });
        clear_background(BLACK);
        draw_texture_ex(
            &texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(uw, uh)),
                ..Default::default()
            },
        );
        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &rt.texture,
            lay.ox,
            lay.oy,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    SCREEN_W as f32 * lay.scale,
                    SCREEN_H as f32 * lay.scale,
                )),
                flip_y: true,
                ..Default::default()
            },
        );
        elapsed += dt;
        if let Some(path) = &opts.screenshot
            && elapsed >= opts.after
        {
            get_screen_data().export_png(path);
            println!("saved {path}");
            break;
        }
        next_frame().await;
    }
}
