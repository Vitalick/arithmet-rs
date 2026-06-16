use super::Status;
use crate::domain::expression::ExerciseWithStartTime;
use crate::domain::session::Session;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Modifier, Style};
use ratatui::widgets::{Gauge, Widget};
use std::cmp::{max, min};
use std::time::Duration;

#[derive(Debug)]
pub struct ProgressWidget<'a> {
    session: &'a Option<Session>,
    exercise_now: &'a Option<ExerciseWithStartTime>,
    status: &'a Status,
}

impl<'a> ProgressWidget<'a> {
    pub fn new(
        session: &'a Option<Session>,
        exercise_now: &'a Option<ExerciseWithStartTime>,
        status: &'a Status,
    ) -> Self {
        ProgressWidget {
            session,
            exercise_now,
            status,
        }
    }
}
impl Widget for ProgressWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.status {
            Status::Welcome | Status::GameFinished => return,
            _ => {}
        }
        let session = self.session.as_ref().unwrap();
        let exercise_now = self.exercise_now.as_ref().unwrap();
        let time_elapsed = min(
            exercise_now.start_time.elapsed(),
            session.settings.limits.answer_time,
        );
        let time_left = max(
            session.settings.limits.answer_time - time_elapsed,
            Duration::ZERO,
        );
        let ratio = time_elapsed.as_millis() as f64
            / session.settings.limits.answer_time.as_millis() as f64;
        Gauge::default()
            .style(Modifier::BOLD)
            .gauge_style(Style::new().blue().on_black())
            .label(format!("Осталось {:?} сек.", time_left.as_secs()))
            .ratio(ratio)
            .render(area, buf);
    }
}
