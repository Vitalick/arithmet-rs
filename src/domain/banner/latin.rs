use super::Glyph;
use super::cyrillic::{
    CYR_A, CYR_E, CYR_H, CYR_K, CYR_M, CYR_N, CYR_O, CYR_R, CYR_S, CYR_T, CYR_V,
};

pub(super) const LATIN_A: Glyph = CYR_A;
pub(super) const LATIN_B: Glyph = CYR_V;
pub(super) const LATIN_C: Glyph = CYR_S;

#[rustfmt::skip]
pub(super) const LATIN_D: Glyph = [
    "███ ",
    "█  █",
    "█  █",
    "█  █",
    "███ ",
];

pub(super) const LATIN_E: Glyph = CYR_E;

#[rustfmt::skip]
pub(super) const LATIN_F: Glyph = [
    "████",
    "█   ",
    "███ ",
    "█   ",
    "█   ",
];

#[rustfmt::skip]
pub(super) const LATIN_G: Glyph = [
    " ███",
    "█   ",
    "█ ██",
    "█  █",
    " ███",
];
pub(super) const LATIN_H: Glyph = CYR_N;

#[rustfmt::skip]
pub(super) const LATIN_I: Glyph = [
    "███",
    " █ ",
    " █ ",
    " █ ",
    "███",
];

#[rustfmt::skip]
pub(super) const LATIN_J: Glyph = [
    "  ██",
    "   █",
    "   █",
    "█  █",
    " ██ ",
];
pub(super) const LATIN_K: Glyph = CYR_K;

#[rustfmt::skip]
pub(super) const LATIN_L: Glyph = [
    "█   ",
    "█   ",
    "█   ",
    "█   ",
    "████",
];
pub(super) const LATIN_M: Glyph = CYR_M;

#[rustfmt::skip]
pub(super) const LATIN_N: Glyph = [
    "█  █",
    "██ █",
    "█ ██",
    "█  █",
    "█  █",
];
pub(super) const LATIN_O: Glyph = CYR_O;
pub(super) const LATIN_P: Glyph = CYR_R;

#[rustfmt::skip]
pub(super) const LATIN_Q: Glyph = [
    " ██ ",
    "█  █",
    "█  █",
    "█ ██",
    " ███",
];

#[rustfmt::skip]
pub(super) const LATIN_R: Glyph = [
    "███ ",
    "█  █",
    "███ ",
    "█ █ ",
    "█  █",
];

#[rustfmt::skip]
pub(super) const LATIN_S: Glyph = [
    " ███",
    "█   ",
    " ██ ",
    "   █",
    "███ ",
];
pub(super) const LATIN_T: Glyph = CYR_T;

#[rustfmt::skip]
pub(super) const LATIN_U: Glyph = [
    "█  █",
    "█  █",
    "█  █",
    "█  █",
    " ██ ",
];

#[rustfmt::skip]
pub(super) const LATIN_V: Glyph = [
    "█   █",
    "█   █",
    " █ █ ",
    " █ █ ",
    "  █  ",
];

#[rustfmt::skip]
pub(super) const LATIN_W: Glyph = [
    "█   █",
    "█   █",
    "█ █ █",
    "██ ██",
    "█   █",
];
pub(super) const LATIN_X: Glyph = CYR_H;

#[rustfmt::skip]
pub(super) const LATIN_Y: Glyph = [
    "█   █",
    " █ █ ",
    "  █  ",
    "  █  ",
    "  █  ",
];

#[rustfmt::skip]
pub(super) const LATIN_Z: Glyph = [
    "████",
    "   █",
    " ██ ",
    "█   ",
    "████",
];
