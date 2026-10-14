//! Balance report: lets the autopilot play and prints statistics.
use stygian::app::autopilot;
use stygian::game::{Event, Game, Phase};

fn main() {
    let mut cleared = 0;
    let mut deaths = 0;
    let mut powers = std::collections::HashMap::new();
    let mut level_times = Vec::new();
    let mut g = Game::new(22);
    let mut t = 0.0;
    let mut max_speed = 0.0f32;
    while t < 3600.0 {
        let input = autopilot(&g);
        g.update(1.0 / 60.0, &input);
        max_speed = max_speed.max(g.speed);
        for e in g.take_events() {
            match e {
                Event::LevelCleared => {
                    cleared += 1;
                    level_times.push((g.level, g.level_time as u32));
                }
                Event::PowerCollected { power, .. } => {
                    *powers.entry(format!("{power:?}")).or_insert(0) += 1
                }
                Event::Sfx(stygian::game::Sfx::LifeLost) => deaths += 1,
                _ => {}
            }
        }
        match g.phase {
            Phase::GameOver => g = Game::new(t as u64 + 1),
            Phase::Victory => g.continue_cycle(),
            _ => {}
        }
        t += 1.0 / 60.0;
    }
    println!(
        "1h autopilot: cleared {cleared} levels, {deaths} lives lost, max speed {max_speed:.0}"
    );
    println!("level (index, seconds): {level_times:?}");
    let mut p: Vec<_> = powers.into_iter().collect();
    p.sort_by_key(|x| std::cmp::Reverse(x.1));
    println!("powers caught: {p:?}");
}
