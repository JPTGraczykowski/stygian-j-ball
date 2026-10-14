//! Level layouts. Each row is exactly 16 characters wide:
//!
//! `.` empty, `1`-`6` plain stone variants (1 hit), `B` iron-bound wood (2 hits),
//! `C` crystal (3 hits), `I` iron (indestructible), `R` fire rune (explodes),
//! `G` treasure (1 hit, always drops a boon).

pub struct Level {
    pub name: &'static str,
    pub rows: &'static [&'static str],
}

pub const LEVELS: &[Level] = &[
    Level {
        name: "The Great Gate",
        rows: &[
            "................",
            "................",
            "................",
            "1111111111111111",
            "2222222222222222",
            "3333333333333333",
            "4444444444444444",
            "5555555555555555",
            "6666666666666666",
        ],
    },
    Level {
        name: "Mountainmen's Mine",
        rows: &[
            "................",
            "................",
            ".......GG.......",
            "......2222......",
            ".....222222.....",
            "....55555555....",
            "...5555555555...",
            "..444444444444..",
            ".11111111111111.",
            "BBBBBBBBBBBBBBBB",
        ],
    },
    Level {
        name: "Hall of Pillars",
        rows: &[
            "................",
            "................",
            "I..1111..1111..I",
            "I..1111..1111..I",
            "I..2222..2222..I",
            "I..2222..2222..I",
            "I..3333..3333..I",
            "I..3333..3333..I",
            "I..RRRR..RRRR..I",
            "III..........III",
        ],
    },
    Level {
        name: "Lizardman Warrens",
        rows: &[
            "................",
            "................",
            "3.3.3.3.3.3.3.3.",
            ".3.3.3.3.3.3.3.3",
            "3.3.3.3.3.3.3.3.",
            ".R.R.R.R.R.R.R.R",
            "3.3.3.3.3.3.3.3.",
            ".3.3.3.3.3.3.3.3",
            "BBBBBBBBBBBBBBBB",
        ],
    },
    Level {
        name: "Crystal Caverns",
        rows: &[
            "................",
            "................",
            ".......CC.......",
            "......C66C......",
            ".....C6666C.....",
            "....C66GG66C....",
            "...C66666666C...",
            "....C666666C....",
            ".....C6666C.....",
            "......C66C......",
            ".......CC.......",
        ],
    },
    Level {
        name: "The Seer's Library",
        rows: &[
            "................",
            "................",
            "I55555555555555I",
            "I5.5.5.55.5.5.5I",
            "IBBBBBBBBBBBBBBI",
            "I22222222222222I",
            "I2.2.2.22.2.2.2I",
            "IBBBBBBBBBBBBBBI",
            "I......GG......I",
            "IIIII......IIIII",
        ],
    },
    Level {
        name: "Tomb of the Ancients",
        rows: &[
            "................",
            "................",
            "....11111111....",
            "...1111111111...",
            "..111111111111..",
            "..11..1111..11..",
            "..1.RR.11.RR.1..",
            "..11..1111..11..",
            "..111111111111..",
            "...1111..1111...",
            "....11111111....",
            ".....C.C.C.C....",
            ".....C.C.C.C....",
        ],
    },
    Level {
        name: "Knights of the Crux",
        rows: &[
            "................",
            "4.....BBBB.....4",
            "44....BBBB....44",
            "444...BBBB...444",
            "BBBBBBBBBBBBBBBB",
            "BBBBBBBGGBBBBBBB",
            "444...BBBB...444",
            "44....BBBB....44",
            "4.....BBBB.....4",
            "......BBBB......",
            "....IIIIIIII....",
        ],
    },
    Level {
        name: "Flooded Vaults",
        rows: &[
            "................",
            "................",
            "6666666666666666",
            "II....IIII....II",
            "2222222222222222",
            "....II....II....",
            "5555555555555555",
            "II....IIII....II",
            "3333333333333333",
            "...R........R...",
        ],
    },
    Level {
        name: "The Rune Forge",
        rows: &[
            "................",
            "................",
            "1R1R1R1R1R1R1R1R",
            "R1R1R1R1R1R1R1R1",
            "4444444444444444",
            "4CCCCCCCCCCCCCC4",
            "4444444444444444",
            "................",
            "I.I.I.I..I.I.I.I",
        ],
    },
    Level {
        name: "Golem Workshop",
        rows: &[
            "................",
            "IIIIIIIIIIIIIIII",
            "I44444444444444I",
            "I4CCCCCCCCCCCC4I",
            "I4C2222222222C4I",
            "I4C2GGGGGGGG2C4I",
            "I4C2222222222C4I",
            "I4CCCCCCCCCCCC4I",
            "I44444444444444I",
            "IIIIII....IIIIII",
        ],
    },
    Level {
        name: "The Ethereal Void",
        rows: &[
            "................",
            "C..............C",
            ".C..6......6..C.",
            "..C..6....6..C..",
            "...C..6RR6..C...",
            "....C..66..C....",
            "...C..6RR6..C...",
            "..C..6....6..C..",
            ".C..6......6..C.",
            "C..............C",
            "BBBB..BBBB..BBBB",
        ],
    },
    Level {
        name: "Chamber of the Slasher",
        rows: &[
            "................",
            "R..............R",
            "BB.BB.BB.BB.BB.B",
            "1111111111111111",
            "I.I.I.I.I.I.I.I.",
            "2222222222222222",
            ".I.I.I.I.I.I.I.I",
            "3333333333333333",
            "CCCCCCCGGCCCCCCC",
            "................",
            "................",
            "IIIII......IIIII",
        ],
    },
    Level {
        name: "The Abyssal Gate",
        rows: &[
            "................",
            "IIIIIIIIIIIIIIII",
            "ICCCCCCCCCCCCCCI",
            "IC444444444444CI",
            "IC4RRRRRRRRRR4CI",
            "IC4R55555555R4CI",
            "IC4R5GGGGGG5R4CI",
            "IC4R55555555R4CI",
            "IC4RRRRRRRRRR4CI",
            "IC444444444444CI",
            "ICCCCCCCCCCCCCCI",
            "IIIIII....IIIIII",
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::{COLS, ROWS};

    #[test]
    fn levels_are_well_formed() {
        for lvl in LEVELS {
            assert!(lvl.rows.len() <= ROWS, "{} has too many rows", lvl.name);
            let mut breakable = 0;
            for row in lvl.rows {
                assert_eq!(
                    row.chars().count(),
                    COLS,
                    "bad row width in {}: {row:?}",
                    lvl.name
                );
                for ch in row.chars() {
                    assert!(
                        ".123456BCIRG".contains(ch),
                        "unknown brick {ch:?} in {}",
                        lvl.name
                    );
                    if ch != '.' && ch != 'I' {
                        breakable += 1;
                    }
                }
            }
            assert!(breakable > 0, "{} has nothing to break", lvl.name);
        }
    }
}
