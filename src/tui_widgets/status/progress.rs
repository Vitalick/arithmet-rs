use super::Status;
use crate::domain::expression::ExerciseWithStartTime;
use crate::domain::session::Session;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Modifier, Style};
use ratatui::widgets::{Gauge, Widget};
use std::cmp::min;
use std::time::Duration;

#[derive(Debug)]
pub struct ProgressWidget<'a> {
    session: &'a Option<Session>,
    exercise_now: &'a Option<ExerciseWithStartTime>,
    status: Status,
}

impl<'a> ProgressWidget<'a> {
    pub fn new(
        session: &'a Option<Session>,
        exercise_now: &'a Option<ExerciseWithStartTime>,
        status: Status,
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
        if self.status != Status::AwaitingAnswer {
            return;
        }
        let session = self.session.as_ref().unwrap();
        let exercise_now = self.exercise_now.as_ref().unwrap();
        let time_elapsed = min(
            exercise_now.start_time.elapsed(),
            session.settings.limits.answer_time,
        );
        let time_left = session
            .settings
            .limits
            .answer_time
            .saturating_sub(time_elapsed);
        let ratio = time_elapsed.as_millis() as f64
            / session.settings.limits.answer_time.as_millis() as f64;
        let warning_secs = Duration::from_secs(5);
        let danger_secs = Duration::from_secs(2);
        let label_string = format!(
            "Осталось {}.{:02} сек.",
            time_left.as_secs(),
            time_left.subsec_millis() / 10
        );
        let gauge_style = if time_left <= danger_secs {
            Style::new().red()
        } else if time_left <= warning_secs {
            Style::new().yellow()
        } else {
            Style::new().blue()
        };
        Gauge::default()
            .style(Modifier::BOLD)
            .gauge_style(gauge_style)
            .label(label_string)
            .ratio(ratio)
            .render(area, buf);
    }
}
