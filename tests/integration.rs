//! Integration tests for codequality-rs.

#[test]
fn test_analyze_long_function() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("long.py");
    let mut body = String::from("def long_function():\n");
    for _ in 0..55 {
        body.push_str("    a = 1\n");
    }
    body.push_str("    return 1\n");
    std::fs::write(&file, body).unwrap();

    let findings = codequality_rs::analyzer::analyze_file(&file).unwrap();
    let long: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_id == "function-length")
        .collect();
    assert!(!long.is_empty(), "Should detect long function");
}

#[test]
fn test_analyze_too_many_params() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("params.py");
    std::fs::write(
        &file,
        "def too_many_params(a, b, c, d, e, f, g):\n    return a\n",
    )
    .unwrap();

    let findings = codequality_rs::analyzer::analyze_file(&file).unwrap();
    let param: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_id == "function-params")
        .collect();
    assert!(!param.is_empty());
    assert!(param[0].message.contains("7"));
}

#[test]
fn test_analyze_simple_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("clean.py");
    std::fs::write(
        &file,
        "def clean(x, y):\n    return x + y\n",
    )
    .unwrap();

    let findings = codequality_rs::analyzer::analyze_file(&file).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn test_render_markdown() {
    use codequality_rs::analyzer::{Finding, Severity};

    let findings = vec![Finding {
        rule_id: "function-length".to_string(),
        rule_name: "Function too long".to_string(),
        severity: Severity::Warning,
        file: "test.py".to_string(),
        line: 10,
        column: 0,
        message: "Function 'foo' is 60 lines long".to_string(),
        suggestion: None,
    }];

    let md = codequality_rs::reporters::render_markdown(&findings, "test.py");
    assert!(md.contains("# `test.py`"));
    assert!(md.contains("function-length"));
    assert!(md.contains("Function 'foo'"));
    assert!(md.contains("🟡"));
}

#[test]
fn test_render_json() {
    use codequality_rs::analyzer::{Finding, Severity};

    let findings = vec![Finding {
        rule_id: "complexity".to_string(),
        rule_name: "Test".to_string(),
        severity: Severity::Info,
        file: "x.py".to_string(),
        line: 1,
        column: 0,
        message: "Test message".to_string(),
        suggestion: None,
    }];

    let json = codequality_rs::reporters::render_json(&findings).unwrap();
    assert!(json.contains("complexity"));
    assert!(json.contains("info"));
}

#[test]
fn test_render_sarif() {
    use codequality_rs::analyzer::{Finding, Severity};

    let findings = vec![Finding {
        rule_id: "nested-depth".to_string(),
        rule_name: "Test".to_string(),
        severity: Severity::Error,
        file: "x.py".to_string(),
        line: 42,
        column: 5,
        message: "Test SARIF".to_string(),
        suggestion: None,
    }];

    let sarif = codequality_rs::reporters::render_sarif(&findings, "x.py").unwrap();
    assert!(sarif.contains("sarif-2.1.0"));
    assert!(sarif.contains("nested-depth"));
    assert!(sarif.contains("error"));
}

#[test]
fn test_no_findings_clean_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("clean.py");
    std::fs::write(
        &file,
        r#"
def hello(name: str) -> str:
    return f"Hi {name}"
"#,
    )
    .unwrap();

    let findings = codequality_rs::analyzer::analyze_file(&file).unwrap();
    assert!(findings.is_empty(), "Expected no findings, got: {:?}", findings);
}
