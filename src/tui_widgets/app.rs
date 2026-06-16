use color_eyre::{Result, eyre::WrapErr};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use strum::IntoEnumIterator;
use validations::Validate;

use crate::domain::answer::{Answer, AnswerError};
use crate::domain::expression::ExerciseWithStartTime;
use crate::domain::session::Session;
use crate::domain::{operation::Operation, settings::Settings};
use crate::tui_widgets::status::{Status, StatusWidget};

const CONFIG_PATH: &str = "arithmet.toml";
const HEADER_NAME: &str = "VIT";
const MAIN_AREA_HEIGHT: u16 = 16;
const STATUS_AREA_HEIGHT: u16 = 10;


const INPUT_CURSOR: [char; 4] = ['-', '\\', '|', '/'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveField {
    PlayerName,
    ResultMin,
    ResultMax,
    ExerciseCount,
    Complexity,
    GameAnswer,
}

#[derive(Debug)]
pub struct App {
    status: Status,
    settings: Settings,
    session: Option<Session>,
    exercise_now: Option<ExerciseWithStartTime>,
    answer: Option<Answer>,
    correct_answers: usize,
    active_field: Option<ActiveField>,
    input_buffer: String,
    cursor_frame: usize,
    exit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            status: Status::Welcome,
            settings: Settings::load(CONFIG_PATH).unwrap_or_default(),
            session: None,
            correct_answers: 0,
            active_field: None,
            exercise_now: None,
            answer: None,
            input_buffer: String::new(),
            cursor_frame: 0,
            exit: false,
        }
    }
}

