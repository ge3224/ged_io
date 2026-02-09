use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn write_temp_gedcom(contents: &str) -> PathBuf {
    let mut path = env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let filename = format!("ged_io_cli_test_{}_{}.ged", std::process::id(), nanos);
    path.push(filename);
    fs::write(&path, contents).expect("write temp gedcom");
    path
}

fn run_cli(args: &[&str]) -> std::process::Output {
    let exe = env!("CARGO_BIN_EXE_ged_io");
    Command::new(exe)
        .args(args)
        .output()
        .expect("run ged_io binary")
}

#[test]
fn validate_lenient_outputs_report_only() {
    let sample = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
    let path = write_temp_gedcom(sample);

    let output = run_cli(&["--validate", path.to_str().unwrap()]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Validation: lenient - errors: 0, warnings: 0"));
    assert!(!stdout.contains("GEDCOM Data Stats"));
}

#[test]
fn validate_strict_reports_errors() {
    let sample = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @F1@ FAM\n1 HUSB @I999@\n0 TRLR";
    let path = write_temp_gedcom(sample);

    let output = run_cli(&[
        "--validate",
        "--validation-level",
        "strict",
        path.to_str().unwrap(),
    ]);

    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Validation: strict - errors: 1, warnings: 0"));
    assert!(stdout.contains("Family references non-existent individual"));
}

#[test]
fn validation_level_requires_validate_flag() {
    let sample = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
    let path = write_temp_gedcom(sample);

    let output = run_cli(&["--validation-level", "strict", path.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("requires --validate"));
}
