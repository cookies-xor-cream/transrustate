use crate::{
    conjugations::VerbConjugations,
    app_event::{
        AppEvent,
        AppEvents
    },
    lookup_event::LookupEvent, user_error::UserError
};

use std::{io, sync::Arc, time::Duration};
use tui::{
    backend::{Backend},
    layout::{Constraint, Layout, Direction, Alignment},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState, Paragraph, Gauge, Wrap},
    Frame, Terminal,
};

pub struct TableData {
    title: String,
    header: Vec<String>,
    items: Vec<Vec<String>>,
}

impl TableData {
    fn new() -> TableData {
        TableData {
            title: String::new(),
            header: Vec::new(),
            items: Vec::new(),
        }
    }
}

pub struct App {
    state: TableState,
    pub conjugations: VerbConjugations,
    pub table_data: TableData,
    pub input: String,
    pub current_table: usize,
    pub language: String,
    error: String,
    io_tx: tokio::sync::mpsc::Sender<AppEvent>,
    lookup_tx: tokio::sync::mpsc::Sender<LookupEvent>,
    closed: bool,
}

impl App {
    pub fn new(
        io_tx: tokio::sync::mpsc::Sender<AppEvent>,
        lookup_tx: tokio::sync::mpsc::Sender<LookupEvent>,
    ) -> App {
        let default_language = "french".to_string();
        App {
            state: TableState::default(),
            conjugations: VerbConjugations::empty(),
            table_data: TableData::new(),
            input: String::new(),
            current_table: 0,
            language: default_language,
            io_tx,
            lookup_tx,
            error: "".to_string(),
            closed: false,
        }
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn set_error(&mut self, error: UserError) {
        self.error = error.message;
        self.clear_tables();
    }

    pub fn clear_error(&mut self) {
        self.error = "".to_string();
    }

    pub fn clear_tables(&mut self) {
        self.conjugations = VerbConjugations::empty();
        self.table_data = TableData::new();
        self.current_table = 0;
        self.state = TableState::default();
    }

    pub fn command_body(&self) -> String {
        self.input
            .split(" ")
            .skip(1)
            .collect::<String>()
    }

    pub fn clear_input(&mut self) {
        self.input = "".to_string();
    }

    pub async fn dispatch_io(&mut self, action: AppEvent) {
        if let Err(_err) = self.io_tx.send(action).await {
        };
    }

    pub async fn dispatch_lookup(&mut self, action: LookupEvent) {
        if let Err(_err) = self.lookup_tx.send(action).await {
        };

    }

    fn remove_prefix(&mut self) {
        self.input = self.input
            .split(" ")
            .skip(1)
            .collect::<String>();
    }

    pub async fn set_verb(&mut self) {
        self.dispatch_lookup(LookupEvent::Verb).await;
    }

    fn set_language(&mut self) {
        self.remove_prefix();
        let language: String = self.input.drain(..).collect();
        self.language = language;
    }

    fn display_help(&mut self) {
        // TODO
    }

    fn handle_error(&mut self) {
        // TODO
        self.clear_input();
    }

    pub async fn handle_entry(&mut self) {
        let string = self.input.as_str();
        match string {
            _ if string.starts_with("lang") => self.set_language(),
            _ if string.starts_with("conj") => self.set_verb().await,
            _ if string.starts_with("help") => self.display_help(),
            _ => self.handle_error(),
        };
    }

    pub fn set_table_data(&mut self) {
        if self.conjugations.conjugation_tables.len() > self.current_table {
            let items = self.conjugations.conjugation_tables[self.current_table].conjugations_as_strings();
            let tense = (&self.conjugations.conjugation_tables[self.current_table].tense).clone();
            self.table_data = TableData {
                title: tense,
                header: vec![
                    "Pronouns".to_string(),
                    "Conjugations".to_string(),
                ],
                items: items,
            };
        }
    }

    fn conjugation_table_open(&self) -> bool {
        self.conjugations.conjugation_tables.len() > 0
    }

    pub fn next(&mut self) {
        let num_tables = self.conjugations.conjugation_tables.len();

        if num_tables > 0 {
            self.current_table = (self.current_table + 1) % num_tables;
            self.set_table_data();
        }
    }

    pub fn prev(&mut self) {
        let num_tables = self.conjugations.conjugation_tables.len();

        if num_tables > 0 {
            self.current_table = (self.current_table + num_tables - 1) % num_tables;
            self.set_table_data();
        }
    }
}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &Arc<tokio::sync::Mutex<App>>) -> io::Result<()> {
    let tick_rate = Duration::from_millis(200);
    let mut app_events = AppEvents::new(tick_rate);

    loop {
        let mut app = app.lock().await;

        terminal.draw(|f| ui(f, &mut app))?;

        match app_events.next().await.unwrap() {
            AppEvent::Input(key_event) => {
                app.dispatch_io(AppEvent::Input(key_event)).await;
            },
            AppEvent::Tick => {
                app.dispatch_io(AppEvent::Tick).await;
            },
        };

        if app.closed {
            break;
        }
    }

    Ok(())
}

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let default_style = Style::default().fg(Color::Yellow).bg(Color::Black);

    let screen_block = Block::default().style(default_style);
    f.render_widget(screen_block, f.size());

    let vertical_divide = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Percentage(90),
            ]
            .as_ref()
        )
        .margin(5)
        .split(f.size());

    let top_bar_area = vertical_divide[0];
    let error_display_area = vertical_divide[1];
    let content_area = vertical_divide[2];

    let top_bar_divide = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(80),
                Constraint::Percentage(20),
            ]
            .as_ref()
        )
        // .margin(5)
        .split(top_bar_area);

    let prompt_rect = top_bar_divide[0];
    let guage_rect = top_bar_divide[1];
    let tables_rect = content_area;

    let input = Paragraph::new(app.input.as_ref())
        .style(default_style.add_modifier(Modifier::SLOW_BLINK))
        .block(Block::default().borders(Borders::ALL).title("Command Prompt"));

    f.render_widget(input, prompt_rect);

    let guage = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(default_style)
                .title("Loading")
        )
        .gauge_style(default_style.add_modifier(Modifier::ITALIC))
        .percent(70);

    f.render_widget(guage, guage_rect);

    if app.error.len() > 0 {
        let error_display = Paragraph::new(app.error.as_str())
            .block(Block::default().title("Error Message").borders(Borders::ALL).style(default_style.fg(Color::Red)))
            .style(default_style.fg(Color::Red))
            .wrap(Wrap { trim: true });

        f.render_widget(error_display, error_display_area);
    }

    if app.conjugation_table_open() {
        let reversed_style = default_style.add_modifier(Modifier::REVERSED);
        let header_cells = app.table_data.header.clone();
        let header = Row::new(header_cells)
            .style(reversed_style)
            .height(1);
        let rows = app.table_data.items
            .iter()
            .map(|item| {
                let height = 1;
                let cells = item.iter().map(|c| Cell::from(c.as_str()));
                Row::new(cells).height(height)
            });

        let current_conjugation_table = Table::new(rows)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title((&app.table_data.title).clone())
            )
            .widths(&[
                Constraint::Percentage(50),
                Constraint::Length(30),
                Constraint::Min(10),
            ])
            .style(default_style);

        f.render_stateful_widget(current_conjugation_table, tables_rect, &mut app.state);
    }
}
