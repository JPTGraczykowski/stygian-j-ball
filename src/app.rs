//! Screen flow (title, play, pause, name entry, hall of heroes, lore) and persistence.
//! Free of any windowing code so it can be driven headlessly.

use crate::consts::*;
use crate::game::{Event, Game, Input, Phase};
use crate::storage;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Screen {
    Title,
    Lore,
    Scores,
    Playing,
    Paused,
    /// Brief pause on the game-over/victory banner before moving on.
    Ending(f32),
    EnterName,
}

#[derive(Clone, Debug, Default)]
pub struct FrameInput {
    pub game: Input,
    /// Enter / Space / left click.
    pub confirm: bool,
    pub back: bool,
    pub pause: bool,
    pub lore: bool,
    pub scores: bool,
    pub typed: Vec<char>,
    pub backspace: bool,
    pub toggle_sound: bool,
    pub toggle_music: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScoreEntry {
    pub name: String,
    pub score: u64,
    pub level: u32,
}

pub const MAX_SCORES: usize = 8;
pub const MAX_NAME: usize = 12;
const MAX_LOG: usize = 24;

pub struct App {
    pub screen: Screen,
    pub game: Game,
    pub demo: Game,
    /// Message scroll, newest last: (text, age in seconds).
    pub log: Vec<(String, f32)>,
    pub scores: Vec<ScoreEntry>,
    pub name: String,
    pub highlight: Option<usize>,
    pub scores_path: Option<PathBuf>,
    pub sound_on: bool,
    pub music_on: bool,
    pub quit: bool,
    pub time: f32,
    seed: u64,
}

#[cfg(target_arch = "wasm32")]
pub fn default_scores_path() -> Option<PathBuf> {
    Some(PathBuf::from("stygian-j-ball/scores.tsv"))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn default_scores_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(base.join("stygian-j-ball").join("scores.tsv"))
}

pub fn parse_scores(text: &str) -> Vec<ScoreEntry> {
    let mut v: Vec<ScoreEntry> = text
        .lines()
        .filter_map(|l| {
            let mut it = l.splitn(3, '\t');
            let score = it.next()?.trim().parse().ok()?;
            let level = it.next()?.trim().parse().ok()?;
            let name: String = it.next()?.chars().take(MAX_NAME).collect();
            Some(ScoreEntry { name, score, level })
        })
        .collect();
    v.sort_by_key(|s| std::cmp::Reverse(s.score));
    v.truncate(MAX_SCORES);
    v
}

pub fn format_scores(scores: &[ScoreEntry]) -> String {
    scores
        .iter()
        .map(|s| format!("{}\t{}\t{}\n", s.score, s.level, s.name))
        .collect()
}

/// Simple paddle AI used for the title-screen attract mode (and tests).
pub fn autopilot(g: &Game) -> Input {
    let ball = g
        .balls
        .iter()
        .filter(|b| b.stuck.is_none() && b.dir.y > 0.0)
        .max_by(|a, b| a.pos.y.total_cmp(&b.pos.y))
        .or_else(|| g.balls.iter().max_by(|a, b| a.pos.y.total_cmp(&b.pos.y)));
    let target = match ball {
        Some(b) if b.stuck.is_none() => {
            // Predict where the ball meets the paddle line, folding wall bounces.
            let dy = PADDLE_Y - BALL_R - b.pos.y;
            let mut x = if b.dir.y > 0.0 {
                b.pos.x + b.dir.x / b.dir.y * dy
            } else {
                b.pos.x
            };
            let span = PF_W - 2.0 * BALL_R;
            x -= BALL_R;
            x = x.rem_euclid(2.0 * span);
            if x > span {
                x = 2.0 * span - x;
            }
            x += BALL_R;
            // Vary the strike point so the demo doesn't loop forever.
            let aim = (g.time * 0.37).sin() * g.paddle.half() * 0.7;
            Some(x - aim)
        }
        _ => Some(PF_W * 0.5 + (g.time * 0.8).sin() * 60.0),
    };
    Input {
        paddle_x: target,
        fire: g.level_time > 1.0,
        fire_held: true,
        ..Default::default()
    }
}

impl App {
    pub fn new(scores_path: Option<PathBuf>, seed: u64) -> App {
        let scores = scores_path
            .as_ref()
            .and_then(|p| storage::read(p))
            .map(|t| parse_scores(&t))
            .unwrap_or_default();
        let mut demo = Game::new(seed ^ 0xD3);
        demo.load_level(seed as usize % 4);
        demo.take_events();
        let mut app = App {
            screen: Screen::Title,
            game: Game::new(seed),
            demo,
            log: Vec::new(),
            scores,
            name: String::new(),
            highlight: None,
            scores_path,
            sound_on: true,
            music_on: true,
            quit: false,
            time: 0.0,
            seed,
        };
        app.say("Welcome, adventurer, to the Stygian Abyss.");
        app.say("Click to descend. Catch the orb upon thy plank!");
        app
    }

