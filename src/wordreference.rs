pub mod wordreference_utils {
    const BASE_URL: &str = "https://www.wordreference.com"; // /conj/frverbs.aspx?v=";

    pub fn conjugation_url(language: String, verb: String) -> String {
        let conjugation_postfix = format!("/conj/{}verbs.aspx?v=", map_language(language));
        let conjugation_url = format!("{}{}{}", BASE_URL, conjugation_postfix, verb);
        conjugation_url
    }

    pub fn map_language(language: String) -> String {
        match language.as_str() {
            "french" => "fr".to_string(),
            "italian" => "it".to_string(),
            _ => "fr".to_string(), // default to french
        }
    }
}
