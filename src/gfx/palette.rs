//! A restricted, earthy VGA-style palette in the spirit of early-90s dungeon crawlers.
//! Every texture is quantised onto one of these ramps (dark to light) with dithering.

use super::canvas::{Rgba, rgb};

pub const BLACK: Rgba = rgb(0x000000);
pub const INK: Rgba = rgb(0x0b0806);
pub const WHITE: Rgba = rgb(0xfff8e8);

/// Warm dungeon stone, lit by torchlight.
pub const WALL_WARM: [Rgba; 11] = [
    rgb(0x050403),
    rgb(0x0d0a07),
    rgb(0x17110c),
    rgb(0x221a12),
    rgb(0x2f2419),
    rgb(0x3d3022),
    rgb(0x4d3d2c),
    rgb(0x5f4c38),
    rgb(0x745e46),
    rgb(0x8a7257),
    rgb(0xa38a6c),
];

/// The same stone under the orb's cold magical light.
pub const WALL_COOL: [Rgba; 11] = [
    rgb(0x040406),
    rgb(0x0a0b10),
    rgb(0x12141b),
    rgb(0x1b1e28),
    rgb(0x252a36),
    rgb(0x303746),
    rgb(0x3d4558),
    rgb(0x4c566c),
    rgb(0x5d6982),
    rgb(0x717e9a),
    rgb(0x8a98b4),
];

/// Carved frame stone (brighter and greyer than the dungeon wall).
pub const FRAME: [Rgba; 9] = [
    rgb(0x0e0c0a),
    rgb(0x1f1b17),
    rgb(0x2f2a24),
    rgb(0x403a32),
    rgb(0x544c42),
    rgb(0x6a6154),
    rgb(0x807668),
    rgb(0x9a8f7f),
    rgb(0xb6ab99),
];

pub const GREY_STONE: [Rgba; 6] = [
    rgb(0x2a2724),
    rgb(0x423d38),
    rgb(0x5b554e),
    rgb(0x767067),
    rgb(0x948c81),
    rgb(0xb4ac9f),
];
pub const SANDSTONE: [Rgba; 6] = [
    rgb(0x3a2a16),
    rgb(0x5a4124),
    rgb(0x7c5b34),
    rgb(0x9f7a48),
    rgb(0xbf9a62),
    rgb(0xdcbc84),
];
pub const MOSS: [Rgba; 6] = [
    rgb(0x1b2412),
    rgb(0x2c3a1c),
    rgb(0x3f5128),
    rgb(0x566a36),
    rgb(0x718748),
    rgb(0x8fa65e),
];
pub const CLAY: [Rgba; 6] = [
    rgb(0x34110b),
    rgb(0x521c12),
    rgb(0x73291a),
    rgb(0x963a24),
    rgb(0xb75432),
    rgb(0xd47a50),
];
pub const WOOD: [Rgba; 6] = [
    rgb(0x231509),
    rgb(0x3a2410),
    rgb(0x553519),
    rgb(0x724824),
    rgb(0x8f5f31),
    rgb(0xae7a45),
];
pub const SLATE: [Rgba; 6] = [
    rgb(0x161c26),
    rgb(0x232c3a),
    rgb(0x334052),
    rgb(0x46566c),
    rgb(0x5d7088),
    rgb(0x7b90a8),
];
pub const IRON: [Rgba; 7] = [
    rgb(0x111316),
    rgb(0x1f2327),
    rgb(0x30363c),
    rgb(0x464e56),
    rgb(0x616b74),
    rgb(0x8a949c),
    rgb(0xc4ccd2),
];
pub const CRYSTAL: [Rgba; 7] = [
    rgb(0x1e0f2c),
    rgb(0x34194c),
    rgb(0x4e2a70),
    rgb(0x6c4094),
    rgb(0x8f62b8),
    rgb(0xb892da),
    rgb(0xe6d0fa),
];
pub const GOLD: [Rgba; 7] = [
    rgb(0x2e1d06),
    rgb(0x553609),
    rgb(0x83570f),
    rgb(0xb07e1a),
    rgb(0xd6a62c),
    rgb(0xf0cc52),
    rgb(0xfff0a8),
];
pub const RUNE_GLOW: [Rgba; 6] = [
    rgb(0x3c0804),
    rgb(0x721408),
    rgb(0xae2a0c),
    rgb(0xe25414),
    rgb(0xff9030),
    rgb(0xffd890),
];
pub const FIRE: [Rgba; 7] = [
    rgb(0x3a0a02),
    rgb(0x7a1a04),
    rgb(0xb83808),
    rgb(0xe8640e),
    rgb(0xff9a24),
    rgb(0xffd060),
    rgb(0xfff6c8),
];
pub const PARCHMENT: [Rgba; 7] = [
    rgb(0x5a4024),
    rgb(0x7a5c36),
    rgb(0x9c7c4c),
    rgb(0xb89a64),
    rgb(0xcfb47c),
    rgb(0xe0c994),
    rgb(0xeedcac),
];
pub const BLOOD: [Rgba; 6] = [
    rgb(0x200404),
    rgb(0x480808),
    rgb(0x780c0c),
    rgb(0xa81814),
    rgb(0xd83024),
    rgb(0xff6a50),
];
pub const MANA: [Rgba; 6] = [
    rgb(0x060a24),
    rgb(0x0c1848),
    rgb(0x18307c),
    rgb(0x2a50b0),
    rgb(0x4a80e0),
    rgb(0x9cc4ff),
];
pub const VERDANT: [Rgba; 6] = [
    rgb(0x061a06),
    rgb(0x0c3a0e),
    rgb(0x166018),
    rgb(0x268c22),
    rgb(0x48b836),
    rgb(0x9ce67a),
];
pub const ORB: [Rgba; 5] = [
    rgb(0x24406a),
    rgb(0x4878b8),
    rgb(0x84b4ec),
    rgb(0xc8e4ff),
    rgb(0xffffff),
];
pub const GHOST: [Rgba; 5] = [
    rgb(0x1a4a3a),
    rgb(0x2e7a5e),
    rgb(0x56b48a),
    rgb(0x9ae4bc),
    rgb(0xe4fff0),
];

/// Text colours.
pub const SCROLL_INK: Rgba = rgb(0x3a210c);
pub const SCROLL_INK_FADED: Rgba = rgb(0x7a5a34);
pub const ENGRAVED_GOLD: Rgba = rgb(0xe0b440);
pub const ENGRAVED_DARK: Rgba = rgb(0x1a1208);
