use super::Status;
use crate::domain::banner;
use crate::domain::session::Session;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

#[derive(Debug)]
pub struct BannerWidget<'a> {
    session: &'a Option<Session>,
    status: Status,
}

impl<'a> BannerWidget<'a> {
    pub fn new(session: &'a Option<Session>, status: Status) -> Self {
        BannerWidget { session, status }
    }
}

impl Widget for BannerWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let banner_text = match self.status {
            Status::Welcome => "Добро пожаловать!".to_string(),
            Status::AwaitingGameContinue => self.session.as_ref().unwrap().last_answer_banner(),
            Status::GameFinished => {
                format!(
                    "Ваша оценка {}",
                    self.session.as_ref().unwrap().get_grade().value()
                )
            }
            Status::ResultsView => String::default(),
            _ => String::default(),
        };
        if banner_text.is_empty() {
            return;
        }

        let mut banner_paragraph = banner::render_to_paragraph(banner_text.as_str());
        if let Some(style) = self.banner_style() {
            banner_paragraph = banner_paragraph.style(style);
        }

        banner_paragraph.centered().render(area, buf);
    }
}

impl BannerWidget<'_> {
    fn banner_style(&self) -> Option<Style> {
        match self.status {
            Status::AwaitingGameContinue => {
                let answer = self.session.as_ref()?.last_answer?;
                if answer.is_correct() {
                    Some(Style::new().green())
                } else {
                    Some(Style::new().red())
                }
            }
            Status::GameFinished => Some(grade_style(
                self.session.as_ref().unwrap().get_grade().value(),
            )),
            Status::ResultsView => None,
            _ => None,
        }
    }
}

fn grade_style(value: u8) -> Style {
    match value {
        5 => Style::new().green(),
        4 => Style::new().light_green(),
        3 => Style::new().yellow(),
        2 => Style::new().light_red(),
        _ => Style::new().red(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::expression::{Exercise, ExerciseWithStartTime};
    use crate::domain::operation::Operation;
    use crate::domain::settings::Settings;
    use ratatui::style::Color;

    fn session_with_answer(entered: i64) -> Session {
        let mut session = Session::new(Settings::default()).unwrap();
        session.exercise_now = Some(ExerciseWithStartTime::new(Exercise::new(
            2,
            Operation::Addition,
            3,
        )));
        session.answer(Ok(entered));
        session
    }

    #[test]
    fn last_answer_banner_is_green_for_correct_answer() {
        let session = Some(session_with_answer(5));
        let widget = BannerWidget::new(&session, Status::AwaitingGameContinue);

        assert_eq!(widget.banner_style().unwrap().fg, Some(Color::Green));
    }

    #[test]
    fn last_answer_banner_is_red_for_wrong_answer() {
        let session = Some(session_with_answer(4));
        let widget = BannerWidget::new(&session, Status::AwaitingGameContinue);

        assert_eq!(widget.banner_style().unwrap().fg, Some(Color::Red));
    }
}
