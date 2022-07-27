#[path="./conjugations.rs"]
mod conjugations;
use conjugations::VerbConjugations;

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

pub struct App {
    state: TableState,
    items: Vec<Vec<String>>,
    input: String,
}

impl App {
    pub fn new(items: Vec<Vec<String>>) -> App {
        App {
            state: TableState::default(),
            items: items,
            input: String::new(),
        }
    }

    fn set_verb(&mut self) {
        let verb: String = self.input.drain(..).collect();
        let y = VerbConjugations::get_conjugation_tables(&verb);
        let x = &y.conjugation_tables[0].conjugations_as_strings();
        self.items = x.to_vec();
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
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

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["Pronouns", "Conjugation"];
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(c.as_str()));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Table"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);

        let input = Paragraph::new(app.input.as_ref())
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title("Input"));
        f.render_widget(input, rects[0]);
        f.render_stateful_widget(t, rects[1], &mut app.state);
}
