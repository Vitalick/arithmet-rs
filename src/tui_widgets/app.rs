use color_eyre::{Result, eyre::WrapErr};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Widget},
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use validations::Validate;

use crate::domain::answer::AnswerError;
use crate::domain::session::{Session, StepResult};
use crate::domain::{operation::Operation, settings::Settings};
use crate::tui_widgets::main::MainWidget;
use crate::tui_widgets::results::ResultsWidget;
use crate::tui_widgets::status::{Status, StatusWidget};

const CONFIG_PATH: &str = "arithmet.toml";
const MAIN_AREA_HEIGHT: u16 = 16;
const STATUS_AREA_HEIGHT: u16 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveField {
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
    last_finished_session: Option<Session>,
    active_field: Option<ActiveField>,
    input_buffer: String,
    cursor_frame: usize,
    results_saved: bool,
    results_scroll: usize,
    results_return_status: Status,
    exit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            status: Status::Welcome,
            settings: Settings::load(CONFIG_PATH).unwrap_or_default(),
            session: None,
            last_finished_session: None,
            active_field: None,
            input_buffer: String::new(),
            cursor_frame: 0,
            results_saved: false,
            results_scroll: 0,
            results_return_status: Status::Welcome,
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
            self.update_status()?;
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
        self.results_saved = false;
        self.game_step()?;
        Ok(())
    }

    fn update_status(&mut self) -> Result<()> {
        if self.status == Status::ResultsView {
            return Ok(());
        }
        if self.session.is_none() {
            self.status = Status::Welcome;
            return Ok(());
        }
        if let Some(session) = self.session.as_mut()
            && session.exercise_now.is_some()
        {
            match session
                .game_step()
                .map_err(|err| color_eyre::eyre::eyre!(err))?
            {
                StepResult::TimedOut => {
                    if matches!(self.active_field, Some(ActiveField::GameAnswer)) {
                        self.cancel_input();
                    }
                }
                _ => {}
            }
        }
        let session = self.session.as_ref().unwrap();
        if session.exercise_now.is_none() && session.last_answer.is_some() && !self.results_saved {
            self.status = Status::AwaitingGameContinue;
            return Ok(());
        }
        if session.is_finished() {
            self.finish_session()?;
            return Ok(());
        }
        if self.session.as_ref().unwrap().exercise_now.is_none() {
            self.status = Status::AwaitingGameContinue;
            return Ok(());
        }
        self.status = Status::AwaitingAnswer;
        Ok(())
    }

    fn game_step(&mut self) -> Result<(), String> {
        match self.session.as_mut() {
            None => Err("session is not set".to_string()),
            Some(session) => {
                match session.game_step()? {
                    StepResult::TimedOut => {
                        if matches!(self.active_field, Some(ActiveField::GameAnswer)) {
                            self.cancel_input();
                        }
                    }
                    _ => {}
                }
                Ok(())
            }
        }
    }

    fn finish_session(&mut self) -> Result<()> {
        let Some(session) = self.session.as_ref() else {
            return Ok(());
        };
        if !self.results_saved {
            session.save().wrap_err("save session results failed")?;
            self.last_finished_session = Some(session.clone());
            self.results_saved = true;
        }
        self.status = Status::GameFinished;
        self.active_field = None;
        self.input_buffer.clear();
        Ok(())
    }

    fn answer_str(&mut self, entered: String) {
        if let Some(session) = self.session.as_mut() {
            match entered.parse() {
                Ok(value) => session.answer(Ok(value)),
                _ => session.answer(Err(AnswerError::InvalidInput)),
            }
        }
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn return_to_welcome(&mut self) {
        self.session = None;
        self.status = Status::Welcome;
        self.active_field = None;
        self.input_buffer.clear();
        self.results_saved = false;
    }

    fn open_results(&mut self) {
        if self.is_game_active() || self.result_session().is_none() {
            return;
        }
        self.results_return_status = self.status;
        self.status = Status::ResultsView;
        self.active_field = None;
        self.input_buffer.clear();
        self.clamp_results_scroll();
    }

    fn close_results(&mut self) {
        self.status = self.results_return_status;
        self.clamp_results_scroll();
    }

    fn result_session(&self) -> Option<&Session> {
        self.session
            .as_ref()
            .filter(|_| self.status == Status::GameFinished)
            .or(self.last_finished_session.as_ref())
    }

    fn result_answer_count(&self) -> usize {
        self.result_session()
            .map(Session::total_answers)
            .unwrap_or_default()
    }

    fn clamp_results_scroll(&mut self) {
        let max_scroll = self
            .result_answer_count()
            .saturating_sub(ResultsWidget::page_size());
        self.results_scroll = self.results_scroll.min(max_scroll);
    }

    fn is_game_active(&self) -> bool {
        matches!(
            self.status,
            Status::AwaitingAnswer | Status::AwaitingGameContinue
        ) || matches!(self.active_field, Some(ActiveField::GameAnswer))
    }

    fn handle_escape_key(&mut self) {
        if self.status == Status::ResultsView {
            self.close_results();
            return;
        }

        if self.active_field == Some(ActiveField::GameAnswer)
            || self.status == Status::AwaitingGameContinue
        {
            self.return_to_welcome();
            return;
        }

        if self.active_field.is_some() {
            self.cancel_input();
            return;
        }

        self.exit();
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
                self.answer_str(value.to_string());
                self.cancel_input();
                return;
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
            ActiveField::GameAnswer => String::default(),
        }
    }

    fn handle_input_key_event(&mut self, key_event: KeyEvent) -> bool {
        if self.active_field.is_none() {
            match self.status {
                Status::ResultsView => {
                    self.handle_results_key_event(key_event);
                    return true;
                }
                Status::Welcome | Status::GameFinished => {
                    if key_event.code == KeyCode::Enter {
                        self.start_game().unwrap();
                        self.start_input(ActiveField::GameAnswer);
                        return true;
                    }
                }
                Status::AwaitingGameContinue => match key_event.code {
                    KeyCode::Enter | KeyCode::Tab => {
                        let have_next = self
                            .session
                            .as_ref()
                            .map(Session::have_next)
                            .unwrap_or_default();
                        if have_next {
                            self.game_step().unwrap();
                            self.start_input(ActiveField::GameAnswer);
                        } else {
                            self.finish_session().unwrap();
                        }
                        return true;
                    }
                    KeyCode::Esc => {
                        self.handle_escape_key();
                        return true;
                    }
                    _ => (),
                },
                _ => (),
            }
            return false;
        }

        match key_event.code {
            KeyCode::Enter => self.commit_input(),
            KeyCode::Esc => self.handle_escape_key(),
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
            self.handle_escape_key();
            return;
        }

        if self.handle_input_key_event(key_event) {
            return;
        }

        if self.is_game_active() {
            return;
        }

        match key_event.code {
            KeyCode::Esc => self.handle_escape_key(),
            KeyCode::Char('q') | KeyCode::Char('Q') => self.exit(),
            KeyCode::F(1) => self.open_results(),
            KeyCode::Char(character) if is_results_hotkey(character) => self.open_results(),
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

    fn handle_results_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.handle_escape_key(),
            KeyCode::Up => {
                self.results_scroll = self.results_scroll.saturating_sub(1);
            }
            KeyCode::Down => {
                self.results_scroll = self.results_scroll.saturating_add(1);
                self.clamp_results_scroll();
            }
            KeyCode::PageUp => {
                self.results_scroll = self
                    .results_scroll
                    .saturating_sub(ResultsWidget::page_size());
            }
            KeyCode::PageDown => {
                self.results_scroll = self
                    .results_scroll
                    .saturating_add(ResultsWidget::page_size());
                self.clamp_results_scroll();
            }
            KeyCode::Home => {
                self.results_scroll = 0;
            }
            KeyCode::End => {
                self.results_scroll = self
                    .result_answer_count()
                    .saturating_sub(ResultsWidget::page_size());
            }
            _ => {}
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            }
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

        if self.status == Status::ResultsView {
            if let Some(session) = self.result_session() {
                ResultsWidget::new(session, self.results_scroll).render(inner, buf);
            }
            return;
        }

        let [main_area, status_area] = Layout::vertical([
            Constraint::Length(MAIN_AREA_HEIGHT),
            Constraint::Min(STATUS_AREA_HEIGHT),
        ])
        .areas(inner);

        let def_session = Session::default();
        let session = self.session.as_ref().unwrap_or_else(|| &def_session);

        MainWidget::new(
            &self.settings,
            &self.session,
            self.active_field,
            &self.input_buffer,
        )
        .render(main_area, buf);
        StatusWidget::new(
            &self.session,
            &session.exercise_now,
            self.status,
            self.active_field,
        )
        .render(status_area, buf);
    }
}

