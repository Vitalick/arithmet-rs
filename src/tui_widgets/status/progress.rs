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
        let warning_secs = Duration::from_secs(5);
        let danger_secs = Duration::from_secs(2);
        // let label_string = if time_left <= danger_secs {
        //     format!("Осталось {}.{:02} сек.", time_left.as_secs(), time_left.as_millis() % Duration::from_secs(1).as_millis() / 10)
        // } else {
        //     format!("Осталось {} сек.", time_left.as_secs()+1)
        // };
        let label_string = format!("Осталось {}.{:02} сек.", time_left.as_secs(), time_left.as_millis() % Duration::from_secs(1).as_millis() / 10);
        let gauge_style = if time_left <= danger_secs {
            Style::new().red().on_black()
        } else if time_left <= warning_secs {
            Style::new().yellow().on_black()
        } else {
            Style::new().blue().on_black()
        };
        Gauge::default()
            .style(Modifier::BOLD)
            .gauge_style(gauge_style)
            .label(label_string)
            .ratio(ratio)
            .render(area, buf);
    }
}