impl App {
    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        shutdown_requested: Arc<AtomicBool>,
    ) -> Result<()> {
        while !self.exit && !shutdown_requested.load(Ordering::Relaxed) {
            self.update_status();
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        self.settings
            .save(CONFIG_PATH)
            .map_err(|err| color_eyre::eyre::eyre!(err))
            .wrap_err("save settings failed")?;
        Ok(())
    }

    fn start_game(&mut self) -> Result<(), String> {
        self.session = Some(Session::new(self.settings.clone())?);
        self.game_step()?;
        Ok(())
    }

    fn update_status(&mut self) {
        if self.session.is_none() {
            self.status = Status::Welcome;
            return;
        }
        let session = self.session.as_ref().unwrap();
        if session.is_finished() {
            self.status = Status::GameFinished;
            return;
        }
        if self.exercise_now.is_none() {
            self.status = Status::AwaitingGameContinue;
            return;
        }
        self.status = Status::AwaitingAnswer;
    }

    fn game_step(&mut self) -> Result<(), String> {
        let session = self.session.as_mut().unwrap();
        if session.have_next() {
            self.exercise_now = Some(session.next()?);
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn toggle_operation(&mut self, operation: Operation) {
        if self.settings.operations.contains(&operation) {
            if self.settings.operations.len() > 1 {
                self.settings.operations.remove(&operation);
            }
            return;
        }
        self.settings.operations.insert(operation);
    }

    fn start_input(&mut self, field: ActiveField) {
        self.active_field = Some(field);
        self.input_buffer = self.field_value(field);
    }

    fn cancel_input(&mut self) {
        self.active_field = None;
        self.input_buffer.clear();
    }

    fn commit_input(&mut self) {
        let Some(field) = self.active_field else {
            return;
        };
        let value = self.input_buffer.trim();
        let mut candidate = self.settings.clone();

        match field {
            ActiveField::PlayerName => {
                if !value.is_empty() {
                    candidate.player_name = value.to_string();
                }
            }
            ActiveField::ResultMin => {
                if let Ok(value) = value.parse() {
                    candidate.limits.result_min = value;
                } else {
                    self.cancel_input();
                    return;
                }
            }
            ActiveField::ResultMax => {
                if let Ok(value) = value.parse() {
                    candidate.limits.result_max = value;
                } else {
                    self.cancel_input();
                    return;
                }
            }
            ActiveField::ExerciseCount => {
                if let Ok(value) = value.parse() {
                    candidate.limits.exercise_count = value;
                } else {
                    self.cancel_input();
                    return;
                }
            }
            ActiveField::GameAnswer => {
                let exercise = self.exercise_now.unwrap();
                if let Ok(value) = value.parse() {
                    self.answer = Some(Answer {
                        exercise: exercise.exercise,
                        entered: Ok(value),
                        time_elapsed: exercise.start_time.elapsed(),
                    });
                } else {
                    self.answer = Some(Answer {
                        exercise: exercise.exercise,
                        entered: Err(AnswerError::InvalidInput),
                        time_elapsed: exercise.start_time.elapsed(),
                    });
                    self.cancel_input();
                    return;
                }
            }
            ActiveField::Complexity => {
                if let Ok(value) = value.parse::<u64>() {
                    candidate.limits.answer_time = Duration::from_secs(value);
                } else {
                    self.cancel_input();
                    return;
                }
            }
        }

        if candidate.validate().is_ok() {
            self.settings = candidate;
        }
        self.cancel_input();
    }

    fn field_value(&self, field: ActiveField) -> String {
        match field {
            ActiveField::PlayerName => self.settings.player_name.clone(),
            ActiveField::ResultMin => self.settings.limits.result_min.to_string(),
            ActiveField::ResultMax => self.settings.limits.result_max.to_string(),
            ActiveField::ExerciseCount => self.settings.limits.exercise_count.to_string(),
            ActiveField::Complexity => self.settings.limits.answer_time.as_secs().to_string(),
            ActiveField::GameAnswer => match self.answer.unwrap_or_default().entered {
                Ok(value) => value.to_string(),
                Err(_) => String::default(),
            },
        }
    }

    fn handle_input_key_event(&mut self, key_event: KeyEvent) -> bool {
        if self.active_field.is_none() {
            if key_event.code == KeyCode::Enter {
                self.start_game().unwrap();
                self.start_input(ActiveField::GameAnswer);
                return true;
            }
            return false;
        }

        match key_event.code {
            KeyCode::Enter => self.commit_input(),
            KeyCode::Esc => self.cancel_input(),
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Delete => self.input_buffer.clear(),
            KeyCode::Char(character) => {
                if self.active_field == Some(ActiveField::PlayerName)
                    || character.is_ascii_digit()
                    || (character == '-'
                        && self.input_buffer.is_empty()
                        && matches!(
                            self.active_field,
                            Some(ActiveField::ResultMin | ActiveField::ResultMax)
                        ))
                {
                    self.input_buffer.push(character);
                }
            }
            _ => {}
        }
        true
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if is_ctrl_c(key_event) {
            self.exit();
            return;
        }

        if self.handle_input_key_event(key_event) {
            return;
        }

        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => self.exit(),
            KeyCode::Char('+') => self.toggle_operation(Operation::Addition),
            KeyCode::Char('-') => self.toggle_operation(Operation::Subtraction),
            KeyCode::Char('*') => self.toggle_operation(Operation::Multiplication),
            KeyCode::Char('/') => self.toggle_operation(Operation::Division),
            KeyCode::Char(':') => self.toggle_operation(Operation::DivisionWithRemainder),
            KeyCode::Char(character) => {
                if let Some(field) = field_hotkey(character) {
                    self.start_input(field);
                }
            }
            _ => {}
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(120))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            }
        } else if self.active_field.is_some() {
            self.cursor_frame = (self.cursor_frame + 1) % INPUT_CURSOR.len();
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let outer = Block::bordered()
            .border_set(border::THICK)
            .title(Line::from("У С Т Н Ы Й   С Ч Е Т".blue().bold()).centered())
            .title_bottom(self.instructions().centered());

        let inner = outer.inner(area);
        outer.render(area, buf);

        let [main_area, status_area] =
            Layout::vertical([Constraint::Length(MAIN_AREA_HEIGHT), Constraint::Fill(1)])
                .areas(inner);

        self.render_main(main_area, buf);
        StatusWidget::new(self.session.clone(), self.exercise_now, self.status).render(status_area, buf);
    }
}

