#![cfg(feature = "cli")]

use std::process::Command;

#[test]
fn test_newline_terminated_patterns() {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("chroot-not-found.log");
    std::fs::write(
        &path,
        "[schroot -c foo] failed (exit status 1, stderr 'E: foo: Chroot not found\\n')\n\
         autopkgtest [22:52:19]: ERROR: testbed failure: cannot send to testbed: [Errno 32] Broken pipe\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_analyze-autopkgtest-log"))
        .arg("--json")
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success());
    // The JSON document is followed by the highlighted lines.
    let result: serde_json::Value = serde_json::Deserializer::from_slice(&output.stdout)
        .into_iter()
        .next()
        .unwrap()
        .unwrap();
    assert_eq!(
        result,
        serde_json::json!({
            "testname": null,
            "error": {"kind": "chroot-not-found", "details": {"chroot": "foo"}},
            "description": "[schroot -c foo] failed (exit status 1, stderr 'E: foo: Chroot not found\\n')\n",
            "offset": 0,
        })
    );
}
