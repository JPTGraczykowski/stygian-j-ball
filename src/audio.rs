//! Procedural sound: every effect and the ambient music loop are synthesised at
//! startup and handed to the audio backend as in-memory WAV files.

use crate::game::Sfx;
use crate::rng::Rng;
use std::f32::consts::TAU;

pub const RATE: u32 = 44_100;

fn secs(n: f32) -> usize {
    (n * RATE as f32) as usize
}

fn midi(n: f32) -> f32 {
    440.0 * 2f32.powf((n - 69.0) / 12.0)
}

/// Encode mono f32 samples (-1..1) as a 16-bit PCM WAV file.
pub fn wav(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

/// One-pole low-pass filter state.
struct Lp(f32);
impl Lp {
    fn run(&mut self, x: f32, cutoff: f32) -> f32 {
        let a = 1.0 - (-TAU * cutoff / RATE as f32).exp();
        self.0 += a * (x - self.0);
        self.0
    }
}

fn tri(phase: f32) -> f32 {
    let p = phase.fract();
    4.0 * (p - 0.5).abs() - 1.0
}

fn square(phase: f32) -> f32 {
    if phase.fract() < 0.5 { 1.0 } else { -1.0 }
}

fn normalize(mut v: Vec<f32>, peak: f32) -> Vec<f32> {
    let m = v.iter().fold(0.0f32, |a, s| a.max(s.abs()));
    if m > 0.0 {
        for s in &mut v {
            *s *= peak / m;
        }
    }
    // Short fade-out to avoid clicks.
    let n = v.len().min(secs(0.005));
    let len = v.len();
    for i in 0..n {
        v[len - 1 - i] *= i as f32 / n as f32;
    }
    v
}

/// Karplus-Strong plucked string.
fn pluck(freq: f32, dur: f32, damp: f32, rng: &mut Rng) -> Vec<f32> {
    let period = (RATE as f32 / freq) as usize;
    let mut buf: Vec<f32> = (0..period.max(2)).map(|_| rng.range(-1.0, 1.0)).collect();
    let mut out = Vec::with_capacity(secs(dur));
    let mut i = 0;
    for _ in 0..secs(dur) {
        let j = (i + 1) % buf.len();
        let v = (buf[i] + buf[j]) * 0.5 * damp;
        out.push(buf[i]);
        buf[i] = v;
        i = j;
    }
    out
}

fn tone_seq(notes: &[(f32, f32)], wave: fn(f32) -> f32, decay: f32) -> Vec<f32> {
    let mut out = Vec::new();
    for &(note, dur) in notes {
        let f = midi(note);
        let mut ph = 0.0;
        for i in 0..secs(dur) {
            let t = i as f32 / RATE as f32;
            ph += f / RATE as f32;
            let env = (-t * decay).exp() * (t * 400.0).min(1.0);
            out.push(wave(ph) * env);
        }
    }
    out
}

pub fn synth(sfx: Sfx) -> Vec<f32> {
    let mut rng = Rng::new(sfx as u64 + 1);
    let mut lp = Lp(0.0);
    let mut lp2 = Lp(0.0);
    let v: Vec<f32> = match sfx {
        Sfx::Paddle => (0..secs(0.13))
            .map(|i| {
                let t = i as f32 / RATE as f32;
                let f = 170.0 - 60.0 * (t / 0.13);
                let body = (TAU * f * t).sin() * (-t * 28.0).exp();
                let knock = lp.run(rng.range(-1.0, 1.0), 1800.0) * (-t * 90.0).exp() * 1.4;
                body + knock
            })
            .collect(),
        Sfx::Wall => (0..secs(0.05))
            .map(|i| {
                let t = i as f32 / RATE as f32;
                (TAU * 820.0 * t).sin() * (-t * 90.0).exp() * 0.5
                    + lp.run(rng.range(-1.0, 1.0), 2500.0) * (-t * 140.0).exp()
            })
            .collect(),
        Sfx::BrickHit => (0..secs(0.09))
            .map(|i| {
                let t = i as f32 / RATE as f32;
                let n = rng.range(-1.0, 1.0);
                let band = lp.run(n, 4000.0) - lp2.run(n, 700.0);
                band * (-t * 55.0).exp() * 1.6 + square(620.0 * t) * (-t * 70.0).exp() * 0.15
            })
            .collect(),
        Sfx::BrickBreak => {
            let mut grain = 0.0;
            (0..secs(0.28))
                .map(|i| {
                    let t = i as f32 / RATE as f32;
                    if rng.chance(0.004) {
                        grain = rng.range(0.5, 1.0);
                    }
                    grain *= 0.9993;
                    let n = lp.run(rng.range(-1.0, 1.0), 2600.0 - t * 6000.0);
                    let thud = (TAU * (95.0 - t * 120.0) * t).sin() * (-t * 22.0).exp();
                    n * (0.4 + grain) * (-t * 11.0).exp() * 1.5 + thud * 0.7
                })
                .collect()
        }
        Sfx::Clang => (0..secs(0.5))
            .map(|i| {
                let t = i as f32 / RATE as f32;
                let partials = [
                    (523.0, 9.0, 1.0),
                    (1307.0, 12.0, 0.6),
                    (2213.0, 16.0, 0.4),
                    (3121.0, 22.0, 0.25),
                ];
                partials
                    .iter()
                    .map(|(f, d, a)| (TAU * f * t).sin() * (-t * d).exp() * a)
                    .sum::<f32>()
                    + lp.run(rng.range(-1.0, 1.0), 6000.0) * (-t * 200.0).exp()
            })
            .collect(),
        Sfx::Explosion => {
            let mut brown = 0.0f32;
            (0..secs(0.8))
                .map(|i| {
                    let t = i as f32 / RATE as f32;
                    brown = (brown + rng.range(-1.0, 1.0) * 0.08).clamp(-1.0, 1.0) * 0.998;
                    let cut = 1800.0 * (-t * 4.0).exp() + 120.0;
                    let rumble = lp.run(brown * 6.0 + rng.range(-0.3, 0.3), cut);
                    let bang = (TAU * 60.0 * t).sin() * (-t * 10.0).exp();
                    (rumble * 1.2 + bang * 0.8) * (-t * 4.0).exp() * (t * 300.0).min(1.0)
                })
                .collect()
        }
        Sfx::PowerGood => {
            let mut v = tone_seq(
                &[(72.0, 0.06), (76.0, 0.06), (79.0, 0.06), (84.0, 0.22)],
                tri,
                7.0,
            );
            for (i, s) in v.iter_mut().enumerate() {
                let t = i as f32 / RATE as f32;
                *s += (TAU * 2093.0 * t).sin() * 0.12 * (t * 30.0).sin().abs() * (-t * 3.0).exp();
            }
            v
        }
        Sfx::PowerBad => tone_seq(
            &[(64.0, 0.1), (60.0, 0.1), (57.0, 0.1), (52.0, 0.3)],
            |p| square(p) * 0.5 + tri(p * 0.5) * 0.5,
            5.0,
        )
        .into_iter()
        .map(|s| lp.run(s, 1400.0))
        .collect(),
        Sfx::Bolt => {
            let mut v = pluck(196.0, 0.3, 0.985, &mut rng);
            for (i, s) in v.iter_mut().enumerate().take(secs(0.01)) {
                *s += rng.range(-1.0, 1.0) * (1.0 - i as f32 / secs(0.01) as f32);
            }
            v.into_iter().map(|s| lp.run(s, 3500.0)).collect()
        }
        Sfx::Launch => (0..secs(0.14))
            .map(|i| {
                let t = i as f32 / RATE as f32;
                let f = 280.0 + 1800.0 * t;
                tri(f * t) * (-t * 14.0).exp() * 0.6
                    + lp.run(rng.range(-1.0, 1.0), 1200.0 + t * 9000.0)
                        * (t * 10.0).min(1.0)
                        * (-t * 20.0).exp()
            })
            .collect(),
        Sfx::LifeLost => {
            let mut ph = 0.0;
            (0..secs(1.0))
                .map(|i| {
                    let t = i as f32 / RATE as f32;
                    let f = 440.0 * (-t * 1.6).exp() + 60.0;
                    ph += f / RATE as f32;
                    let trem = 0.7 + 0.3 * (TAU * 9.0 * t).sin();
                    (tri(ph) * 0.8 + square(ph * 0.5) * 0.2) * trem * (-t * 2.2).exp()
                })
                .map(|s| lp.run(s, 2000.0))
                .collect()
        }
        Sfx::LevelClear => {
            let mut v = tone_seq(
                &[
                    (60.0, 0.11),
                    (64.0, 0.11),
                    (67.0, 0.11),
                    (72.0, 0.11),
                    (67.0, 0.08),
                ],
                |p| square(p) * 0.35 + tri(p) * 0.65,
                4.0,
            );
            let start = v.len();
            v.resize(start + secs(0.8), 0.0);
            for (k, n) in [72.0, 76.0, 79.0].iter().enumerate() {
                let f = midi(*n);
                for i in 0..secs(0.8) {
                    let t = i as f32 / RATE as f32;
                    v[start + i] += (tri(f * t + k as f32 * 0.1) * 0.6
                        + (TAU * f * 2.0 * t).sin() * 0.15)
                        * (-t * 2.5).exp();
                }
            }
            v.into_iter().map(|s| lp.run(s, 5000.0)).collect()
        }
        Sfx::GameOver => {
            let mut v = Vec::new();
            for (n, d) in [(57.0, 0.45), (53.0, 0.45), (50.0, 1.2)] {
                let f = midi(n);
                for i in 0..secs(d) {
                    let t = i as f32 / RATE as f32;
                    let organ = (TAU * f * t).sin()
                        + 0.5 * (TAU * f * 2.0 * t).sin()
                        + 0.25 * (TAU * f * 3.0 * t).sin();
                    v.push(organ * (t * 40.0).min(1.0) * (-t * 1.5).exp());
                }
            }
            v
        }
    };
    let peak = match sfx {
        Sfx::Wall => 0.25,
        Sfx::Paddle | Sfx::BrickHit => 0.55,
        Sfx::Explosion | Sfx::LevelClear | Sfx::GameOver => 0.8,
        _ => 0.6,
    };
    normalize(v, peak)
}

pub const ALL_SFX: [Sfx; 13] = [
    Sfx::Paddle,
    Sfx::Wall,
    Sfx::BrickHit,
    Sfx::BrickBreak,
    Sfx::Clang,
    Sfx::Explosion,
    Sfx::PowerGood,
    Sfx::PowerBad,
    Sfx::Bolt,
    Sfx::Launch,
    Sfx::LifeLost,
    Sfx::LevelClear,
    Sfx::GameOver,
];

/// A brooding dungeon loop in D dorian: a low drone, a slow pad and sparse plucked
/// melody with echoes. Exactly loopable.
pub fn music() -> Vec<f32> {
    let bpm = 64.0;
    let beat = 60.0 / bpm;
    let bars = 8;
    let len = secs(beat * 4.0 * bars as f32);
    let mut out = vec![0.0f32; len];
    let mut rng = Rng::new(1066);

    // Drone: D2 + A2, gently breathing. Frequencies are snapped to whole cycles per
    // loop so the seam is inaudible.
    let loop_secs = len as f32 / RATE as f32;
    let snap = |f: f32| (f * loop_secs).round() / loop_secs;
    let mut lp = Lp(0.0);
    for (i, s) in out.iter_mut().enumerate() {
        let t = i as f32 / RATE as f32;
        let breathe = 0.75 + 0.25 * (TAU * t / (beat * 8.0)).sin();
        let d = snap(midi(38.0));
        let a = snap(midi(45.0));
        let saw = |f: f32| 2.0 * (f * t).fract() - 1.0;
        let raw = saw(d) * 0.5 + saw(snap(d * 1.003)) * 0.5 + saw(a) * 0.35;
        *s += lp.run(raw, 260.0) * 0.32 * breathe;
    }

    // Pad chords, one per two bars: Dm, C, Bb, A(sus).
    let chords: [[f32; 3]; 4] = [
        [62.0, 65.0, 69.0],
        [60.0, 64.0, 67.0],
        [58.0, 62.0, 65.0],
        [57.0, 62.0, 64.0],
    ];
    let chord_len = secs(beat * 8.0);
    for (c, chord) in chords.iter().enumerate() {
        for n in chord {
            let f = midi(*n - 12.0);
            let mut lp = Lp(0.0);
            for i in 0..chord_len {
                let t = i as f32 / RATE as f32;
                let env = (t / 1.5).min(1.0) * ((chord_len - i) as f32 / secs(1.5) as f32).min(1.0);
                let v = tri(f * t) * 0.6 + tri(f * 1.004 * t) * 0.4;
                out[c * chord_len + i] += lp.run(v, 900.0) * env * 0.09;
            }
        }
    }

    // Sparse lute melody (D dorian), with a feedback echo that wraps around the loop.
    let scale = [62.0, 64.0, 65.0, 67.0, 69.0, 71.0, 72.0, 74.0];
    let mut dry = vec![0.0f32; len];
    let mut step = 0.0;
    let mut deg: i32 = 4;
    while step < (bars * 4) as f32 {
        if rng.chance(0.62) {
            deg = (deg + rng.below(5) as i32 - 2).clamp(0, scale.len() as i32 - 1);
            let note = pluck(midi(scale[deg as usize]), beat * 2.0, 0.996, &mut rng);
            let start = secs(step * beat);
            for (k, s) in note.iter().enumerate() {
                dry[(start + k) % len] += s * 0.22;
            }
        }
        step += if rng.chance(0.3) { 0.5 } else { 1.0 };
    }
    let delay = secs(beat * 0.75);
    let mut wet = dry.clone();
    for _ in 0..2 {
        for i in 0..len {
            let j = (i + len - delay) % len;
            wet[i] = dry[i] + wet[j] * 0.45;
        }
    }
    for i in 0..len {
        out[i] += wet[i];
    }
    normalize(out, 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_sounds_are_valid_and_audible() {
        for s in ALL_SFX {
            let v = synth(s);
            assert!(!v.is_empty(), "{s:?}");
            assert!(v.iter().all(|x| x.is_finite() && x.abs() <= 1.0), "{s:?}");
            assert!(v.iter().any(|x| x.abs() > 0.1), "{s:?} is silent");
            assert!(v.len() < secs(3.0));
        }
    }

    #[test]
    fn wav_header_is_correct() {
        let w = wav(&[0.0, 0.5, -0.5]);
        assert_eq!(&w[0..4], b"RIFF");
        assert_eq!(&w[8..12], b"WAVE");
        assert_eq!(w.len(), 44 + 6);
        assert_eq!(u32::from_le_bytes([w[24], w[25], w[26], w[27]]), RATE);
    }

    #[test]
    fn music_loops() {
        let m = music();
        assert!(m.len() > secs(20.0));
        assert!(m.iter().all(|x| x.is_finite()));
    }
}
