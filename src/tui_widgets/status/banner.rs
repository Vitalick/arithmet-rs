use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use crate::domain::banner;
use crate::domain::expression::ExerciseWithStartTime;
use crate::domain::session::Session;
use super::Status;

pub struct BannerWidget {
    session: Option<Session>,
    exercise_now: Option<ExerciseWithStartTime>,
    status: Status,
}

impl BannerWidget {
    pub fn new(session: Option<Session>, exercise_now: Option<ExerciseWithStartTime>, status: Status) -> Self {
        BannerWidget { session, exercise_now, status }
    }
}

impl Widget for BannerWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let banner_text = match self.status {
            Status::Welcome => "Добро пожаловать!".to_string(),
            Status::AwaitingGameContinue => self.session.as_ref().unwrap().last_answer_banner(),
            Status::GameFinished => {
                format!(
                    "Ваша оценка: {}",
                    self.session.as_ref().unwrap().get_grade()
                )
            }
            _ => String::default(),
        };
        if banner_text.is_empty() {
            return;
        }

        let banner_paragraph = banner::render_to_paragraph(banner_text.as_str());

        banner_paragraph.centered().render(area, buf);
    }
}
