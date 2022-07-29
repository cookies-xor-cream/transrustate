use reqwest;
use scraper::{Html};

use crate::wordreference::wordreference_utils;

pub struct ConjugationTable {
    pub tense: String,
    pub conjugations: Vec<Vec<String>>,
}

impl ConjugationTable {
    fn new(cell_values: Vec<String>) -> ConjugationTable {
        // skip header text
        let mut text_iter = cell_values.iter().skip(1);
        
        // get pronoun/conjugation pairs remaining
        let mut conjugations: Vec<Vec<String>> = Vec::new();
        loop {
            if let Some(pronoun) = text_iter.next() {
                if let Some(conjugation) = text_iter.next() {
                    conjugations.push(vec![pronoun.to_string(), conjugation.to_string()]);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let tense = cell_values[0].to_string();

        ConjugationTable {
            tense: tense,
            conjugations: conjugations,
        }
    }

    pub fn conjugations_as_strings(&self) -> Vec<Vec<String>> {
        let mut conj_table: Vec<Vec<String>> = Vec::new();
        for vector in &self.conjugations {
            let mut conj_row: Vec<String> = Vec::new();
            for string in vector {
                conj_row.push(string.to_string());
            }
            conj_table.push(conj_row);
        }

        conj_table
    }
}

pub struct VerbConjugations {
    pub conjugation_tables: Vec<ConjugationTable>,
}

impl VerbConjugations {
    fn new() -> VerbConjugations {
        VerbConjugations {
            conjugation_tables: Vec::new(),
        }
    }

    pub fn empty() -> VerbConjugations {
        VerbConjugations::new()
    }

    async fn scrape_conjugation_tables(&self, verb: &str, language: &str) -> Vec<Html> {
        let verb_query_url = wordreference_utils::conjugation_url(
            language.to_string(),
            verb.to_string(),
        );

        let response = reqwest::get(
            verb_query_url,
        )
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let document = scraper::Html::parse_document(&response);
        let table_query = scraper::Selector::parse("table.neoConj").unwrap();
        let tables = document
            .select(&table_query)
            .map(|x| scraper::Html::parse_fragment(&x.html()));

        tables.collect::<Vec<Html>>()
    }

    fn extract_conjugations_from_table(&mut self, table: Html) {        
        let row_query = scraper::Selector::parse("tr").unwrap();
        let cell_query = scraper::Selector::parse("td, th").unwrap();

        let rows = table.select(&row_query);

        // concatenate all text inside a cell first
        // this is because there can be further nested elements
        let cell_values = rows.map(|row| {
            let cells = row.select(&cell_query);
            cells.map(|cell| {
                cell.text().collect::<String>()
            })
                .collect::<Vec<String>>()
        })
            .flatten()
            .collect::<Vec<String>>();

        self.conjugation_tables.push(ConjugationTable::new(cell_values));
    }

    pub async fn get_conjugation_tables(verb: &str, language: &str) -> VerbConjugations {
        let mut verb_conjugations = VerbConjugations::new();
        let tables = verb_conjugations.scrape_conjugation_tables(verb, language).await;
        for table in tables {
            verb_conjugations.extract_conjugations_from_table(table);
        }

        verb_conjugations
    }
}