    pub fn say(&mut self, msg: impl Into<String>) {
        self.log.push((msg.into(), 0.0));
        if self.log.len() > MAX_LOG {
            self.log.remove(0);
        }
    }

    pub fn best_score(&self) -> u64 {
        self.scores
            .first()
            .map(|s| s.score)
            .unwrap_or(0)
            .max(self.game.score)
    }

    pub fn qualifies(&self, score: u64) -> bool {
        score > 0 && (self.scores.len() < MAX_SCORES || self.scores.iter().any(|s| score > s.score))
    }

    fn save_scores(&self) {
        if let Some(p) = &self.scores_path {
            storage::write(p, &format_scores(&self.scores));
        }
    }

    pub fn insert_score(&mut self, name: &str, score: u64, level: u32) -> usize {
        let name = if name.trim().is_empty() {
            "Nameless".to_string()
        } else {
            name.trim().to_string()
        };
        let pos = self
            .scores
            .iter()
            .position(|s| score > s.score)
            .unwrap_or(self.scores.len());
        self.scores.insert(pos, ScoreEntry { name, score, level });
        self.scores.truncate(MAX_SCORES);
        self.save_scores();
        pos
    }

    fn start_game(&mut self) {
        self.seed = self
            .seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.game = Game::new(self.seed);
        self.highlight = None;
        self.screen = Screen::Playing;
    }

    /// Whether the visible playfield shows the demo game.
    pub fn showing_demo(&self) -> bool {
        matches!(self.screen, Screen::Title | Screen::Lore | Screen::Scores)
    }

    pub fn active_game(&self) -> &Game {
        if self.showing_demo() {
            &self.demo
        } else {
            &self.game
        }
    }

