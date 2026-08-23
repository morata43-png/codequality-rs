//! Code quality analyzer using tree-sitter AST.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tree_sitter::Node;

/// A single code quality finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub rule_id: String,
    pub rule_name: String,
    pub severity: Severity,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }
}

/// Analyze a Python file and return findings.
pub fn analyze_file(file_path: &Path) -> Result<Vec<Finding>> {
    let source = std::fs::read_to_string(file_path)?;
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_python::language())
        .map_err(|e| anyhow::anyhow!("Failed to set language: {}", e))?;

    let tree = parser
        .parse(&source, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse file"))?;

    let mut findings = Vec::new();

    // File-level checks
    let line_count = source.lines().count();
    if line_count > 500 {
        findings.push(Finding {
            rule_id: "file-length".to_string(),
            rule_name: "File too long".to_string(),
            severity: Severity::Warning,
            file: file_path.display().to_string(),
            line: 1,
            column: 0,
            message: format!("File is {} lines long (>500)", line_count),
            suggestion: Some("Consider splitting the file into smaller modules".to_string()),
        });
    }

    // AST-level checks
    walk_tree(tree.root_node(), &source, file_path, &mut findings);

    Ok(findings)
}

fn walk_tree(node: Node, source: &str, file_path: &Path, findings: &mut Vec<Finding>) {
    if !is_function(&node) {
        // Recurse only into classes/blocks to find nested functions
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                walk_tree(cursor.node(), source, file_path, findings);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        return;
    }

    // Run all checks on this function
    let func_findings = check_function(&node, source, file_path);
    findings.extend(func_findings);

    // Recurse for nested functions
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            walk_tree(cursor.node(), source, file_path, findings);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

fn check_function(
    node: &Node,
    source: &str,
    file_path: &Path,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let name = get_function_name(node, source);
    let line = node.start_position().row + 1;
    let column = node.start_position().column as u32;
    let file = file_path.display().to_string();

    // R1: function length
    let start_row = node.start_position().row;
    let end_row = node.end_position().row;
    let func_length = end_row.saturating_sub(start_row) + 1;
    if func_length > 50 {
        findings.push(Finding {
            rule_id: "function-length".to_string(),
            rule_name: "Function too long".to_string(),
            severity: Severity::Warning,
            file: file_path.display().to_string(),
            line: line as u32,
            column,
            message: format!(
                "Function '{}' is {} lines long (>50)",
                name, func_length
            ),
            suggestion: Some("Consider breaking it into smaller functions".to_string()),
        });
    }

    // R3: parameter count
    let params_count = count_parameters(node);
    if params_count > 5 {
        findings.push(Finding {
            rule_id: "function-params".to_string(),
            rule_name: "Too many parameters".to_string(),
            severity: Severity::Info,
            file: file_path.display().to_string(),
            line: line as u32,
            column,
            message: format!(
                "Function '{}' has {} parameters (>5)",
                name, params_count
            ),
            suggestion: Some("Consider grouping related params into a dataclass or dict".to_string()),
        });
    }

    // R4: cyclomatic complexity
    let complexity = calculate_complexity(node);
    if complexity > 10 {
        findings.push(Finding {
            rule_id: "complexity".to_string(),
            rule_name: "High cyclomatic complexity".to_string(),
            severity: Severity::Warning,
            file: file_path.display().to_string(),
            line: line as u32,
            column,
            message: format!("Cyclomatic complexity is {} (>10)", complexity),
            suggestion: Some("Extract helper functions to reduce branches".to_string()),
        });
    }

    // R5: nesting depth
    let max_depth = calculate_max_depth(node, 0);
    if max_depth > 4 {
        findings.push(Finding {
            rule_id: "nested-depth".to_string(),
            rule_name: "Deep nesting".to_string(),
            severity: Severity::Warning,
            file: file_path.display().to_string(),
            line: line as u32,
            column,
            message: format!("Nesting depth is {} (>4)", max_depth),
            suggestion: Some("Use early returns to flatten the control flow".to_string()),
        });
    }

    // R6: return count
    let returns = count_returns(node);
    if returns > 5 {
        findings.push(Finding {
            rule_id: "returns".to_string(),
            rule_name: "Too many return statements".to_string(),
            severity: Severity::Info,
            file: file_path.display().to_string(),
            line: line as u32,
            column,
            message: format!("{} return statements (>5)", returns),
            suggestion: Some("Consider using a single exit point".to_string()),
        });
    }

    findings
}

fn is_function(node: &Node) -> bool {
    matches!(node.kind(), "function_definition" | "async_function_definition")
}

fn get_function_name(node: &Node, source: &str) -> String {
    node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("<unknown>")
        .to_string()
}

fn count_parameters(node: &Node) -> u32 {
    let Some(params) = node.child_by_field_name("parameters") else {
        return 0;
    };
    let mut count = 0;
    let mut cursor = params.walk();
    for child in params.children(&mut cursor) {
        if matches!(
            child.kind(),
            "identifier" | "typed_parameter" | "default_parameter" | "list_splat_pattern" | "dictionary_splat_pattern"
        ) {
            count += 1;
        }
    }
    count
}

fn calculate_complexity(node: &Node) -> u32 {
    let mut complexity = 1;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "if_statement" | "elif_clause" | "if_clause" => {
                complexity += 1;
            }
            "for_statement" | "while_statement" => {
                complexity += 1;
            }
            "try_statement" | "except_clause" => {
                complexity += 1;
            }
            "boolean_operator" => {
                complexity += 1;
            }
            "case_match" | "case_clause" => {
                complexity += 1;
            }
            _ => {}
        }
    }
    complexity
}

fn calculate_max_depth(node: &Node, current_depth: u32) -> u32 {
    let mut max_depth = current_depth;
    let mut cursor = node.walk();
    let kind = node.kind();
    let is_control_flow = matches!(
        kind,
        "if_statement"
            | "for_statement"
            | "while_statement"
            | "try_statement"
            | "with_statement"
            | "match_statement"
    );
    let new_depth = if is_control_flow { current_depth + 1 } else { current_depth };
    if new_depth > max_depth {
        max_depth = new_depth;
    }
    for child in node.children(&mut cursor) {
        let child_depth = calculate_max_depth(&child, new_depth);
        if child_depth > max_depth {
            max_depth = child_depth;
        }
    }
    max_depth
}

fn count_returns(node: &Node) -> u32 {
    let mut count = 0;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "return_statement" {
            count += 1;
        }
    }
    count
}
