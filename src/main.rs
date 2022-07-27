use reqwest;
use scraper::{Html};

struct ConjugationTable {
    tense: String,
    conjugations: Vec<Vec<String>>,
}

impl ConjugationTable {
    fn new(cell_values: Vec<&str>) -> ConjugationTable {
        // skip header text
        let mut text_iter = cell_values.iter().skip(2);
        
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
}

struct VerbConjugations {
    conjugation_tables: Vec<ConjugationTable>,
}

impl VerbConjugations {
    fn new() -> VerbConjugations {
        VerbConjugations {
            conjugation_tables: Vec::new(),
        }
    }

    fn scrape_conjugation_tables(&self) -> Vec<Html> {
        let response = reqwest::blocking::get(
            "https://www.wordreference.com/conj/frverbs.aspx?v=saper",
        )
            .unwrap()
            .text()
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
        let rows = table.select(&row_query);

        // flatten all text in rows
        let cell_values = rows.map(|row| {
            row.text().collect::<Vec<&str>>()
        })
            .into_iter()
            .flatten()
            .collect::<Vec<&str>>();

        self.conjugation_tables.push(ConjugationTable::new(cell_values));
    }

    pub fn get_conjugation_tables() -> VerbConjugations {
        let mut verb_conjugations = VerbConjugations::new();
        let tables = verb_conjugations.scrape_conjugation_tables();
        for table in tables {
            verb_conjugations.extract_conjugations_from_table(table);
        }

        verb_conjugations
    }
}

fn main() {
    let verb_conjugations = VerbConjugations::get_conjugation_tables();
    for table in verb_conjugations.conjugation_tables {
        println!("{} {:?}", table.tense, table.conjugations);
    }
}
