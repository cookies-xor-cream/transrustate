
use reqwest;
use scraper::{Html, ElementRef};
use crate::wordreference::wordreference_utils;

pub struct DefinitionTable {
    pub header: Vec<String>, // [1, 2]
    pub definitions: Vec<Vec<String>>, // [[1, 2], [3, 4]]
}

pub async fn get_table_or_smth(language: String, word: String) -> DefinitionTable {
    let word_query_url = wordreference_utils::definition_url(
        language,
        word,
    );

    let APP_USER_AGENT = "user-agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/56.0.2924.87 Safari/537.36".to_string();

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .unwrap();

    let response = client
        .get(word_query_url)
        .send()
        .await
        .unwrap()
        .text_with_charset("utf-8")
        .await
        .unwrap();

    let document = scraper::Html::parse_document(&response);
    let table_query = scraper::Selector::parse("table.WRD")
        .expect("word definitions should have a table with the WRD class");
    let table = document
        .select(&table_query)
        .nth(0)
        .unwrap();

    let table = scraper::Html::parse_fragment(&table.html());

    let from_word_to_word_query = scraper::Selector::parse(
        "tr > td.FrWrd, tr > td.ToWrd"
    )
        .expect("td.FrWrd and td.ToWrd exist");

    let from_query = scraper::Selector::parse(
        "em[data-lang=\"en\"], span[data-ph=\"sLang_en\"]"
    )
        .expect("todo");

    let words = table
        .select(&from_word_to_word_query)
        .map(|x| scraper::Html::parse_fragment(&x.html()))
        .collect::<Vec<Html>>();

    let mut from_words: Vec<String> = Vec::new();
    let mut to_words: Vec<Vec<String>> = Vec::new();

    for x in words {
        let a = x.select(&from_query).collect::<Vec<ElementRef>>();
        let b = x
            .root_element()
            .text()
            .collect::<String>(); // take first element instead?

        if a.len() == 0 {
            from_words.push(b);
            to_words.push(Vec::new());
        } else {
            let l = to_words.last_mut().unwrap();
            l.push(b);
        }
    }

    // let table_cells: Vec<Vec<String>> = Vec::new();
    let iter = std::iter::zip(
        from_words.iter(),
        to_words.iter()
    );

    let mut table_cells: Vec<Vec<String>> = Vec::new();
    for (left, right_vec) in iter {
        let mut table_text: Vec<String> = Vec::new();
        table_text.push(left.to_string());
        for right in right_vec {
            table_text.push(right.to_string());
            if table_text.len() == 2 {
                table_cells.push(table_text);
                table_text = Vec::new();
            }
            table_text.push("".to_string());
        }
    }

    // table_cells[0];

    // let table_headers = &table_cells[0];
        // .drain(0..1)
        // .as_slice()[0];

    let table_headers = vec!["1".to_string(), "2".to_string()];

    DefinitionTable {
        header: table_headers,
        definitions: table_cells,
    }
}