impl App {
    fn instructions(&self) -> Line<'static> {
        if self.is_game_active() {
            return match self.status {
                Status::AwaitingGameContinue => Line::from(vec![
                    " ".into(),
                    "<Enter/Tab>".yellow().bold(),
                    " - следующий пример, ".into(),
                    "<Esc/Ctrl+C>".yellow().bold(),
                    " - на главную".into(),
                    " ".into(),
                ]),
                _ => Line::from(vec![
                    " ".into(),
                    "<Enter>".yellow().bold(),
                    " - ответить, ".into(),
                    "<Backspace/Delete>".yellow().bold(),
                    " - правка, ".into(),
                    "<Esc/Ctrl+C>".yellow().bold(),
                    " - на главную".into(),
                    " ".into(),
                ]),
            };
        }

        match self.status {
            Status::GameFinished => Line::from(vec![
                " ".into(),
                "<F1/Р>".yellow().bold(),
                " - результаты, ".into(),
                "<Enter>".yellow().bold(),
                " - новая игра, ".into(),
                "<Esc/Ctrl+C/Q>".yellow().bold(),
                " - выход".into(),
                " ".into(),
            ]),
            Status::ResultsView => Line::from(vec![
                " ".into(),
                "<↑↓ PgUp PgDn Home End>".yellow().bold(),
                " - прокрутка, ".into(),
                "<Esc/Ctrl+C>".yellow().bold(),
                " - назад".into(),
                " ".into(),
            ]),
            _ => Line::from(vec![
                " ".into(),
                "<+ - * / :>".yellow().bold(),
                " - действие, ".into(),
                "<И О Д К С>".yellow().bold(),
                " - поля, ".into(),
                if self.result_session().is_some() {
                    "<F1/Р>".yellow().bold()
                } else {
                    "".into()
                },
                if self.result_session().is_some() {
                    " - результаты, ".into()
                } else {
                    "".into()
                },
                "<Enter>".yellow().bold(),
                " - старт, ".into(),
                "<Esc/Ctrl+C/Q>".yellow().bold(),
                " - выход".into(),
                " ".into(),
            ]),
        }
    }
}

