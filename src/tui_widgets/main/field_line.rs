use super::CursorType;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Span, Stylize, Widget};

#[derive(Debug, Clone)]
pub struct FieldLineWidget<'a> {
    label: Line<'a>,
    value: String,
    active: bool,
    brackets: bool,
    colon: bool,
    cursor: CursorType,
}

impl<'a> FieldLineWidget<'a> {
    pub fn new(
        label: Line<'a>,
        value: String,
        active: bool,
        brackets: bool,
        colon: bool,
        cursor: CursorType,
    ) -> Self {
        Self {
            label,
            value,
            active,
            brackets,
            colon,
            cursor,
        }
    }

    pub fn into_line(self) -> Line<'a> {
        let mut spans = self.label.spans;
        if self.colon {
            spans.push(Span::raw(":"));
        }
        spans.push(Span::raw(" "));

        let value = if self.active {
            format!("{}{}", self.value, self.cursor)
        } else {
            self.value
        };
        let value = if self.brackets {
            format!("[ {} ]", value)
        } else {
            value
        };

        let value = if self.active {
            value.blue().bold()
        } else {
            Span::raw(value)
        };
        spans.push(value);
        Line::from(spans)
    }
}

impl Widget for FieldLineWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.into_line().render(area, buf);
    }
}
