use super::operation_item::OperationItemWidget;
use super::{CursorType, cursor};
use crate::domain::answer::{Answer, AnswerError};
use crate::domain::expression::Exercise;
use crate::domain::operation::Operation;
use crate::domain::session::Session;
use crate::domain::settings::Settings;
use crate::tui_widgets::app::ActiveField;
use crate::tui_widgets::main::field_line::FieldLineWidget;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Line, Span, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph, Widget};
use strum::IntoEnumIterator;

pub const HEADER_NAME: &str = "VIT";

#[derive(Debug)]
pub struct MainWidget<'a> {
    settings: &'a Settings,
    session: &'a Option<Session>,
    active_field: Option<ActiveField>,
    input_buffer: &'a str,
}

impl<'a> MainWidget<'a> {
    pub fn new(
        settings: &'a Settings,
        session: &'a Option<Session>,
        active_field: Option<ActiveField>,
        input_buffer: &'a str,
    ) -> Self {
        Self {
            settings,
            session,
            active_field,
            input_buffer,
        }
    }

    fn render_actions_column(&self, area: Rect, buf: &mut Buffer) {
        let [_, actions] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        self.render_actions(actions, buf);
    }

    fn render_actions(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .border_set(border::PLAIN)
            .title(Line::from(" Действие ".bold()).centered());

        let operations = Operation::iter()
            .map(|operation| {
                OperationItemWidget::new(
                    self.settings.operations.contains(&operation),
                    operation_label(operation),
                )
                .into_line()
            })
            .collect::<Vec<_>>();

        Paragraph::new(operations).block(block).render(area, buf);
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

    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let header = Paragraph::new(vec![
            Line::from(format!("==== версия {} ====", env!("CARGO_PKG_VERSION"))),
            Line::from(format!("{:<12}{:>12}", HEADER_NAME, 2026)),
        ])
        .centered();

        header.render(area, buf);
    }

    fn render_exercise(&self, area: Rect, buf: &mut Buffer) {
        let exercise_block = Block::bordered()
            .border_set(border::PLAIN)
            .title(Line::from(self.exercise_title()).bold().centered())
            .title_bottom(
                Line::from(format!(
                    " Верных ответов: {} ",
                    self.session
                        .as_ref()
                        .map(|session| session.correct_answers)
                        .unwrap_or_default()
                ))
                .bold()
                .centered(),
            );
        let exercise = Paragraph::new(self.exercise_text());
        exercise.block(exercise_block).render(area, buf);
    }

    fn render_check(&self, area: Rect, buf: &mut Buffer) {
        let check_block = Block::bordered()
            .border_set(border::PLAIN)
            .title(Line::from(" Проверка ".bold()).centered())
            .title_bottom(Line::from(self.correct_answer_text()).bold().centered());
        let check_text = self.check_text();
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
            .title(Line::from(" Настройки ".bold()).centered());

        Paragraph::new(vec![
            self.field_line(
                highlighted_prefix("И", "мя"),
                ActiveField::PlayerName,
                true,
                true,
            ),
            Line::from(""),
            Line::from("Величина результата".bold()),
            self.result_limits_line(),
            Line::from(""),
            self.field_line(
                highlighted_prefix("К", "оличество примеров"),
                ActiveField::ExerciseCount,
                true,
                true,
            ),
            self.field_line(
                highlighted_prefix("С", "ложность"),
                ActiveField::Complexity,
                true,
                true,
            ),
        ])
        .block(block)
        .render(area, buf);
    }

