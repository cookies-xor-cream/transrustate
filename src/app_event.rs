use std::{time::Duration, thread, sync::{mpsc::{channel, Receiver, Sender, RecvError}, Arc}};
use crossterm::event::{Event, KeyEvent, self, KeyCode};

use crate::app::App;

pub enum AppEvent {
    Input(KeyEvent),
    Tick,
}

pub struct Events {
    rx: Receiver<AppEvent>,
    _tx: Sender<AppEvent>,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Events {
        let (tx, rx) = channel();

        let event_tx = tx.clone(); // the thread::spawn own event_tx 
        thread::spawn(move || {
            loop {
                // poll for tick rate duration, if no event, sent tick event.
                if crossterm::event::poll(tick_rate).unwrap() {
                    if let event::Event::Key(key) = event::read().unwrap() {
                        let key = KeyEvent::from(key);
                        event_tx.send(AppEvent::Input(key)).unwrap();
                    }
                }
                event_tx.send(AppEvent::Tick).unwrap();
            }
        });

        Events { rx, _tx: tx }
    }

    /// Attempts to read an event.
    /// This function block the current thread.
    pub fn next(&self) -> Result<AppEvent, RecvError> {
        self.rx.recv()
    }
}

pub struct AppEventHandler {
    app: Arc<tokio::sync::Mutex<App>>,
}

impl AppEventHandler {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    pub async fn handle_app_event(&mut self, app_event: AppEvent) {
        let result = match app_event {
            AppEvent::Input(chr) => self.handle_input_event(chr).await,
            AppEvent::Tick => Ok(()),
        };
    }

    async fn handle_input_event(&mut self, input: KeyEvent) -> Result<(), ()> {
        let mut app = self.app.lock().await;

        if let Event::Key(key) = event::read().expect("Read an event") {
            match key.code {
                KeyCode::Esc => app.close(),
                KeyCode::Right => {
                    app.next();
                }
                KeyCode::Left => {
                    app.prev();
                }
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Enter => {
                    app.handle_entry().await;
                }
                KeyCode::Char(c) => {
                    app.input.push(c);
                }
                _ => {}
            }
        }

        Ok(())
    }
}