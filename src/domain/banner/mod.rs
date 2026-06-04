pub mod cyrillic;
pub mod latin;
pub mod symbols;

use cyrillic::*;
use latin::*;
use ratatui::prelude::{Line, Text};
use ratatui::widgets::Paragraph;
use symbols::*;

pub const HEIGHT: usize = 7;

pub(super) type Glyph = [&'static str; HEIGHT];

const LETTER_GAP: usize = 1;
const WORD_GAP: usize = 3;

pub fn render(text: &str) -> Vec<String> {
    let mut rows = vec![String::new(); HEIGHT];
    let mut pending_word_gap = false;
    let mut has_visible_glyph = false;

    for ch in text.chars().flat_map(char::to_uppercase) {
        if ch.is_whitespace() {
            pending_word_gap = has_visible_glyph;
            continue;
        }

        if has_visible_glyph {
            append_spaces(
                &mut rows,
                if pending_word_gap {
                    WORD_GAP
                } else {
                    LETTER_GAP
                },
            );
        }

        append_glyph(&mut rows, glyph_for(ch).unwrap_or(&SYMBOL_QUESTION));
        has_visible_glyph = true;
        pending_word_gap = false;
    }

    rows
}

pub fn render_to_paragraph(text: &str) -> Paragraph<'_> {
    Paragraph::new(Text::from(
        render(text)
            .iter()
            .map(|x| Line::from(x.to_string()))
            .collect::<Vec<_>>(),
    ))
}

fn append_spaces(rows: &mut [String], count: usize) {
    for row in rows {
        row.push_str(&" ".repeat(count));
    }
}

fn append_glyph(rows: &mut [String], glyph: &Glyph) {
    for (row, glyph_row) in rows.iter_mut().zip(glyph.iter()) {
        row.push_str(glyph_row);
    }
}

