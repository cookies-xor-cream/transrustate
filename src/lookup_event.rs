use std::{sync::{Arc}, time::Duration};

use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::app::App;

pub enum LookupEvent {
    Verb(String),
}

pub struct LookupEventHandler {
    app: Arc<tokio::sync::Mutex<App>>,
}

impl LookupEventHandler {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    pub async fn handle_lookup_event(&mut self, lookup_event: LookupEvent) {
        let result = match lookup_event {
            LookupEvent::Verb(verb) => self.handle_verb_lookup(verb).await,
        };
    }

    async fn handle_verb_lookup(&mut self, verb: String) -> Result<(), ()> {
        let mut app = self.app.lock().await;
        app.set_verb().await;
        Ok(())
    }
}