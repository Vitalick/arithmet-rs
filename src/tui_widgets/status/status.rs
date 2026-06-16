use super::banner::BannerWidget;
use super::progress::ProgressWidget;
use crate::domain::expression::ExerciseWithStartTime;
use crate::domain::session::Session;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::Widget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Welcome,
    AwaitingAnswer,
    AwaitingGameContinue,
    GameFinished,
}
pub struct StatusWidget {
    session: Option<Session>,
    exercise_now: Option<ExerciseWithStartTime>,
    status: Status,

    banner: BannerWidget,
    progress: ProgressWidget,
}

impl StatusWidget {
    pub fn new(
        session: Option<Session>,
        exercise_now: Option<ExerciseWithStartTime>,
        status: Status,
    ) -> Self {
        StatusWidget {
            session,
            exercise_now,
            status,
            banner: BannerWidget::new(session, exercise_now, status),
            progress: ProgressWidget::new(session, exercise_now, status),
        }
    }
}

impl Widget for StatusWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [_, banner, _, progress] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(7),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .spacing(1)
        .areas(area);

        self.banner.render(banner, buf);
        self.progress.render(progress, buf);
    }
}
