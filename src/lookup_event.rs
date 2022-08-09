use std::sync::Arc;

use reqwest::Client;

use crate::{
    app::App,
    conjugations::VerbConjugations,
    user_error::UserError,
    definitions::WordDefinitions
};

use rusqlite::{Connection, Result};

pub enum LookupEvent {
    Verb,
    Definition,
    Translation,
}

pub struct LookupEventHandler {
    app: Arc<tokio::sync::Mutex<App>>,
    client: Client,
    connection: Connection,
}

impl LookupEventHandler {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        let app_user_agent = "user-agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/56.0.2924.87 Safari/537.36".to_string();
        let client = reqwest::Client::builder()
            .user_agent(app_user_agent)
            .build()
            .expect("Create a client");

        let connection = LookupEventHandler::init_db();

        Self {
            app,
            client,
            connection,
        }
    }

    fn init_db() -> Connection {
        let connection = Connection::open("lang_rs.db")
            .expect("Connected to the sqlite database");

        connection.execute(
            "CREATE TABLE IF NOT EXISTS conjugations (
                id INTEGER PRIMARY KEY,
                language TEXT NOT NULL,
                verb TEXT NOT NULL,
                verb_conjugations TEXT NOT NULL
            )",
           [],
        ).expect("Initialized conjugations table");

        connection
    }

    pub async fn handle_lookup_event(&mut self, lookup_event: LookupEvent) {
        let mut app = self.app.lock().await;
        app.start_load();
        drop(app);

        match lookup_event {
            LookupEvent::Verb => {
                self.handle_verb_lookup().await;
            },
            LookupEvent::Definition => {
                self.handle_word_definition().await;
            },
            LookupEvent::Translation => {
                self.handle_word_translation().await;
            }
        };
        let mut app = self.app.lock().await;
        app.end_load();
    }

    async fn handle_verb_lookup(&mut self) {
        if let Err(err) = self.attempt_verb_lookup().await {
            let mut app = self.app.lock().await;
            app.set_error(err);
        }
    }

    async fn handle_word_definition(&mut self) {
        let app_obj = self.app.lock().await;
        let to_language = app_obj.language.clone();
        drop(app_obj);

        let from_language = "english".to_string();

        if let Err(err) = self.attempt_word_definition(
            from_language,
            to_language,
        ).await {
            let mut app = self.app.lock().await;
            app.set_error(err);
        }
    }

    async fn handle_word_translation(&mut self) {
        let app_obj = self.app.lock().await;
        let from_language = app_obj.language.clone();
        drop(app_obj);

        let to_language = "english".to_string();

        if let Err(err) = self.attempt_word_definition(
            from_language,
            to_language,
        ).await {
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
            &self.client,
        ).await?;

        // Add the conjugation to the database
        let conjugations_json = serde_json::to_string(&conjugations.clone())
            .expect("Serialized conjugations");

        self.connection.execute(
            "INSERT INTO conjugations (language, verb, verb_conjugations) values (?1, ?2, ?3)",
            &[&language.to_string(), &verb.to_string(), &conjugations_json.to_string()],
        ).expect("Inserted conjugation into the database");

        // Display the conjugation
        let mut app_obj = self.app.lock().await;
        app_obj.set_conjugations(conjugations);
        Ok(())
    }

    async fn attempt_word_definition(
        &mut self,
        to_language: String,
        from_language: String,
    ) -> Result<(), UserError> {
        let mut app_obj = self.app.lock().await;
        let word = app_obj.command_body();
        app_obj.clear_input();
        drop(app_obj);

        let tables = WordDefinitions::get_definition_tables(
            to_language,
            from_language,
            word,
            &self.client,
        )
            .await;

        if let Err(user_error) = tables {
            return Err(user_error);
        }

        let tables = tables?;

        let mut app_obj = self.app.lock().await;
        app_obj.set_definitions(tables);
        Ok(())
    }
}