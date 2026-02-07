use std::fs;
use std::process::Command;

#[test]
fn test_cli_basic_qr_generation() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--url",
            "https://example.com",
            "--out",
            "test_qr.png",
        ])
        .output()
        .expect("Failed to execute command");

    // Clean up test file
    let _ = fs::remove_file("test_qr.png");

    assert!(output.status.success(), "Command failed: {:?}", output);
}

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Help command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("qrbrand"),
        "Help output should contain 'qrbrand'"
    );
    assert!(
        stdout.contains("--url"),
        "Help output should contain '--url' option"
    );
}

#[test]
fn test_cli_invalid_url() {
    let output = Command::new("cargo")
        .args(["run", "--", "--url", "not-a-valid-url"])
        .output()
        .expect("Failed to execute command");

    // Should fail with invalid URL
    assert!(!output.status.success(), "Invalid URL should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid URL") || stderr.contains("error"),
        "Should show URL error"
    );
}