    fn result_limits_line(&self) -> Line<'static> {
        let mut spans = self
            .field_line(
                highlighted_prefix("О", "т"),
                ActiveField::ResultMin,
                true,
                false,
            )
            .spans;
        spans.push(Span::raw("      "));
        spans.extend(
            self.field_line(
                highlighted_prefix("Д", "о"),
                ActiveField::ResultMax,
                true,
                false,
            )
            .spans,
        );
        Line::from(spans)
    }

    fn field_line(
        &self,
        label: Line<'static>,
        field: ActiveField,
        brackets: bool,
        colon: bool,
    ) -> Line<'static> {
        let active = self.active_field == Some(field);
        let value = if active {
            self.input_buffer.to_string()
        } else {
            self.field_value(field)
        };

        FieldLineWidget::new(label, value, active, brackets, colon, CursorType::Spinner).into_line()
    }

    fn field_value(&self, field: ActiveField) -> String {
        match field {
            ActiveField::PlayerName => self.settings.player_name.clone(),
            ActiveField::ResultMin => self.settings.limits.result_min.to_string(),
            ActiveField::ResultMax => self.settings.limits.result_max.to_string(),
            ActiveField::ExerciseCount => self.settings.limits.exercise_count.to_string(),
            ActiveField::Complexity => self.settings.limits.answer_time.as_secs().to_string(),
            ActiveField::GameAnswer => String::new(),
        }
    }

    fn exercise_title(&self) -> String {
        let Some(session) = self.session.as_ref() else {
            return " Пример ".to_string();
        };
        let total = session.settings.limits.exercise_count;
        let current = if session.exercise_now.is_some() {
            session.total_answers() + 1
        } else {
            session.total_answers()
        };
        format!(" Пример {}/{} ", current.clamp(1, total), total)
    }

    fn exercise_text(&self) -> String {
        let Some(session) = self.session.as_ref() else {
            return String::new();
        };
        if let Some(exercise_now) = session.exercise_now {
            return format!(
                "{} = {}{}",
                exercise_now.exercise,
                self.input_buffer,
                CursorType::Spinner
            );
        }
        let Some(answer) = session.last_answer else {
            return String::new();
        };
        format!("{} = {}", answer.exercise, answer_text(&answer))
    }

    fn check_text(&self) -> Vec<Line<'static>> {
        let Some(answer) = self
            .session
            .as_ref()
            .and_then(|session| session.last_answer)
        else {
            return vec![Line::from("a)"), Line::from("b)"), Line::from("")];
        };
        let Ok([first, second]) = answer.check_expressions() else {
            return vec![Line::from("a)"), Line::from("b)"), Line::from("")];
        };
        vec![
            Line::from(format!("a) {}", expression_text(&*first))),
            Line::from(format!("b) {}", expression_text(&*second))),
            Line::from(""),
        ]
    }

    fn correct_answer_text(&self) -> String {
        let Some(exercise) = self.visible_answer().map(|answer| answer.exercise) else {
            return " Верный ответ: ".to_string();
        };
        format!(" Верный ответ: {} ", expected_text(exercise))
    }

    fn visible_answer(&self) -> Option<Answer> {
        self.session
            .as_ref()
            .and_then(|session| session.last_answer)
    }
}

fn answer_text(answer: &Answer) -> String {
    match answer.entered {
        Ok(entered) => entered.to_string(),
        Err(AnswerError::InvalidInput) => "некорректно".to_string(),
        Err(AnswerError::TimedOut) => "время вышло".to_string(),
        Err(AnswerError::Escaped | AnswerError::SessionAborted) => "прервано".to_string(),
    }
}

fn expected_text(exercise: Exercise) -> String {
    exercise.expected_str().unwrap_or_else(|_| "?".to_string())
}

fn expression_text(expression: &dyn crate::domain::expression::Expression) -> String {
    expression.evaluate().unwrap_or_else(|_| "?".to_string())
}

impl Widget for MainWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
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
}

pub fn highlighted_label(
    prefix: &'static str,
    highlighted: &'static str,
    suffix: &'static str,
) -> Line<'static> {
    Line::from(vec![
        Span::raw(prefix),
        highlighted.yellow().bold(),
        Span::raw(suffix),
    ])
}

pub fn highlighted_prefix(highlighted: &'static str, suffix: &'static str) -> Line<'static> {
    highlighted_label("", highlighted, suffix)
}

#[allow(dead_code)]
pub fn highlighted_suffix(prefix: &'static str, highlighted: &'static str) -> Line<'static> {
    highlighted_label(prefix, highlighted, "")
}

fn operation_label(operation: Operation) -> Line<'static> {
    Line::from(vec![
        operation.symbol().yellow().bold(),
        " ".into(),
        operation.label().to_lowercase().into(),
    ])
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
