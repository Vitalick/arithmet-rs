use super::Status;
use crate::domain::banner;
use crate::domain::session::Session;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
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