impl App {
    fn instructions(&self) -> Line<'static> {
        Line::from(vec![
            "<+ - * / :>".blue().bold(),
            " - действие, ".into(),
            "<И О Д К С>".blue().bold(),
            " - поля, ".into(),
            "<F1>".blue().bold(),
            " - результат, ".into(),
            "<Esc>".blue().bold(),
            " - выход, ".into(),
            "<Enter>".blue().bold(),
            " - старт".into(),
        ])
    }

    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let header = Paragraph::new(vec![
            Line::from(format!("==== версия {} ====", env!("CARGO_PKG_VERSION"))),
            Line::from(format!("{:<12}{:>12}", HEADER_NAME, 2026)),
        ])
        .centered();

        header.render(area, buf);
    }

    fn render_main(&self, area: Rect, buf: &mut Buffer) {
        let body = horizontal_inset(area, 4);
        let [left, center, right] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .spacing(6)
        .areas(body);

        self.render_actions_column(left, buf);
        self.render_center_column(center, buf);
        self.render_settings_column(right, buf);
    }

    fn render_actions_column(&self, area: Rect, buf: &mut Buffer) {
        let [_, actions] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        self.render_actions(actions, buf);
    }

    fn render_actions(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .border_set(border::PLAIN)
            .title(Line::from("Действие".bold()).centered());

        let operations = Operation::iter()
            .map(|operation| self.operation_line(operation))
            .collect::<Vec<_>>();

        Paragraph::new(operations).block(block).render(area, buf);
    }

    fn operation_line(&self, operation: Operation) -> Line<'static> {
        let checked = if self.settings.operations.contains(&operation) {
            "[x]"
        } else {
            "[ ]"
        };

        Line::from(vec![
            format!("{}  ", checked).into(),
            operation.symbol().blue().bold(),
            " ".into(),
            operation.label().to_lowercase().into(),
        ])
    }

    fn render_center_column(&self, area: Rect, buf: &mut Buffer) {
        let [header, exercise, check] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .spacing(1)
        .areas(area);

        self.render_header(header, buf);

        self.render_exercise(exercise, buf);

        self.render_check(check, buf);
    }

    fn render_exercise(&self, area: Rect, buf: &mut Buffer) {
        let exercise_block = Block::bordered()
            .border_set(border::PLAIN)
            .title(Line::from("Пример".bold()).centered())
            .title_bottom(
                Line::from(format!("Верных ответов: {}", self.correct_answers).bold()).centered(),
            );
        Paragraph::new("").block(exercise_block).render(area, buf);
    }

    fn render_check(&self, area: Rect, buf: &mut Buffer) {
        let check_block = Block::bordered()
            .border_set(border::PLAIN)
            .title(Line::from("Проверка".bold()).centered())
            .title_bottom(Line::from("Верный ответ:".bold()).centered());
        let check_text = vec![Line::from("a)"), Line::from("b)"), Line::from("")];
        Paragraph::new(check_text)
            .block(check_block)
            .render(area, buf);
    }


    fn render_settings_column(&self, area: Rect, buf: &mut Buffer) {
        let [_, settings] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        self.render_settings(settings, buf);
    }

    fn render_settings(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .border_set(border::PLAIN)
            .title(Line::from("Настройки".bold()).centered());

        Paragraph::new(vec![
            self.field_line("И", "мя: ", ActiveField::PlayerName),
            Line::from(""),
            Line::from("Величина результата".bold()),
            Line::from(vec![
                "О".blue().bold(),
                "т ".into(),
                self.field_value_span(ActiveField::ResultMin),
                Span::raw("      "),
                "Д".blue().bold(),
                "о ".into(),
                self.field_value_span(ActiveField::ResultMax),
            ]),
            Line::from(""),
            self.field_line("К", "оличество примеров: ", ActiveField::ExerciseCount),
            self.field_line("С", "ложность: ", ActiveField::Complexity),
        ])
        .block(block)
        .render(area, buf);
    }

    fn field_line(
        &self,
        first_letter: &'static str,
        rest: &'static str,
        field: ActiveField,
    ) -> Line<'static> {
        Line::from(vec![
            first_letter.blue().bold(),
            rest.into(),
            self.field_value_span(field),
        ])
    }

    fn field_value_span(&self, field: ActiveField) -> Span<'static> {
        let value = if self.active_field == Some(field) {
            format!(
                "[ {}{} ]",
                self.input_buffer, INPUT_CURSOR[self.cursor_frame]
            )
        } else {
            format!("[ {} ]", self.field_value(field))
        };

        if self.active_field == Some(field) {
            value.blue().bold()
        } else {
            value.into()
        }
    }
}

fn field_hotkey(character: char) -> Option<ActiveField> {
    match character {
        'и' | 'И' | 'b' | 'B' | 'i' | 'I' => Some(ActiveField::PlayerName),
        'о' | 'О' | 'j' | 'J' | 'o' | 'O' => Some(ActiveField::ResultMin),
        'д' | 'Д' | 'l' | 'L' | 'd' | 'D' => Some(ActiveField::ResultMax),
        'к' | 'К' | 'r' | 'R' | 'k' | 'K' => Some(ActiveField::ExerciseCount),
        'с' | 'С' | 'c' | 'C' | 's' | 'S' => Some(ActiveField::Complexity),
        _ => None,
    }
}

fn is_ctrl_c(key_event: KeyEvent) -> bool {
    matches!(key_event.code, KeyCode::Char('c' | 'C'))
        && key_event.modifiers.contains(KeyModifiers::CONTROL)
}

