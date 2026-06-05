use color_eyre::{
    Result,
    eyre::{WrapErr, bail},
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Constraint;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn exit(&mut self) -> Result<()> {
        self.exit = true;
        Ok(())
    }

    fn increment_counter(&mut self) -> Result<()> {
        self.counter += 1;
        if self.counter > 2 {
            bail!("counter overflow");
        }
        Ok(())
    }

    fn decrement_counter(&mut self) -> Result<()> {
        if self.counter == 0 {
            bail!("counter is must be greater than 0.");
        }
        self.counter -= 1;
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.exit()?,
            KeyCode::Left => self.decrement_counter()?,
            KeyCode::Right => self.increment_counter()?,
            _ => {}
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)?
            }
            _ => {}
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Устный Счёт ".bold());

        let instructions = Line::from(vec![
            " ".into(),
            "Уменьшить".into(),
            " ".into(),
            "<Left>".blue().bold(),
            " ".into(),
            "Увеличить".into(),
            " ".into(),
            "<Right>".blue().bold(),
            " ".into(),
            "Выход".into(),
            " ".into(),
            "<Q> ".into(),
            " ".into(),
        ]);

        let main_block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        let [top, main] = main_block
            .inner(area)
            .layout(&Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).spacing(1));
        let version = Line::from(format!("версия {}", env!("CARGO_PKG_VERSION")));
        let developer = Line::from(format!("{}", 2026));

        Paragraph::new(vec![version, developer])
            .centered()
            .render(top, buf);

        let [left, center, right] = main_block.inner(main).layout(
            &Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Fill(2),
                Constraint::Fill(1),
            ])
            .spacing(1),
        );

        Paragraph::new(counter_text).centered().render(center, buf);


        main_block.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;
    //
    // #[test]
    // fn render() {
    //     let app = App::default();
    //     let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));
    //
    //     app.render(buf.area, &mut buf);
    //
    //     let mut expected = Buffer::with_lines(vec![
    //         "┏━━━━━━━━━━━━━━ Приложение счётчик ━━━━━━━━━━━━━━┓",
    //         "┃                    Value: 0                    ┃",
    //         "┃                                                ┃",
    //         "┗━ Уменьшить <Left> Увеличить <Right> Выход <Q> ━┛",
    //     ]);
    //     let title_style = Style::new().bold();
    //     let counter_style = Style::new().yellow();
    //     let key_style = Style::new().blue().bold();
    //     expected.set_style(Rect::new(15, 0, 20, 1), title_style);
    //     expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
    //     expected.set_style(Rect::new(13, 3, 6, 1), key_style);
    //     expected.set_style(Rect::new(30, 3, 7, 1), key_style);
    //
    //     assert_eq!(buf, expected);
    // }

    // #[test]
    // fn handle_key_event() {
    //     let mut app = App::default();
    //     app.handle_key_event(KeyCode::Right.into());
    //     assert_eq!(app.counter, 1);
    //
    //     app.handle_key_event(KeyCode::Left.into());
    //     assert_eq!(app.counter, 0);
    //
    //     let mut app = App::default();
    //     app.handle_key_event(KeyCode::Char('q').into());
    //     assert!(app.exit);
    // }
}
