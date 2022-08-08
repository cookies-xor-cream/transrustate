
use reqwest::{self, Client};
use scraper::{Html, ElementRef};
use crate::{wordreference::wordreference_utils, user_error::UserError};

pub struct DefinitionTable {
    pub header: Vec<String>, // [1, 2]
    pub definitions: Vec<Vec<String>>, // [[1, 2], [3, 4]]
}

pub struct WordDefinitions {
    pub title: String,
    pub definitions: Vec<DefinitionTable>,
}

impl WordDefinitions {
    pub fn empty() -> Self {
        Self {
            title: "".to_string(),
            definitions: Vec::new(),
        }
    }

    fn extract_definitions_from_table(
        table: Html,
        to_language: String,
    ) -> Result<DefinitionTable, UserError> {
        let invalid_parsing_error = UserError {
            message: format!(
                "Translations to '{to_language}' could not be found for the word. \
                Please double check your spelling",
            )
        };

        let from_word_to_word_query = scraper::Selector::parse(
            "tr > td.FrWrd, tr > td.ToWrd"
        )
            .expect("td.FrWrd and td.ToWrd exist");

        let language_code = wordreference_utils::map_language(to_language);
        let selector_query = format!(
            "em[data-lang=\"{0}\"], span[data-ph=\"sLang_{0}\"]",
            language_code,
        );
        let from_query = scraper::Selector::parse(
            selector_query.as_str(),
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
                if to_words.len() != 0 {
                    let l = to_words.last_mut().unwrap();
                    l.push(b);
                } else {
                    return Err(invalid_parsing_error);
                }
            }
        }

        let translation = std::iter::zip(
            from_words.iter(),
            to_words.iter()
        );

        let mut table_cells: Vec<Vec<String>> = Vec::new();
        for (from_word, to_words) in translation {
            let mut table_text: Vec<String> = Vec::new();
            table_text.push(from_word.to_string());
            for to_word in to_words {
                table_text.push(to_word.to_string());
                if table_text.len() == 2 {
                    table_cells.push(table_text);
                    table_text = Vec::new();
                }
                table_text.push("".to_string());
            }
        }

        let definitions = table_cells.drain(1..).collect::<Vec<_>>();
        let header = table_cells
            .iter()
            .flatten()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();

        let definition_table = DefinitionTable {
            header,
            definitions,
        };

        Ok(definition_table)
    }

    fn extract_definitions_from_tables(
        tables: Vec<Html>,
        to_language: String,
    ) -> Vec<DefinitionTable> {
        let mut definitions: Vec<DefinitionTable> = Vec::new();
        for table in tables {
            let definition_table_result = WordDefinitions::extract_definitions_from_table(table, to_language.clone());

            if let Ok(definition_table) = definition_table_result {
                definitions.push(definition_table);
            } else if let Err(_err) = definition_table_result {
                return Vec::new();
            }
        }

        definitions
    }

    async fn scrape_definition_tables(
        word: String,
        from_language: String,
        to_language: String,
        client: &Client,
    ) -> Result<Vec<Html>, UserError> {
        let not_exist_error = UserError {
            message: format!(
                "The word '{word}' does not exist in the selected language \
                ({from_language}). Please double check your spelling",
            )
        };

        let network_error = UserError {
            message: "Could not find the corresponding definitions, \
            please check your network connection".to_string(),
        };

        let word_query_url = wordreference_utils::definition_url(
            from_language.clone(),
            to_language.clone(),
            word.clone(),
        );

        let response = match match client.get(
                word_query_url,
            )
            .send()
            .await {
                Ok(it) => it,
                Err(_err) => return Err(network_error),
            }
            .text()
            .await {
                Ok(it) => it,
                Err(_err) => return Err(not_exist_error),
            };

        let document = scraper::Html::parse_document(&response);
        let table_query = scraper::Selector::parse("table.WRD")
            .expect("word definitions should have a table with the WRD class");
        let tables = document
            .select(&table_query);

        let tables = tables
            .map(|x| scraper::Html::parse_fragment(&x.html()));

        let tables = tables.collect::<Vec<Html>>();

        Ok(tables)
    }

    pub async fn get_definition_tables(
        to_language: String,
        from_language: String,
        word: String,
        client: &Client,
    ) -> Result<WordDefinitions, UserError> {
        let not_exist_error = UserError {
            message: format!(
                "The word '{word}' does not exist in the selected language \
                ({from_language}). Please double check your spelling",
            )
        };

        let tables_result = WordDefinitions::scrape_definition_tables(
            word.clone(),
            from_language.clone(),
            to_language.clone(),
            client,
        ).await;

        match tables_result {
            Err(err) => {
                Err(err)
            }
            Ok(tables) => {
                let definitions = WordDefinitions::extract_definitions_from_tables(
                    tables,
                    to_language.clone(),
                );

                if definitions.len() == 0 {
                    return Err(not_exist_error);
                }

                let title = format!("Translate '{word}' to {to_language}");
                let word_definitions = WordDefinitions {
                    title,
                    definitions,
                };

                Ok(word_definitions)
            }
        }
    }
}
