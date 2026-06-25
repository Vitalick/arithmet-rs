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
            _ => String::default(),
        };
        if banner_text.is_empty() {
            return;
        }

        let mut banner_paragraph = banner::render_to_paragraph(banner_text.as_str());
        if self.status == Status::GameFinished {
            banner_paragraph = banner_paragraph.style(grade_style(
                self.session.as_ref().unwrap().get_grade().value(),
            ));
        }

        banner_paragraph.centered().render(area, buf);
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
