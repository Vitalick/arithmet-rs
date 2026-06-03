use arithmet::domain::banner;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph};
use ratatui::{DefaultTerminal, Frame};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(render)?;
        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    let hello_world_text = banner::render("Hello, World!");
    let text = Text::from(
        hello_world_text
            .iter()
            .map(|x| Line::from(x.to_string()))
            .collect::<Vec<_>>(),
    );
    let para = Paragraph::new(text);
    frame.render_widget(para, frame.area());
    // frame.render_widget("hello world", frame.area());
}
