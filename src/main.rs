use reqwest;
// use scraper::Html;

fn get_conjugation_tables() -> Vec<String> {
    let response = reqwest::blocking::get(
        "https://www.wordreference.com/conj/frverbs.aspx?v=saper",
    )
        .unwrap()
        .text()
        .unwrap();

    let document = scraper::Html::parse_document(&response);
    let table_queries = scraper::Selector::parse("table.neoConj > tbody").unwrap();
    let tables = document.select(&table_queries).map(|x| x.inner_html());

    tables.collect::<Vec<String>>()
}

fn main() {
    println!("{:#?}", get_conjugation_tables()[0]); // gets the first conjugation table
}
