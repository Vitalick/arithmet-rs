use super::Status;
use crate::domain::expression::ExerciseWithStartTime;
use crate::domain::session::Session;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Modifier, Style};
use ratatui::widgets::{Gauge, Widget};
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
        let time_elapsed = exercise_now.start_time.elapsed();
        let time_left = session.settings.limits.answer_time - time_elapsed;
        if time_left <= Duration::ZERO {
            Gauge::default()
                .style(Modifier::BOLD)
                .gauge_style(Style::new().red().on_black())
                .percent(100)
                .render(area, buf);
            return;
        }
        Gauge::default()
            .style(Modifier::BOLD)
            .gauge_style(Style::new().blue().on_black())
            .label(format!("Осталось {:?} сек.", time_left.as_secs()))
            .percent(
                (time_elapsed.as_millis() * 100 / session.settings.limits.answer_time.as_millis())
                    as u16,
            )
            .render(area, buf);
    }
}