fn glyph_for(ch: char) -> Option<&'static Glyph> {
    match ch {
        'А' => Some(&CYR_A),
        'Б' => Some(&CYR_B),
        'В' => Some(&CYR_V),
        'Г' => Some(&CYR_G),
        'Д' => Some(&CYR_D),
        'Е' => Some(&CYR_E),
        'Ё' => Some(&CYR_YO),
        'Ж' => Some(&CYR_ZH),
        'З' => Some(&CYR_Z),
        'И' => Some(&CYR_I),
        'Й' => Some(&CYR_SHORT_I),
        'К' => Some(&CYR_K),
        'Л' => Some(&CYR_L),
        'М' => Some(&CYR_M),
        'Н' => Some(&CYR_N),
        'О' => Some(&CYR_O),
        'П' => Some(&CYR_P),
        'Р' => Some(&CYR_R),
        'С' => Some(&CYR_S),
        'Т' => Some(&CYR_T),
        'У' => Some(&CYR_U),
        'Ф' => Some(&CYR_F),
        'Х' => Some(&CYR_H),
        'Ц' => Some(&CYR_TS),
        'Ч' => Some(&CYR_CH),
        'Ш' => Some(&CYR_SH),
        'Щ' => Some(&CYR_SHCH),
        'Ъ' => Some(&CYR_HARD_SIGN),
        'Ы' => Some(&CYR_YI),
        'Ь' => Some(&CYR_SOFT_SIGN),
        'Э' => Some(&CYR_YE),
        'Ю' => Some(&CYR_YU),
        'Я' => Some(&CYR_YA),
        'A' => Some(&LATIN_A),
        'B' => Some(&LATIN_B),
        'C' => Some(&LATIN_C),
        'D' => Some(&LATIN_D),
        'E' => Some(&LATIN_E),
        'F' => Some(&LATIN_F),
        'G' => Some(&LATIN_G),
        'H' => Some(&LATIN_H),
        'I' => Some(&LATIN_I),
        'J' => Some(&LATIN_J),
        'K' => Some(&LATIN_K),
        'L' => Some(&LATIN_L),
        'M' => Some(&LATIN_M),
        'N' => Some(&LATIN_N),
        'O' => Some(&LATIN_O),
        'P' => Some(&LATIN_P),
        'Q' => Some(&LATIN_Q),
        'R' => Some(&LATIN_R),
        'S' => Some(&LATIN_S),
        'T' => Some(&LATIN_T),
        'U' => Some(&LATIN_U),
        'V' => Some(&LATIN_V),
        'W' => Some(&LATIN_W),
        'X' => Some(&LATIN_X),
        'Y' => Some(&LATIN_Y),
        'Z' => Some(&LATIN_Z),
        '0' => Some(&SYMBOL_ZERO),
        '1' => Some(&SYMBOL_ONE),
        '2' => Some(&SYMBOL_TWO),
        '3' => Some(&SYMBOL_THREE),
        '4' => Some(&SYMBOL_FOUR),
        '5' => Some(&SYMBOL_FIVE),
        '6' => Some(&SYMBOL_SIX),
        '7' => Some(&SYMBOL_SEVEN),
        '8' => Some(&SYMBOL_EIGHT),
        '9' => Some(&SYMBOL_NINE),
        '.' => Some(&SYMBOL_DOT),
        ',' => Some(&SYMBOL_COMMA),
        ':' => Some(&SYMBOL_COLON),
        ';' => Some(&SYMBOL_SEMICOLON),
        '!' => Some(&SYMBOL_EXCLAMATION),
        '?' => Some(&SYMBOL_QUESTION),
        '@' => Some(&SYMBOL_AT),
        '-' => Some(&SYMBOL_MINUS),
        '+' => Some(&SYMBOL_PLUS),
        '*' => Some(&SYMBOL_ASTERISK),
        '/' => Some(&SYMBOL_SLASH),
        '%' => Some(&SYMBOL_PERCENT),
        '=' => Some(&SYMBOL_EQUALS),
        '(' => Some(&SYMBOL_LEFT_PAREN),
        ')' => Some(&SYMBOL_RIGHT_PAREN),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_uppercase_letters() {
        assert_eq!(render("ваша"), render("ВАША"));
        assert_eq!(render("arithmet"), render("ARITHMET"));
    }

    #[test]
    fn supports_russian_english_and_digits() {
        for ch in "АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars()
        {
            assert!(glyph_for(ch).is_some(), "missing glyph for {ch}");
        }
    }

    #[test]
    fn latin_y_is_not_cyrillic_u() {
        assert_ne!(render("Y"), render("У"));
    }

    #[test]
    fn cyrillic_e_is_not_cyrillic_yo() {
        assert_ne!(render("Е"), render("Ё"));
    }

    #[test]
    fn renders_descenders_on_sixth_row() {
        assert_eq!(render("Д")[6], "█    █");
        assert_eq!(render("Ц")[6], "    █");
        assert_eq!(render("Щ")[6], "     █");
        assert_eq!(render("Q")[6], "   █");
        assert_eq!(render(",")[6], "█ ");
    }

    #[test]
    fn separates_letters_with_one_space() {
        let rows = render("АБ");

        assert_eq!(rows[0], format!("{} {}", CYR_A[0], CYR_B[0]));
    }

    #[test]
    fn separates_words_with_three_spaces() {
        let rows = render("А Б");

        assert_eq!(rows[0], format!("{}   {}", CYR_A[0], CYR_B[0]));
    }

    #[test]
    fn collapses_repeated_whitespace_between_words() {
        assert_eq!(render("А   Б"), render("А Б"));
    }

    #[test]
    fn renders_final_grade_phrase() {
        let rows = render("Ваша оценка 5");

        assert_eq!(rows.len(), HEIGHT);
        assert!(rows.iter().all(|row| !row.is_empty()));
        assert!(rows[0].contains("   "));
    }

    #[test]
    fn replaces_unknown_characters_with_question_mark() {
        assert_eq!(render("$"), SYMBOL_QUESTION);
    }

    #[test]
    fn renders_at_sign() {
        assert_eq!(render("@"), SYMBOL_AT);
    }
}
