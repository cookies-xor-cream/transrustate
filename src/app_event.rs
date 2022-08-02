use std::{time::Duration, sync::Arc, process::exit};
use crossterm::event::{KeyEvent, self, KeyCode};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::app::App;

#[derive(Copy, Clone)]
pub enum AppEvent {
    Input(KeyEvent),
    Tick,
}

pub struct AppEvents {
    rx: Receiver<AppEvent>,
    _tx: Sender<AppEvent>,
}

impl AppEvents {
    pub fn new(tick_rate: Duration) -> AppEvents {
        let (tx, rx) = channel(512);

        let event_tx = tx.clone(); // the thread::spawn own event_tx 
        tokio::spawn(async move {
            loop {
                // poll for tick rate duration, if no event, sent tick event.
                if crossterm::event::poll(tick_rate).unwrap() {
                    let event = event::read().unwrap();
                    if let event::Event::Key(key) = event {
                        let key = KeyEvent::from(key);
                        if let Err(err) = event_tx.send(AppEvent::Input(key)).await {
                            println!("Send App Input Failed, connection is closed? {}", err);
                            exit(1);
                        }
                    } else {
                        if let Err(err) = event_tx.send(AppEvent::Tick).await {
                            println!("Send App Tick Failed, connection is closed? {}", err);
                            exit(1);
                        }
                    }
                } else {
                    if let Err(err) = event_tx.send(AppEvent::Tick).await {
                        println!("Send App Tick Failed, connection is closed? {}", err);
                        exit(1);
                    }
                }
            }
        });

        AppEvents {
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
        match app_event {
            AppEvent::Input(chr) => {
                self.handle_input_event(chr).await;
            },
            AppEvent::Tick => {
                self.update_on_tick().await;
            },
        };
    }

    async fn update_on_tick(&mut self) {
        let mut _app = self.app.lock().await;
    }

    async fn handle_input_event(&mut self, input: KeyEvent) {
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
                app.pop_char();
            }
            KeyCode::Enter => {
                app.handle_entry().await;
            }
            KeyCode::Char(c) => {
                app.put_char(c);
            }
            _ => {}
        }
    }
}