//! Reporters: Markdown, JSON, and SARIF output.

use crate::analyzer::{Finding, Severity};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
struct SarifReport {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize)]
struct SarifDriver {
    name: &'static str,
    version: &'static str,
    information_uri: &'static str,
    rules: Vec<SarifRule>,
}

#[derive(Serialize)]
struct SarifRule {
    id: String,
    name: String,
    short_description: SarifText,
    full_description: SarifText,
    default_configuration: SarifConfig,
}

#[derive(Serialize)]
struct SarifText {
    text: String,
}

#[derive(Serialize)]
struct SarifConfig {
    level: String,
}

#[derive(Serialize)]
struct SarifResult {
    rule_id: String,
    level: String,
    message: SarifText,
    locations: Vec<SarifLocation>,
}

#[derive(Serialize)]
struct SarifLocation {
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
struct SarifPhysicalLocation {
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
struct SarifRegion {
    start_line: u32,
    start_column: u32,
}

pub fn render_markdown(findings: &[Finding], file: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("# `{}`\n\n", file));

    if findings.is_empty() {
        out.push_str("✅ No issues found.\n");
        return out;
    }

    let mut by_severity: BTreeMap<&str, Vec<&Finding>> = BTreeMap::new();
    for f in findings {
        by_severity
            .entry(f.severity.as_str())
            .or_default()
            .push(f);
    }

    for (sev, items) in by_severity.iter().rev() {
        out.push_str(&format!("## {} ({} issue{})\n\n", capitalize(sev), items.len(), if items.len() == 1 { "" } else { "s" }));
        for f in items {
            let icon = match f.severity {
                Severity::Error => "🔴",
                Severity::Warning => "🟡",
                Severity::Info => "🔵",
            };
            out.push_str(&format!(
                "- {} **L{}** `{}` ({}): {}\n",
                icon, f.line, f.rule_id, f.severity.as_str(), f.message
            ));
            if let Some(s) = &f.suggestion {
                out.push_str(&format!("  - 💡 {}\n", s));
            }
        }
        out.push('\n');
    }

    out
}

pub fn render_json(findings: &[Finding]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(findings)
}

pub fn render_sarif(findings: &[Finding], file: &str) -> Result<String, serde_json::Error> {
    let mut unique_rules: BTreeMap<String, &Finding> = BTreeMap::new();
    for f in findings {
        unique_rules.entry(f.rule_id.clone()).or_insert(f);
    }

    let rules: Vec<SarifRule> = unique_rules
        .values()
        .map(|f| SarifRule {
            id: f.rule_id.clone(),
            name: f.rule_name.clone(),
            short_description: SarifText {
                text: f.rule_name.clone(),
            },
            full_description: SarifText {
                text: format!("{}: {}", f.rule_id, f.rule_name),
            },
            default_configuration: SarifConfig {
                level: severity_to_sarif_level(&f.severity).to_string(),
            },
        })
        .collect();

    let results: Vec<SarifResult> = findings
        .iter()
        .map(|f| SarifResult {
            rule_id: f.rule_id.clone(),
            level: severity_to_sarif_level(&f.severity).to_string(),
            message: SarifText {
                text: format!("{}: {}", f.rule_id, f.message),
            },
            locations: vec![SarifLocation {
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: file.to_string(),
                    },
                    region: SarifRegion {
                        start_line: f.line,
                        start_column: f.column + 1,
                    },
                },
            }],
        })
        .collect();

    let report = SarifReport {
        schema: "https://json.schemastore.org/sarif-2.1.0.json",
        version: "2.1.0",
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "codequality-rs",
                    version: env!("CARGO_PKG_VERSION"),
                    information_uri: "https://github.com/morata43-png/codequality-rs",
                    rules,
                },
            },
            results,
        }],
    };

    serde_json::to_string_pretty(&report)
}

fn severity_to_sarif_level(sev: &Severity) -> &'static str {
    match sev {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "note",
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
