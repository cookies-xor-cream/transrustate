use reqwest;
use scraper::Html;

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

struct ConjugationTable<'a> {
    title: String,
    items: Vec<Vec<&'a str>>,
}

fn extract_conjugations_from_table(table: &Html) {
    let row_query = scraper::Selector::parse("tr").unwrap();
    let conjugation_tables: Vec<ConjugationTable> = Vec::new();

    let rows = table.select(&row_query);
    for row in rows {
        let mut text_elements = row.text();
        let conjugation_table = ConjugationTable {title: "".to_string(), items: Vec::new()};

        loop {
            if let Some(x) = text_elements.next() {
                let left = x;

                if let Some(y) = text_elements.next() {
                    let right = y;

                    let table_row = vec![left, right];
                } else {
                    break
                }    
            } else {
                break
            }
        }
    }
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