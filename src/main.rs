pub mod app;

use std::io;

use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};


fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let app_result = app::App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}
