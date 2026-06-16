use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Span, Widget};

#[derive(Debug)]
pub struct OperationItemWidget<'a> {
    checked: bool,
    line: Line<'a>,
}

impl<'a> OperationItemWidget<'a> {
    pub fn new(checked: bool, line: Line<'a>) -> Self {
        Self { checked, line }
    }

    pub fn into_line(self) -> Line<'a> {
        let checked = if self.checked { "[x]" } else { "[ ]" };
        let mut spans = vec![Span::raw(format!("{checked}  "))];
        spans.extend(self.line.spans);
        Line::from(spans)
    }
}

impl Widget for OperationItemWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.into_line().render(area, buf);
    }
}
