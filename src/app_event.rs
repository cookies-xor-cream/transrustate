use std::{time::Duration, sync::Arc};
use crossterm::event::{Event, KeyEvent, self, KeyCode};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use tokio::fs::File;
use std::io::Write;

use crate::app::App;

#[derive(Copy, Clone)]
pub enum AppEvent {
    Input(KeyEvent),
    Tick,
    Close,
}

pub struct Events {
    rx: Receiver<AppEvent>,
    _tx: Sender<AppEvent>,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Events {
        let (tx, rx) = channel(512);

        let event_tx = tx.clone(); // the thread::spawn own event_tx 
        tokio::spawn(async move {
            loop {
                // poll for tick rate duration, if no event, sent tick event.
                if crossterm::event::poll(tick_rate).unwrap() {
                    let event = event::read().unwrap();
                    if let event::Event::Key(key) = event {
                        let key = KeyEvent::from(key);
                        event_tx.send(AppEvent::Input(key)).await;
                    } else {
                        event_tx.send(AppEvent::Tick).await;
                    }
                } else {
                    event_tx.send(AppEvent::Tick).await;
                }
            }
        });

        Events {
            rx,
            _tx: tx
        }
    }

    /// Attempts to read an event.
    /// This function block the current thread.
    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
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
            AppEvent::Tick => self.update_on_tick().await,
            AppEvent::Close => self.handle_close().await,
        };
    }

    async fn update_on_tick(&mut self) -> Result<(), ()> {
        let mut app = self.app.lock().await;
        Ok(())
    }

    async fn handle_close(&mut self) -> Result<(), ()> {
        let mut app = self.app.lock().await;
        app.close();
        Ok(())
    }

    async fn handle_input_event(&mut self, input: KeyEvent) -> Result<(), ()> {
        let mut app = self.app.lock().await;

        match input.code {
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

        Ok(())
    }
}