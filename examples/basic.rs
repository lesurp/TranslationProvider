use translation_provider::generate_translation;
use serde_json;

generate_translation! {
    january,
    format_money(whole: i32, decimal: i32),
    february,
}

const SOME_TRANSLATOR_PROVIDED_DATA: &str = 
r#"
{
    "january": "janvier",
    "february": "février",
    "format_money": "{whole},{decimal}€"
}
"#;

fn main() {
    let ts : TranslationProvider = serde_json::from_str(SOME_TRANSLATOR_PROVIDED_DATA).unwrap();

    println!("{}", ts.january().unwrap());
    println!("{}", ts.february().unwrap());
    println!("{}", ts.format_money(3, 14).unwrap());

    println!("The generated code was:\n{}", TranslationProvider::generated_code());


}
