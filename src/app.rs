use crate::conjugations::VerbConjugations;

use reqwest;
use scraper::{ElementRef, Html};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout, Direction},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState, Paragraph},
    Frame, Terminal,
};

use crate::wordreference::wordreference_utils;

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
    conjugations: VerbConjugations,
    table_data: TableData,
    input: String,
    current_table: usize,
    language: String,
}

impl App {
    pub fn new() -> App {
        let default_language = "french".to_string();
        App {
            state: TableState::default(),
            conjugations: VerbConjugations::empty(),
            table_data: TableData::new(),
            input: String::new(),
            current_table: 0,
            language: default_language,
        }
    }

    fn remove_prefix(&mut self) {
        self.input = self.input
            .split(" ")
            .skip(1)
            .collect::<String>();
    }

    fn set_verb(&mut self) {
        self.remove_prefix();
        let verb: String = self.input.drain(..).collect();
        self.conjugations = VerbConjugations::get_conjugation_tables(verb.as_str(), self.language.as_str());
        self.current_table = 0;
        self.set_table_data();
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
        self.input = "".to_string();
    }

    fn handle_entry(&mut self) {
        let string = self.input.as_str();
        match string {
            _ if string.starts_with("lang") => self.set_language(),
            _ if string.starts_with("conj") => self.set_verb(),
            _ if string.starts_with("help") => self.display_help(),
            _ => self.handle_error(),
        };
    }

    fn set_table_data(&mut self) {
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

    fn conjugation_table_open(&self) -> bool {
        self.conjugations.conjugation_tables.len() > 0
    }

    fn next(&mut self) {
        let num_tables = self.conjugations.conjugation_tables.len();
        self.current_table = (self.current_table + 1) % num_tables;
        self.set_table_data();
    }

    fn prev(&mut self) {
        let num_tables = self.conjugations.conjugation_tables.len();
        self.current_table = (self.current_table + num_tables - 1) % num_tables;
        self.set_table_data();
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => return Ok(()),
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
                    app.handle_entry();
                }
                KeyCode::Char(c) => {
                    app.input.push(c);
                }
                _ => {}
            }
        }
    }
}

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Percentage(90),
            ]
            .as_ref()
        )
        .margin(5)
        .split(f.size());

    let input = Paragraph::new(app.input.as_ref())
    .style(Style::default())
    .block(Block::default().borders(Borders::ALL).title("Verb Input"));

    f.render_widget(input, rects[0]);

    if (app.conjugation_table_open()) {
        let reversed_style = Style::default().add_modifier(Modifier::REVERSED);
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
            ]);

        f.render_stateful_widget(current_conjugation_table, chunks[1], &mut app.state);
    }
}
