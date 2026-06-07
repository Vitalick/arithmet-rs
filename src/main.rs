use std::sync::{
    Arc,
    atomic::AtomicBool,
};

use arithmet::app::App;
use color_eyre::{Result, eyre::WrapErr};

#[cfg(unix)]
fn install_signal_handler(shutdown_requested: Arc<AtomicBool>) -> Result<()> {
    use signal_hook::{consts::signal::{SIGINT, SIGTERM}, flag};

    flag::register(SIGINT, Arc::clone(&shutdown_requested))
        .wrap_err("install SIGINT handler failed")?;
    flag::register(SIGTERM, shutdown_requested).wrap_err("install SIGTERM handler failed")?;
    Ok(())
}

#[cfg(not(unix))]
fn install_signal_handler(_shutdown_requested: Arc<AtomicBool>) -> Result<()> {
    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    install_signal_handler(Arc::clone(&shutdown_requested))?;

    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal, shutdown_requested);
    ratatui::restore();
    app_result
}
