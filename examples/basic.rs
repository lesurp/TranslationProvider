use translation_provider::generate_translation;

generate_translation! {
    january,
    format_money(whole: i32, decimal: i32),
    february,
}

fn main() {
    let ts = TranslationProvider {
        january: "janvier".to_owned(),
        february: "février".to_owned(),
        format_money: "{whole},{decimal}€".to_owned(),
    };

    println!("{}", ts.january_().unwrap());
    println!("{}", ts.february_().unwrap());
    println!("{}", ts.format_money_(3, 14).unwrap());
}
