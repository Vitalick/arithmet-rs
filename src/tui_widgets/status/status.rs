use super::banner::BannerWidget;
use super::progress::ProgressWidget;
use crate::domain::expression::ExerciseWithStartTime;
use crate::domain::session::Session;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};
use strum_macros::Display;
use crate::tui_widgets::app::ActiveField;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Status {
    Welcome,
    AwaitingAnswer,
    AwaitingGameContinue,
    GameFinished,
}

#[derive(Debug)]
pub struct StatusWidget<'a> {
    banner: BannerWidget<'a>,
    progress: ProgressWidget<'a>,
    status: Status,
    active_field: Option<ActiveField>,
}

impl<'a> StatusWidget<'a> {
    pub fn new(
        session: &'a Option<Session>,
        exercise_now: &'a Option<ExerciseWithStartTime>,
        status: Status,
        active_field: Option<ActiveField>
    ) -> Self {
        StatusWidget {
            banner: BannerWidget::new(session, status),
            progress: ProgressWidget::new(session, exercise_now, status),
            status,
            active_field
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
        Paragraph::new(vec![
            Line::from(self.status.to_string()),
            Line::from(format!("{:?}", self.active_field)),
        ])
        .render(status, buf);
        self.banner.render(banner, buf);
        self.progress.render(progress, buf);
    }
}
