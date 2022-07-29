mod app;
mod conjugations;
mod wordreference;
mod app_event;

use app::{App, ui, run_app};
use app_event::{AppEventHandler, AppEvent};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, sync::Arc};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};

use conjugations::VerbConjugations;

async fn start_app() -> Result<(), Box<dyn Error>> {
    let (sync_io_tx, mut sync_io_rx) = tokio::sync::mpsc::channel::<AppEvent>(100);

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = Arc::new(tokio::sync::Mutex::new(App::new(sync_io_tx.clone())));
    let app_ui = Arc::clone(&app);

    tokio::spawn(async move {
        let mut handler = AppEventHandler::new(app);
        while let Some(app_event) = sync_io_rx.recv().await {
            handler.handle_app_event(app_event).await;
        }
    });

    let res = run_app(&mut terminal, &app_ui).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    start_app().await;
}
