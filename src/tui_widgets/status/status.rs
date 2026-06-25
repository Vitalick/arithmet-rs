use super::banner::BannerWidget;
use super::progress::ProgressWidget;
use crate::domain::expression::ExerciseWithStartTime;
use crate::domain::session::Session;
use crate::tui_widgets::app::ActiveField;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};
use strum_macros::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Status {
    Welcome,
    AwaitingAnswer,
    AwaitingGameContinue,
    GameFinished,
}

#[derive(Debug)]
pub struct StatusWidget<'a> {
    session: &'a Option<Session>,
    banner: BannerWidget<'a>,
    progress: ProgressWidget<'a>,
    status: Status,
}

impl<'a> StatusWidget<'a> {
    pub fn new(
        session: &'a Option<Session>,
        exercise_now: &'a Option<ExerciseWithStartTime>,
        status: Status,
        _active_field: Option<ActiveField>,
    ) -> Self {
        StatusWidget {
            session,
            banner: BannerWidget::new(session, status),
            progress: ProgressWidget::new(session, exercise_now, status),
            status,
        }
    }
}

impl Widget for StatusWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [status, banner, _, progress] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(7),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .spacing(1)
        .areas(area);
        Paragraph::new(self.status_lines()).render(status, buf);
        self.banner.render(banner, buf);
        self.progress.render(progress, buf);
    }
}

impl StatusWidget<'_> {
    fn status_lines(&self) -> Vec<Line<'static>> {
        match self.status {
            Status::Welcome => vec![Line::from("Нажмите Enter, чтобы начать игру").centered()],
            Status::AwaitingAnswer => vec![Line::from("Введите ответ и нажмите Enter").centered()],
            Status::AwaitingGameContinue => {
                vec![Line::from("Enter или Tab - следующий пример").centered()]
            }
            Status::GameFinished => {
                let Some(session) = self.session.as_ref() else {
                    return vec![Line::from("Игра завершена").centered()];
                };
                vec![
                    Line::from("Игра завершена".bold()).centered(),
                    Line::from(session.result_summary()).centered(),
                    Line::from("Результат сохранён").centered(),
                ]
            }
        }
    }
}
