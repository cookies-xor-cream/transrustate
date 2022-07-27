use reqwest;
use scraper::Html;
// use scraper::Html;

fn get_conjugation_tables() -> Vec<Html> {
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

fn extract_conjugations_from_table(table: &Html) {
    let row_query = scraper::Selector::parse("tr").unwrap();

    let rows = table.select(&row_query);
    for row in rows {
        for cell in row.text() {
            println!("cell: {:#?}", cell);
        }
    }
}

fn main() {
    let conjugation_tables = get_conjugation_tables();
    for conjugation_table in conjugation_tables {
        println!("Conjugation Table:");
        extract_conjugations_from_table(&conjugation_table);

        println!("");
    }
}