    /// Advance one frame. Returns the events produced by the visible game and
    /// whether they came from the (muted) demo.
    pub fn update(&mut self, dt: f32, input: &FrameInput) -> (Vec<Event>, bool) {
        self.time += dt;
        for m in &mut self.log {
            m.1 += dt;
        }
        if input.toggle_sound {
            self.sound_on = !self.sound_on;
            let msg = if self.sound_on {
                "Sound effects on."
            } else {
                "Sound effects off."
            };
            self.say(msg);
        }
        if input.toggle_music {
            self.music_on = !self.music_on;
            let msg = if self.music_on {
                "Music on."
            } else {
                "Music off."
            };
            self.say(msg);
        }

        match self.screen {
            Screen::Title | Screen::Lore | Screen::Scores => {
                let ai = autopilot(&self.demo);
                self.demo.update(dt, &ai);
                if matches!(self.demo.phase, Phase::GameOver | Phase::Victory) {
                    let lvl = (self.demo.level + 1) % 5;
                    self.demo = Game::new(self.time.to_bits() as u64);
                    self.demo.load_level(lvl);
                }
                let events = self.demo.take_events();
                match self.screen {
                    Screen::Title => {
                        if input.back {
                            self.quit = true;
                        } else if input.lore {
                            self.screen = Screen::Lore;
                        } else if input.scores {
                            self.screen = Screen::Scores;
                        } else if input.confirm {
                            self.start_game();
                        }
                    }
                    _ => {
                        if input.back || input.confirm || input.lore || input.scores {
                            self.screen = Screen::Title;
                        }
                    }
                }
                (events, true)
            }
            Screen::Playing => {
                if input.pause || input.back {
                    self.screen = Screen::Paused;
                    self.say("Thou restest a moment. (P to continue, Esc to flee)");
                    return (Vec::new(), false);
                }
                if self.game.phase == Phase::Victory && input.confirm {
                    self.game.continue_cycle();
                    self.say("Thou descendest again, deeper and faster...");
                }
                self.game.update(dt, &input.game);
                let events = self.game.take_events();
                for e in &events {
                    if let Event::Message(m) = e {
                        self.say(m.clone());
                    }
                }
                if self.game.phase == Phase::GameOver {
                    self.screen = Screen::Ending(3.0);
                }
                if self.game.phase == Phase::Victory && input.back {
                    self.screen = Screen::Ending(0.0);
                }
                (events, false)
            }
            Screen::Paused => {
                if input.pause || input.confirm {
                    self.screen = Screen::Playing;
                } else if input.back {
                    self.say("Thou fleest the Abyss.");
                    self.screen = Screen::Ending(0.0);
                }
                (Vec::new(), false)
            }
            Screen::Ending(t) => {
                let t = t - dt;
                if t > 0.0 && !(input.confirm && t < 2.0) {
                    self.screen = Screen::Ending(t);
                } else if self.qualifies(self.game.score) {
                    self.name.clear();
                    self.screen = Screen::EnterName;
                    self.say("Thy deeds shall be remembered. Inscribe thy name.");
                } else {
                    self.screen = Screen::Scores;
                }
                (Vec::new(), false)
            }
            Screen::EnterName => {
                for &c in &input.typed {
                    if (c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '\'')
                        && self.name.chars().count() < MAX_NAME
                    {
                        self.name.push(c);
                    }
                }
                if input.backspace {
                    self.name.pop();
                }
                if input.confirm {
                    let name = self.name.clone();
                    let level = self.game.level as u32
                        + 1
                        + self.game.cycle * crate::levels::LEVELS.len() as u32;
                    let pos = self.insert_score(&name, self.game.score, level);
                    self.highlight = Some(pos);
                    self.screen = Screen::Scores;
                } else if input.back {
                    self.screen = Screen::Scores;
                }
                (Vec::new(), false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(app: &mut App, f: impl Fn(&mut FrameInput)) {
        let mut i = FrameInput::default();
        f(&mut i);
        app.update(1.0 / 60.0, &i);
    }

    #[test]
    fn scores_round_trip_and_sort() {
        let s = vec![
            ScoreEntry {
                name: "Bob".into(),
                score: 50,
                level: 2,
            },
            ScoreEntry {
                name: "Ann Lee".into(),
                score: 900,
                level: 5,
            },
        ];
        let parsed = parse_scores(&format_scores(&s));
        assert_eq!(parsed[0].name, "Ann Lee");
        assert_eq!(parsed[1].score, 50);
        assert!(parse_scores("garbage\n\t\t\n").is_empty());
    }

    #[test]
    fn title_to_play_to_pause_and_back() {
        let mut app = App::new(None, 1);
        assert_eq!(app.screen, Screen::Title);
        press(&mut app, |i| i.confirm = true);
        assert_eq!(app.screen, Screen::Playing);
        press(&mut app, |i| i.pause = true);
        assert_eq!(app.screen, Screen::Paused);
        press(&mut app, |i| i.pause = true);
        assert_eq!(app.screen, Screen::Playing);
    }

    #[test]
    fn game_over_leads_to_name_entry_and_score_table() {
        let mut app = App::new(None, 2);
        press(&mut app, |i| i.confirm = true);
        app.game.score = 1234;
        app.game.lives = 1;
        app.game.paddle.x = 20.0;
        app.game.release_balls();
        for b in &mut app.game.balls {
            b.pos = macroquad::math::vec2(250.0, 170.0);
            b.dir = macroquad::math::vec2(0.0, 1.0);
        }
        for _ in 0..400 {
            press(&mut app, |_| {});
        }
        assert_eq!(app.screen, Screen::EnterName);
        press(&mut app, |i| i.typed = "Shamino".chars().collect());
        press(&mut app, |i| i.backspace = true);
        press(&mut app, |i| i.confirm = true);
        assert_eq!(app.screen, Screen::Scores);
        assert_eq!(app.scores[0].name, "Shamin");
        assert_eq!(app.scores[0].score, 1234);
        assert_eq!(app.highlight, Some(0));
    }

    #[test]
    fn demo_runs_unattended() {
        let mut app = App::new(None, 3);
        for _ in 0..(60 * 120) {
            press(&mut app, |_| {});
        }
        assert_eq!(app.screen, Screen::Title);
        assert!(app.demo.time > 100.0);
    }
}
