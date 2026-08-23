use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

fn run(arguments: &[&str], stdin: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_swissmath"));
    command
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if stdin.is_some() {
        command.stdin(Stdio::piped());
    }
    let mut child = command.spawn().unwrap();
    if let Some(input) = stdin {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    child.wait_with_output().unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn temp_csv(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "swissmath-{name}-{}-{unique}.csv",
        std::process::id()
    ))
}

#[test]
fn human_prime_factor_analyze_and_research_primitive_are_compact() {
    let prime = run(&["prime", "97"], None);
    assert!(prime.status.success());
    assert_eq!(stdout(&prime).trim(), "Prime — exact");

    let factor = run(&["factor", "360"], None);
    assert!(factor.status.success());
    assert_eq!(stdout(&factor).trim(), "360 = 2^3 * 3^2 * 5");

    let analysis = run(&["analyze", "360"], None);
    assert!(analysis.status.success());
    assert!(stdout(&analysis).contains("phi=96"));
    assert!(stdout(&analysis).contains("tau=24"));

    let mobius = run(&["mobius", "30"], None);
    assert!(mobius.status.success());
    assert_eq!(stdout(&mobius).trim(), "-1");
}

#[test]
fn reconstruction_and_json_are_structured() {
    let reconstruction = run(&["reconstruct", "7", "101", "--json"], None);
    assert!(reconstruction.status.success());
    let value: Value = serde_json::from_slice(&reconstruction.stdout).unwrap();
    assert_eq!(value["status"], "ok");
    assert_eq!(value["operation"], "reconstruct");
    assert_eq!(value["result"]["numerator"], "7");
    assert_eq!(value["result"]["denominator"], "1");
    assert_eq!(value["exactness"], "exact");
    assert_eq!(value["core_version"], "0.6");
    assert!(value["elapsed_ms"].is_number());
}

#[test]
fn jsonl_stream_continues_after_a_malformed_record() {
    let output = run(&["prime", "--jsonl"], Some("97\nabc\n101\n"));
    assert!(output.status.success());
    let records = stdout(&output)
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0]["status"], "ok");
    assert_eq!(records[1]["status"], "error");
    assert_eq!(records[1]["error"]["code"], "invalid_input");
    assert_eq!(records[2]["status"], "ok");
}

#[test]
fn csv_preserves_columns_and_appends_results() {
    let input = temp_csv("input");
    let output = temp_csv("output");
    fs::write(&input, "name,n\n\"Alice, A\",97\nBob,abc\n").unwrap();
    let process = run(
        &[
            "prime",
            "--input",
            input.to_str().unwrap(),
            "--column",
            "n",
            "--output",
            output.to_str().unwrap(),
        ],
        None,
    );
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    let csv = fs::read_to_string(&output).unwrap();
    assert!(csv.starts_with(
        "name,n,swissmath_status,swissmath_result,swissmath_exactness,swissmath_error\n"
    ));
    assert!(csv.contains("\"Alice, A\",97,ok"));
    assert!(csv.contains("Bob,abc,error"));
    fs::remove_file(input).unwrap();
    fs::remove_file(output).unwrap();
}
