use chinese_regional_localizer::{Runtime, RuntimeRequest, RUNTIME_API_VERSION};
use serde::Deserialize;
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct Case {
    input: String,
}

fn value_after(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].clone())
}

fn percentile(mut values: Vec<f64>, percentile: f64) -> f64 {
    values.sort_by(|a, b| a.total_cmp(b));
    if values.is_empty() {
        return 0.0;
    }
    let index = ((values.len() - 1) as f64 * percentile).round() as usize;
    values[index]
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let db = value_after(&args, "--db").unwrap_or_else(|| "build/regional-demo.sqlite".into());
    let corpus = value_after(&args, "--corpus")
        .unwrap_or_else(|| "data/fixtures/realistic_corpus.json".into());
    let target_chars = value_after(&args, "--target-chars")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(5000);
    let iterations = value_after(&args, "--iterations")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(5);

    let raw = fs::read_to_string(PathBuf::from(corpus)).expect("read corpus");
    let cases: Vec<Case> = serde_json::from_str(&raw).expect("parse corpus");
    let seed = cases
        .iter()
        .map(|case| case.input.as_str())
        .collect::<Vec<_>>()
        .join("。\n");
    assert!(!seed.is_empty(), "corpus must not be empty");

    let mut text = String::new();
    while text.chars().count() < target_chars {
        text.push_str(&seed);
        text.push('\n');
    }

    let runtime = Runtime::new(db, Option::<&str>::None);
    let request = RuntimeRequest {
        api_version: RUNTIME_API_VERSION.to_owned(),
        text: text.clone(),
        source_locale: "zh-CN".into(),
        target_locale: "zh-TW".into(),
        context: None,
    };

    let warmup = runtime.localize(&request).expect("warmup");
    let expected = warmup.output;
    let input_chars = text.chars().count();
    let mut latencies_ms = Vec::with_capacity(iterations);
    let mut deterministic = true;

    for _ in 0..iterations {
        let started = Instant::now();
        let result = runtime.localize(&request).expect("benchmark localize");
        let elapsed = started.elapsed().as_secs_f64() * 1000.0;
        latencies_ms.push(elapsed);
        deterministic &= result.output == expected;
    }

    let mean_ms = latencies_ms.iter().sum::<f64>() / latencies_ms.len() as f64;
    let chars_per_sec = input_chars as f64 / (mean_ms / 1000.0);
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "runtime_api_version": RUNTIME_API_VERSION,
            "source_locale": "zh-CN",
            "target_locale": "zh-TW",
            "input_chars": input_chars,
            "iterations": iterations,
            "deterministic": deterministic,
            "chars_per_sec": chars_per_sec,
            "latency_ms": {
                "mean": mean_ms,
                "p50": percentile(latencies_ms.clone(), 0.50),
                "p95": percentile(latencies_ms.clone(), 0.95),
                "max": latencies_ms.iter().copied().fold(0.0_f64, f64::max),
            }
        }))
        .expect("serialize benchmark")
    );
}
