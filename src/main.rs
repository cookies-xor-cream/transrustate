mod app;
mod conjugations;
mod wordreference;
mod app_event;
mod lookup_event;
mod user_error;
mod definitions;

use app::{App, run_app};
use app_event::{AppEventHandler, AppEvent};
use lookup_event::{LookupEventHandler, LookupEvent};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::sync::mpsc::channel;
use std::{error::Error, io, sync::Arc, process::exit};
use tui::{backend::CrosstermBackend, Terminal,};

async fn start_app() -> Result<(), Box<dyn Error>> {
    let (sync_io_tx, mut sync_io_rx) = channel::<AppEvent>(512);
    let (sync_lookup_tx, mut sync_lookup_rx) = channel::<LookupEvent>(512);

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = Arc::new(
        tokio::sync::Mutex::new(
            App::new(
                sync_io_tx.clone(),
                sync_lookup_tx.clone(),
            )
        )
    );
    let app_io = Arc::clone(&app);
    let app_lookup = Arc::clone(&app);
    let app_ui = Arc::clone(&app);

    tokio::spawn(async move {
        let mut handler = AppEventHandler::new(app_io);
        while let Some(app_event) = sync_io_rx.recv().await {
            handler.handle_app_event(app_event).await;
        }
    });

    tokio::spawn(async move {
        let mut handler = LookupEventHandler::new(app_lookup);
        while let Some(lookup_event) = sync_lookup_rx.recv().await {
            handler.handle_lookup_event(lookup_event).await;
        }
    });

    run_app(&mut terminal, &app_ui).await?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = start_app().await {
        eprintln!("{}", err);
        exit(1);
    } else {
        exit(0);
    }
}
