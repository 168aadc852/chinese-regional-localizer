use chinese_regional_localizer::LocalizerEngine;
use serde_json::to_string_pretty;
use std::env;
use std::process::ExitCode;

fn value_after(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].clone())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let Some(db) = value_after(&args, "--db") else {
        eprintln!("missing --db");
        return ExitCode::from(2);
    };
    let Some(source) = value_after(&args, "--from") else {
        eprintln!("missing --from");
        return ExitCode::from(2);
    };
    let Some(target) = value_after(&args, "--to") else {
        eprintln!("missing --to");
        return ExitCode::from(2);
    };
    let Some(text) = value_after(&args, "--text") else {
        eprintln!("missing --text");
        return ExitCode::from(2);
    };

    let engine = match LocalizerEngine::open(&db) {
        Ok(engine) => engine,
        Err(error) => {
            eprintln!("failed to open database: {error:?}");
            return ExitCode::from(1);
        }
    };
    let result = match engine.localize(&text, &source, &target, None) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("localization failed: {error:?}");
            return ExitCode::from(1);
        }
    };
    println!("{}", to_string_pretty(&result).expect("serialize result"));
    ExitCode::SUCCESS
}
