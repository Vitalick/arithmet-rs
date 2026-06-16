pub const INPUT_CURSOR: [char; 4] = ['-', '\\', '|', '/'];

pub fn spinner_cursor() -> String {
    // tick - 250ms
    let ticks = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        / 250;
    let cursor_index = ticks % INPUT_CURSOR.len() as u128;
    INPUT_CURSOR[cursor_index as usize].to_string()
}

#[allow(dead_code)]
pub fn blinking_block_cursor() -> String {
    // tick - 500ms
    let ticks = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        / 500;
    if ticks % 2 == 0 {
        return "█".to_string();
    }
    " ".to_string()
}