fn horizontal_inset(area: Rect, inset: u16) -> Rect {
    let inset = inset.min(area.width / 2);
    Rect {
        x: area.x + inset,
        y: area.y,
        width: area.width.saturating_sub(inset * 2),
        height: area.height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn test_app(settings: Settings) -> App {
        App {
            status: Status::Welcome,
            settings,
            answer: None,
            correct_answers: 0,
            active_field: None,
            session: None,
            exercise_now: None,
            input_buffer: String::new(),
            cursor_frame: 0,
            exit: false,
        }
    }

    #[test]
    fn toggles_operation_without_leaving_empty_set() {
        let mut app = test_app(Settings {
            operations: HashSet::from([Operation::Addition]),
            ..Settings::default()
        });

        app.toggle_operation(Operation::Addition);
        assert!(app.settings.operations.contains(&Operation::Addition));

        app.toggle_operation(Operation::Subtraction);
        assert!(app.settings.operations.contains(&Operation::Subtraction));

        app.toggle_operation(Operation::Addition);
        assert!(!app.settings.operations.contains(&Operation::Addition));
    }

    #[test]
    fn render_does_not_panic() {
        let app = test_app(Settings::default());
        let mut buffer = Buffer::empty(Rect::new(0, 0, 200, 60));

        (&app).render(buffer.area, &mut buffer);
    }

    #[test]
    fn field_hotkeys_work_on_russian_and_latin_layouts() {
        assert_eq!(field_hotkey('И'), Some(ActiveField::PlayerName));
        assert_eq!(field_hotkey('b'), Some(ActiveField::PlayerName));
        assert_eq!(field_hotkey('i'), Some(ActiveField::PlayerName));
        assert_eq!(field_hotkey('О'), Some(ActiveField::ResultMin));
        assert_eq!(field_hotkey('j'), Some(ActiveField::ResultMin));
        assert_eq!(field_hotkey('o'), Some(ActiveField::ResultMin));
        assert_eq!(field_hotkey('Д'), Some(ActiveField::ResultMax));
        assert_eq!(field_hotkey('l'), Some(ActiveField::ResultMax));
        assert_eq!(field_hotkey('d'), Some(ActiveField::ResultMax));
        assert_eq!(field_hotkey('К'), Some(ActiveField::ExerciseCount));
        assert_eq!(field_hotkey('r'), Some(ActiveField::ExerciseCount));
        assert_eq!(field_hotkey('k'), Some(ActiveField::ExerciseCount));
        assert_eq!(field_hotkey('С'), Some(ActiveField::Complexity));
        assert_eq!(field_hotkey('c'), Some(ActiveField::Complexity));
        assert_eq!(field_hotkey('s'), Some(ActiveField::Complexity));
    }

    #[test]
    fn ctrl_c_exits_even_while_editing_text_input() {
        let mut app = test_app(Settings::default());
        app.start_input(ActiveField::PlayerName);
        let input_buffer = app.input_buffer.clone();

        app.handle_key_event(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));

        assert!(app.exit);
        assert_eq!(app.input_buffer, input_buffer);
    }

    #[test]
    fn commits_numeric_field_input() {
        let mut app = test_app(Settings::default());

        app.start_input(ActiveField::ExerciseCount);
        app.input_buffer.clear();
        app.handle_key_event(KeyCode::Char('4').into());
        app.handle_key_event(KeyCode::Char('2').into());
        app.handle_key_event(KeyCode::Enter.into());

        assert_eq!(app.settings.limits.exercise_count, 42);
        assert_eq!(app.active_field, None);
    }

    #[test]
    fn escape_cancels_input_without_changing_settings() {
        let mut app = test_app(Settings {
            player_name: "old-name".to_string(),
            ..Settings::default()
        });

        app.start_input(ActiveField::PlayerName);
        app.input_buffer = "new-name".to_string();
        app.handle_key_event(KeyCode::Esc.into());

        assert_eq!(app.settings.player_name, "old-name");
        assert_eq!(app.active_field, None);
        assert!(app.input_buffer.is_empty());
    }

    #[test]
    fn invalid_settings_input_restores_old_field_value() {
        let mut app = test_app(Settings::default());
        let old_min = app.settings.limits.result_min;

        app.start_input(ActiveField::ResultMin);
        app.input_buffer = app.settings.limits.result_max.to_string();
        app.handle_key_event(KeyCode::Enter.into());

        assert_eq!(app.settings.limits.result_min, old_min);
        assert_eq!(app.active_field, None);
    }

    #[test]
    fn header_does_not_use_player_name() {
        let app = test_app(Settings {
            player_name: "changed-player".to_string(),
            ..Settings::default()
        });
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 5));

        app.render_header(buffer.area, &mut buffer);
        let rendered = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains(HEADER_NAME));
        assert!(!rendered.contains("changed-player"));
    }
}