fn is_results_hotkey(character: char) -> bool {
    matches!(character, 'р' | 'Р' | 'h' | 'H')
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::expression::{Exercise, ExerciseWithStartTime};
    use crate::tui_widgets::main::HEADER_NAME;
    use std::collections::HashSet;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_app(settings: Settings) -> App {
        App {
            status: Status::Welcome,
            settings,
            active_field: None,
            session: None,
            last_finished_session: None,
            input_buffer: String::new(),
            cursor_frame: 0,
            results_saved: false,
            results_scroll: 0,
            results_return_status: Status::Welcome,
            exit: false,
        }
    }

    fn unique_results_dir() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!(
            "arithmet-app-flow-{}-{}",
            std::process::id(),
            suffix
        ))
    }

    fn rendered_text(buffer: &Buffer) -> String {
        buffer
            .content()
            .chunks(buffer.area.width as usize)
            .map(|line| line.iter().map(|cell| cell.symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
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
    fn escape_exits_from_welcome_screen() {
        let mut app = test_app(Settings::default());

        app.handle_key_event(KeyCode::Esc.into());

        assert!(app.exit);
    }

    #[test]
    fn ctrl_c_matches_escape_while_editing_text_input() {
        let mut app = test_app(Settings::default());
        app.start_input(ActiveField::PlayerName);
        app.input_buffer = "changed".to_string();

        app.handle_key_event(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));

        assert!(!app.exit);
        assert_eq!(app.active_field, None);
        assert!(app.input_buffer.is_empty());
    }

    #[test]
    fn escape_and_ctrl_c_return_to_welcome_during_game() {
        let mut escape_app = test_app(Settings::default());
        escape_app.start_game().unwrap();
        escape_app.start_input(ActiveField::GameAnswer);

        escape_app.handle_key_event(KeyCode::Esc.into());

        assert!(!escape_app.exit);
        assert_eq!(escape_app.status, Status::Welcome);
        assert!(escape_app.session.is_none());
        assert_eq!(escape_app.active_field, None);

        let mut ctrl_c_app = test_app(Settings::default());
        ctrl_c_app.start_game().unwrap();
        ctrl_c_app.start_input(ActiveField::GameAnswer);

        ctrl_c_app.handle_key_event(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));

        assert!(!ctrl_c_app.exit);
        assert_eq!(ctrl_c_app.status, Status::Welcome);
        assert!(ctrl_c_app.session.is_none());
        assert_eq!(ctrl_c_app.active_field, None);
    }

    #[test]
    fn f1_does_not_change_active_game() {
        let mut app = test_app(Settings::default());
        app.start_game().unwrap();
        app.start_input(ActiveField::GameAnswer);
        app.input_buffer = "12".to_string();

        app.handle_key_event(KeyCode::F(1).into());

        assert!(!app.exit);
        assert!(app.session.is_some());
        assert_eq!(app.active_field, Some(ActiveField::GameAnswer));
        assert_eq!(app.input_buffer, "12");
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
    fn game_answer_shows_banner_then_starts_next_exercise() {
        let mut app = test_app(Settings {
            operations: HashSet::from([Operation::Addition]),
            limits: crate::domain::settings::Limits {
                exercise_count: 2,
                ..Default::default()
            },
            ..Settings::default()
        });

        app.start_game().unwrap();
        app.start_input(ActiveField::GameAnswer);
        let expected = app
            .session
            .as_ref()
            .unwrap()
            .exercise_now
            .unwrap()
            .exercise
            .expected()
            .unwrap();

        app.input_buffer = expected.to_string();
        app.handle_key_event(KeyCode::Enter.into());
        app.update_status().unwrap();

        let session = app.session.as_ref().unwrap();
        assert_eq!(session.total_answers(), 1);
        assert!(session.exercise_now.is_none());
        assert_eq!(session.last_answer_banner(), "Молодец!");
        assert_eq!(app.status, Status::AwaitingGameContinue);

        app.handle_key_event(KeyCode::Enter.into());

        let session = app.session.as_ref().unwrap();
        assert!(session.exercise_now.is_some());
        assert_eq!(app.active_field, Some(ActiveField::GameAnswer));
    }

    #[test]
    fn answered_exercise_stays_visible_with_check_and_counter() {
        let settings = Settings {
            operations: HashSet::from([Operation::Addition]),
            limits: crate::domain::settings::Limits {
                exercise_count: 2,
                ..Default::default()
            },
            ..Settings::default()
        };
        let mut app = test_app(settings.clone());
        let mut session = Session::new(settings).unwrap();
        session.exercise_now = Some(ExerciseWithStartTime::new(Exercise::new(
            2,
            Operation::Addition,
            3,
        )));
        app.session = Some(session);
        app.start_input(ActiveField::GameAnswer);
        app.input_buffer = "5".to_string();

        app.handle_key_event(KeyCode::Enter.into());
        app.update_status().unwrap();

        let mut buffer = Buffer::empty(Rect::new(0, 0, 200, 60));
        (&app).render(buffer.area, &mut buffer);
        let rendered = rendered_text(&buffer);

        assert!(rendered.contains("Пример 1/2"));
        assert!(rendered.contains("2 + 3 = 5"));
        assert!(rendered.contains("a) 5 - 2 = 3"));
        assert!(rendered.contains("b) 5 - 3 = 2"));
        assert!(rendered.contains("Верный ответ: 5"));
    }

    #[test]
    fn final_answer_shows_banner_before_grade_then_saves_results_once() {
        let results_dir = unique_results_dir();
        let mut app = test_app(Settings {
            player_name: "ui_player".to_string(),
            results_dir: results_dir.to_string_lossy().into_owned(),
            operations: HashSet::from([Operation::Addition]),
            limits: crate::domain::settings::Limits {
                exercise_count: 1,
                ..Default::default()
            },
        });

        app.start_game().unwrap();
        app.start_input(ActiveField::GameAnswer);
        let expected = app
            .session
            .as_ref()
            .unwrap()
            .exercise_now
            .unwrap()
            .exercise
            .expected()
            .unwrap();

        app.input_buffer = expected.to_string();
        app.handle_key_event(KeyCode::Enter.into());
        app.update_status().unwrap();

        assert_eq!(app.status, Status::AwaitingGameContinue);
        assert!(!app.results_saved);
        assert!(app.last_finished_session.is_none());

        app.handle_key_event(KeyCode::Enter.into());
        assert_eq!(app.status, Status::GameFinished);
        assert!(app.results_saved);
        assert!(app.last_finished_session.is_some());

        let player_dir = results_dir.join("ui_player");
        let result_files = fs::read_dir(&player_dir)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(result_files.len(), 2);

        let _ = fs::remove_dir_all(results_dir);
    }

    #[test]
    fn f1_and_russian_r_open_results_after_finished_game() {
        let results_dir = unique_results_dir();
        let mut app = test_app(Settings {
            player_name: "results_player".to_string(),
            results_dir: results_dir.to_string_lossy().into_owned(),
            operations: HashSet::from([Operation::Addition]),
            limits: crate::domain::settings::Limits {
                exercise_count: 1,
                ..Default::default()
            },
            ..Settings::default()
        });
        app.start_game().unwrap();
        app.start_input(ActiveField::GameAnswer);
        let expected = app
            .session
            .as_ref()
            .unwrap()
            .exercise_now
            .unwrap()
            .exercise
            .expected()
            .unwrap();

        app.input_buffer = expected.to_string();
        app.handle_key_event(KeyCode::Enter.into());
        app.update_status().unwrap();
        app.handle_key_event(KeyCode::Enter.into());

        app.handle_key_event(KeyCode::F(1).into());
        assert_eq!(app.status, Status::ResultsView);

        app.handle_key_event(KeyCode::Esc.into());
        assert_eq!(app.status, Status::GameFinished);

        app.handle_key_event(KeyCode::Char('Р').into());
        assert_eq!(app.status, Status::ResultsView);

        let _ = fs::remove_dir_all(results_dir);
    }

    #[test]
    fn results_view_renders_summary_and_answer_table() {
        let results_dir = unique_results_dir();
        let mut app = test_app(Settings {
            player_name: "results_player".to_string(),
            results_dir: results_dir.to_string_lossy().into_owned(),
            operations: HashSet::from([Operation::Addition]),
            limits: crate::domain::settings::Limits {
                exercise_count: 1,
                ..Default::default()
            },
        });
        app.start_game().unwrap();
        app.start_input(ActiveField::GameAnswer);
        let expected = app
            .session
            .as_ref()
            .unwrap()
            .exercise_now
            .unwrap()
            .exercise
            .expected()
            .unwrap();

        app.input_buffer = expected.to_string();
        app.handle_key_event(KeyCode::Enter.into());
        app.update_status().unwrap();
        app.handle_key_event(KeyCode::Enter.into());
        app.handle_key_event(KeyCode::F(1).into());

        let mut buffer = Buffer::empty(Rect::new(0, 0, 200, 60));
        (&app).render(buffer.area, &mut buffer);
        let rendered = rendered_text(&buffer);

        assert!(rendered.contains("Р Е З У Л Ь Т А Т Ы"));
        assert!(rendered.contains("results_player"));
        assert!(rendered.contains("Количество действий"));
        assert!(rendered.contains("Предложенные действия"));
        assert!(rendered.contains("верно"));

        let _ = fs::remove_dir_all(results_dir);
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
        let mut buffer = Buffer::empty(Rect::new(0, 0, 200, 60));

        (&app).render(buffer.area, &mut buffer);
        let rendered_lines = rendered_text(&buffer)
            .lines()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let rendered = rendered_lines.join("\n");
        let header_line = rendered_lines
            .iter()
            .find(|line| line.contains(HEADER_NAME))
            .unwrap();

        assert!(rendered.contains(HEADER_NAME));
        assert!(!header_line.contains("changed-player"));
    }
}
