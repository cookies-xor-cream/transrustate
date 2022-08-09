pub mod wordreference_utils {
    const BASE_URL: &str = "https://www.wordreference.com";

    pub fn definition_url(from_language: String, to_language: String, word: String) -> String {
        let definition_postfix = format!(
            "/{}{}/",
            map_language(from_language),
            map_language(to_language)
        );
        let definition_url = format!("{}{}{}", BASE_URL, definition_postfix, word);
        definition_url
    }

    pub fn conjugation_url(language: String, verb: String) -> String {
        let conjugation_postfix = format!("/conj/{}verbs.aspx?v=", map_language(language));
        let conjugation_url = format!("{}{}{}", BASE_URL, conjugation_postfix, verb);
        conjugation_url
    }

    pub fn map_language(language: String) -> String {
        match language.as_str() {
            "french" => "fr".to_string(),
            "italian" => "it".to_string(),
            "english" => "en".to_string(),
            "spanish" => "es".to_string(),
            _ => "".to_string(), // is an error state
        }
    }


}
