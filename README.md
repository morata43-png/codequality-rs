# codequality-rs

> Fast code quality analyzer for Python. Built in Rust, no AI, no API costs.

A blazingly fast static code analyzer for Python, written in Rust. Inspired by the Python `codequality-cli` but with **10-100x better performance** thanks to tree-sitter AST parsing.

## Why codequality-rs?

- ✅ **Fast** — 10-100x faster than the Python equivalent (tree-sitter Rust vs Python AST)
- ✅ **No API costs** — no LLM calls, no cloud, no OpenAI key
- ✅ **Standard output** — supports Markdown, JSON, and SARIF (GitHub Security standard)
- ✅ **Smart** — only checks files that match your extensions, respects `.gitignore`
- ✅ **Cross-platform** — single static binary for Linux, macOS, Windows

## Installation

### From source
```bash
git clone https://github.com/morata43-png/codequality-rs
cd codequality-rs
cargo install --path .
```

### From prebuilt binary (coming soon)
```bash
# macOS/Linux
curl -L https://github.com/morata43-png/codequality-rs/releases/latest/download/codequality-$(uname -s)-$(uname -m) -o /usr/local/bin/codequality
chmod +x /usr/local/bin/codequality
```

## Usage

```bash
# Scan current directory, markdown output
codequality

# Scan specific path
codequality ./src

# Output as JSON
codequality ./src --format json

# Output as SARIF (GitHub Security standard)
codequality ./src --format sarif

# Only run specific rules
codequality ./src --only function-length,complexity

# Skip certain rules
codequality ./src --skip nested-depth
```

## Rules

| Rule | ID | Default threshold | Severity |
|---|---|---|---|
| Function too long | `function-length` | > 50 lines | warning |
| File too long | `file-length` | > 500 lines | warning |
| Too many parameters | `function-params` | > 5 params | info |
| High cyclomatic complexity | `complexity` | > 10 | warning |
| Deep nesting | `nested-depth` | > 4 levels | warning |
| Too many return statements | `returns` | > 5 returns | info |

## Output formats

### Markdown (default, human-readable)
```markdown
# `path/to/file.py`

## Warning (1 issue)
- 🟡 L42 `function-length` (warning): Function 'foo' is 60 lines long (>50)
  - 💡 Consider breaking it into smaller functions

## Info (1 issue)
- 🔵 L45 `function-params` (info): Function 'foo' has 7 parameters (>5)
```

### JSON (machine-readable)
```json
[
  {
    "rule_id": "function-length",
    "rule_name": "Function too long",
    "severity": "warning",
    "file": "path/to/file.py",
    "line": 42,
    "column": 0,
    "message": "Function 'foo' is 60 lines long (>50)",
    "suggestion": "Consider breaking it into smaller functions"
  }
]
```

### SARIF (GitHub Security)
Standard SARIF 2.1.0 format. Works with GitHub Code Scanning, IDE integrations, etc.

## Pricing

| Tier | Price | Features |
|------|-------|----------|
| **Free** | $0/forever | All features, single machine, unlimited files |
| **Sponsor** | GitHub Sponsors | Support development, get priority issues |

This is an open-source project. The CLI itself is free forever.

## License

MIT
