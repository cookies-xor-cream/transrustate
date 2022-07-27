use reqwest;
use scraper::Html;

struct ConjugationTable<'a> {
    title: String,
    items: Vec<Vec<&'a str>>,
}

fn scrape_conjugation_tables() -> Vec<Html> {
    let response = reqwest::blocking::get(
        "https://www.wordreference.com/conj/frverbs.aspx?v=saper",
    )
        .unwrap()
        .text()
        .unwrap();

    let document = scraper::Html::parse_document(&response);
    let table_query = scraper::Selector::parse("table.neoConj").unwrap();
    let tables = document.select(&table_query).map(|x| scraper::Html::parse_fragment(&x.html()));

    tables.collect::<Vec<Html>>()
}

fn extract_conjugations_from_table(table: &Html) -> ConjugationTable {
    let mut conjugation_table = ConjugationTable {
        title: "".to_string(),
        items: Vec::new(),
    };
    
    let row_query = scraper::Selector::parse("tr").unwrap();
    let rows = table.select(&row_query);

    // flatten all text in rows
    let cell_values = rows.map(|row| {
        row.text().collect::<Vec<&str>>()
    })
        .into_iter()
        .flatten()
        .collect::<Vec<&str>>();

    conjugation_table.title = cell_values[0].to_string();

    // skip header text
    let mut text_iter = cell_values.iter().skip(2);
    
    // get pronoun/conjugation pairs remaining
    let mut conjugations: Vec<Vec<&str>> = Vec::new();
    loop {
        if let Some(pronoun) = text_iter.next() {
            if let Some(conjugation) = text_iter.next() {
                conjugations.push(vec![pronoun, conjugation]);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    conjugation_table.items = conjugations;
    conjugation_table
}

fn get_conjugation_tables() {
    let scraped_tables = scrape_conjugation_tables();
    for scraped_table in scraped_tables {
        let table = extract_conjugations_from_table(&scraped_table);
    }
}

fn main() {
    get_conjugation_tables();
}
