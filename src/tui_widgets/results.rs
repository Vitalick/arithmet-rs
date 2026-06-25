use crate::domain::answer::{Answer, AnswerError};
use crate::domain::operation::Operation;
use crate::domain::session::Session;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table, Widget};
use strum::IntoEnumIterator;

const ANSWER_PAGE_SIZE: usize = 11;

#[derive(Debug)]
pub struct ResultsWidget<'a> {
    session: &'a Session,
    scroll: usize,
}

impl<'a> ResultsWidget<'a> {
    pub fn new(session: &'a Session, scroll: usize) -> Self {
        Self { session, scroll }
    }

    pub fn page_size() -> usize {
        ANSWER_PAGE_SIZE
    }

    fn summary_lines(&self) -> Vec<Line<'static>> {
        let settings = &self.session.settings;
        vec![
            Line::from(vec![
                "Имя: ".cyan().bold(),
                settings.player_name.clone().into(),
                "    Действия: ".cyan().bold(),
                settings
                    .enabled_operations()
                    .iter()
                    .map(|operation| operation.label().to_lowercase())
                    .collect::<Vec<_>>()
                    .join(", ")
                    .into(),
            ]),
            Line::from(vec![
                "Количество: ".cyan().bold(),
                settings.limits.exercise_count.to_string().into(),
                "    Пределы: ".cyan().bold(),
                format!(
                    "от {} до {}",
                    settings.limits.result_min, settings.limits.result_max
                )
                .into(),
                "    Сложность: ".cyan().bold(),
                format!("{} сек.", settings.limits.answer_time.as_secs()).into(),
            ]),
            Line::from(vec![
                "Верных ответов: ".cyan().bold(),
                self.session.correct_answers.to_string().into(),
                " из ".into(),
                self.session.total_answers().to_string().into(),
                "    Оценка: ".cyan().bold(),
                format!(
                    "{} ({})",
                    self.session.get_grade().value(),
                    self.session.get_grade()
                )
                .into(),
            ]),
            Line::from(self.time_summary()),
        ]
    }

    fn operation_rows(&self) -> Vec<Row<'static>> {
        Operation::iter()
            .filter(|operation| self.session.settings.operations.contains(operation))
            .map(|operation| {
                let total = self
                    .session
                    .get_answers()
                    .iter()
                    .filter(|answer| answer.exercise.operation == operation)
                    .count();
                let correct = self
                    .session
                    .get_answers()
                    .iter()
                    .filter(|answer| answer.exercise.operation == operation && answer.is_correct())
                    .count();
                Row::new(vec![
                    Cell::from(operation.label().to_lowercase()),
                    Cell::from(total.to_string()),
                    Cell::from(correct.to_string()),
                ])
            })
            .collect()
    }

    fn answer_rows(&self) -> Vec<Row<'static>> {
        self.session
            .get_answers()
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(ANSWER_PAGE_SIZE)
            .map(|(index, answer)| {
                let style = if answer.is_correct() {
                    Style::new().green()
                } else {
                    Style::new().red()
                };
                Row::new(vec![
                    Cell::from((index + 1).to_string()),
                    Cell::from(if answer.is_correct() {
                        "верно"
                    } else {
                        "неверно"
                    }),
                    Cell::from(format!(
                        "{}{}{}",
                        answer.exercise.left,
                        answer.exercise.operation.symbol(),
                        answer.exercise.right
                    )),
                    Cell::from(entered_text(answer)),
                    Cell::from(
                        answer
                            .exercise
                            .expected_str()
                            .unwrap_or_else(|_| "?".to_string()),
                    ),
                    Cell::from(format!("{} сек.", answer.time_elapsed.as_secs())),
                ])
                .style(style)
            })
            .collect()
    }

    fn time_summary(&self) -> String {
        let seconds = self
            .session
            .get_answers()
            .iter()
            .filter(|answer| answer.entered.is_ok())
            .map(|answer| answer.time_elapsed.as_secs())
            .collect::<Vec<_>>();
        if seconds.is_empty() {
            return "Время: нет введённых ответов".to_string();
        }
        let best = seconds.iter().min().copied().unwrap();
        let worst = seconds.iter().max().copied().unwrap();
        let average = seconds.iter().sum::<u64>() / seconds.len() as u64;
        format!("Время: лучшее - {best}, худшее - {worst}, среднее - {average}")
    }

    fn answer_scroll_hint(&self) -> Line<'static> {
        let total = self.session.total_answers();
        if total <= ANSWER_PAGE_SIZE {
            return Line::from("Предложенные действия").centered();
        }
        let from = self.scroll + 1;
        let to = (self.scroll + ANSWER_PAGE_SIZE).min(total);
        Line::from(format!("Предложенные действия: {from}-{to} из {total}")).centered()
    }
}

impl Widget for ResultsWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [summary_area, operations_area, answers_area] = Layout::vertical([
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Min(8),
        ])
        .spacing(1)
        .areas(area);

        Paragraph::new(self.summary_lines())
            .block(
                Block::bordered()
                    .border_set(border::PLAIN)
                    .title(Line::from("Р Е З У Л Ь Т А Т Ы".green().bold()).centered()),
            )
            .render(summary_area, buf);

        Table::new(
            self.operation_rows(),
            [
                Constraint::Percentage(50),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ],
        )
        .header(Row::new(vec!["Действие", "Всего", "Верно"]).style(Style::new().yellow().bold()))
        .block(
            Block::bordered()
                .border_set(border::PLAIN)
                .title(Line::from("Количество действий").centered()),
        )
        .render(operations_area, buf);

        Table::new(
            self.answer_rows(),
            [
                Constraint::Length(4),
                Constraint::Length(8),
                Constraint::Percentage(22),
                Constraint::Percentage(24),
                Constraint::Percentage(18),
                Constraint::Percentage(14),
            ],
        )
        .header(
            Row::new(vec!["№", "Статус", "Пример", "Введено", "Верно", "Время"])
                .style(Style::new().yellow().bold()),
        )
        .block(
            Block::bordered()
                .border_set(border::PLAIN)
                .title(self.answer_scroll_hint()),
        )
        .render(answers_area, buf);
    }
}

fn entered_text(answer: &Answer) -> String {
    match answer.entered {
        Ok(entered) => entered.to_string(),
        Err(AnswerError::Escaped) => "нажата клавиша <Esc>".to_string(),
        Err(AnswerError::TimedOut) => "вышло время".to_string(),
        Err(AnswerError::SessionAborted) => "нажата клавиша <F10>".to_string(),
        Err(AnswerError::InvalidInput) => "некорректный ввод".to_string(),
    }
}
