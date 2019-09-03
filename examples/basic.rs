#![feature(proc_macro_hygiene)]

use translation_provider::{generate_translation, create_provider_index};
use serde_json;

generate_translation! {
    id = id,
    january,
    display = display,
    format_money(whole: i32, decimal: i32),
    february,
}

const SOME_TRANSLATOR_PROVIDED_DATA: &str = 
r#"
{
    "id": "fr_FR",
    "january": "janvier",
    "february": "février",
    "display": "français",
    "format_money": "{whole},{decimal}€"
}
"#;

const SOME_TRANSLATOR_PROVIDED_DATA_EN: &str = 
r#"
{
    "id": "en_GB",
    "january": "January",
    "february": "February",
    "display": "English",
    "format_money": "£{whole}.{decimal}"
}
"#;

fn main() {
    let ts : TranslationProvider = serde_json::from_str(SOME_TRANSLATOR_PROVIDED_DATA).unwrap();

    println!("{}", ts.january());
    println!("{}", ts.february());
    println!("{}", ts.format_money(3, 14).unwrap());

    println!("The generated code was:\n{}", TranslationProvider::generated_code());


    let ts2 : TranslationProvider = serde_json::from_str(SOME_TRANSLATOR_PROVIDED_DATA_EN).unwrap();

    let allofthem = vec![ts, ts2];
    let indexes = create_provider_index!(allofthem);
    for (id, display) in indexes {
        println!("Supported language: {} ({})", display, id);        
    }
}
