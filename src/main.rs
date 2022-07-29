mod app;
mod conjugations;
mod wordreference;

use app::{App, ui, run_app};
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
    let (sync_io_tx, mut sync_io_rx) = tokio::sync::mpsc::channel::<Event>(100);

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    // let app = App::new();
    let app = Arc::new(tokio::sync::Mutex::new(App::new(/* sync_io_tx.clone() */)));
    // let app_ui = Arc::clone(&app);

    // tokio::spawn(async move {
    //     // let mut handler = IoAsyncHandler::new(app);
    //     // while let Some(io_event) = sync_io_rx.recv().await {
    //     //     // handler.handle_io_event(io_event).await;
    //     // }
    // });

    let res = run_app(&mut terminal, &app).await;

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
