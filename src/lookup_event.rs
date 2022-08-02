use std::sync::Arc;

use crate::{app::App, conjugations::VerbConjugations, user_error::UserError};

pub enum LookupEvent {
    Verb,
}

pub struct LookupEventHandler {
    app: Arc<tokio::sync::Mutex<App>>,
}

impl LookupEventHandler {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    pub async fn handle_lookup_event(&mut self, lookup_event: LookupEvent) {
        let mut app = self.app.lock().await;
        app.clear_error();
        drop(app);

        match lookup_event {
            LookupEvent::Verb => {
                self.handle_verb_lookup().await;
            },
        };
    }

    async fn handle_verb_lookup(&mut self) {
        if let Err(err) = self.attempt_verb_lookup().await {
            let mut app = self.app.lock().await;
            app.set_error(err);
        }
    }

    async fn attempt_verb_lookup(&mut self) -> Result<(), UserError> {
        let mut app_obj = self.app.lock().await;
        let verb = app_obj.command_body();
        app_obj.clear_input();

        let language = app_obj.language.clone();

        drop(app_obj);

        let conjugations = VerbConjugations::get_conjugation_tables(
            verb.as_str(),
            language.as_str(),
        ).await;

        let mut app_obj = self.app.lock().await;
        app_obj.conjugations = conjugations?;
        app_obj.current_table = 0;
        app_obj.set_table_data();
        Ok(())
    }
}