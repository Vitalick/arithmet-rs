use ratatui::prelude::Widget;
use std::cmp::min;
use std::fmt::Display;
use std::time::Duration;

pub trait Cursor {
    fn cursor(&self) -> String;
}


#[derive(Debug, Clone, Copy)]
pub enum CursorType {
    BlinkingBlock,
    Spinner,
}

impl Cursor for CursorType {
    fn cursor(&self) -> String {
        match self {
            CursorType::BlinkingBlock => Self::cursor_from_array(&Self::BLINKING_BLOCK_CURSOR),
            CursorType::Spinner => Self::cursor_from_array(&Self::SPINNER_CURSOR),
        }
    }
}

impl Display for CursorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cursor())
    }
}

impl Display for dyn Cursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cursor())
    }
}

impl CursorType {
    const FULL_ANIMATION_LENGTH_MS: u128 = 720;
    const SPINNER_CURSOR: [char; 4] = ['-', '\\', '|', '/'];
    const BLINKING_BLOCK_CURSOR: [char; 2] = ['█', ' '];

    fn frame_index(frame_count: usize) -> usize {
        let one_animation_cycle_ms = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            % Self::FULL_ANIMATION_LENGTH_MS);
        let one_frame_ms = Self::FULL_ANIMATION_LENGTH_MS / frame_count as u128;
        min(
            (one_animation_cycle_ms / one_frame_ms) as usize,
            frame_count - 1,
        )
    }

    fn cursor_from_array(cursor_array: &[char]) -> String {
        cursor_array[Self::frame_index(cursor_array.len())].to_string()
    }
}
