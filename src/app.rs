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
}

impl App {
    pub fn new(conj: VerbConjugations) -> App {
        App {
            state: TableState::default(),
            conjugations: conj,
            table_data: TableData::new(),
            input: String::new(),
            current_table: 0,
        }
    }

    fn set_verb(&mut self) {
        let verb: String = self.input.drain(..).collect();
        self.conjugations = VerbConjugations::get_conjugation_tables(&verb);
        self.current_table = 0;
        self.set_table_data();
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
                    app.set_verb();
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
    f.render_stateful_widget(current_conjugation_table, rects[1], &mut app.state);

    let input = Paragraph::new(app.input.as_ref())
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title("Verb Input"));

    f.render_widget(input, rects[0]);
}
