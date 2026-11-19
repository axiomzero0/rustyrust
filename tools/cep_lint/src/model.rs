// CEP:FILE: tools/cep_lint/src/model.rs
// CEP:WHAT: Core data model for rules, findings, configuration, and bootstrap failures.
// CEP:WHY: All checks share one finding type and one deterministic ordering; the engine contains mechanism only, so the model must be data-shaped.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: none; pure data structures with no fallible operations.
// CEP:ASSUMES: Finding line zero denotes a file-level (not line-scoped) finding; documented in the engine report contract.
// CEP:COST: cold; allocation cost is bounded by repository size and is acceptable for an offline tool.
// CEP:EVIDENCE: unit test cep_lint::model::tests::ordering_is_deterministic.

/// A single lint finding, ordered deterministically by file, line, rule, then message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    /// Severity class zero through three of section 34.2.
    pub severity: u8,
    /// Identifier of the rule that produced the finding.
    pub rule: String,
    /// Path of the scanned file, relative to the working directory.
    pub file: String,
    /// One-based source line, or zero for file-level findings.
    pub line: usize,
    /// Rendered message from the configured template.
    pub message: String,
}

/// A configured lint rule; every pattern, threshold, severity, and message lives here, not in code.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Catalog identifier such as CEP-LINT-FILE-HEADER.
    pub id: String,
    /// Check primitive name selected by the engine.
    pub check: String,
    /// Optional mode within the check primitive.
    pub mode: Option<String>,
    /// Severity zero through three.
    pub severity: u8,
    /// Message template with detail placeholders.
    pub message: String,
    /// Parameters object with patterns, thresholds, and lists.
    pub params: crate::json::Value,
}

/// Top-level lint configuration loaded from `.cep/cep_lint.json` and its includes.
#[derive(Debug, Clone)]
pub struct Config {
    /// Standard name from configuration.
    pub standard_name: String,
    /// Standard version that must match the document version line.
    pub standard_version: String,
    /// Glob patterns selecting first-party files.
    pub file_globs: Vec<String>,
    /// Glob patterns excluded from scanning.
    pub exclude_paths: Vec<String>,
    /// Severities that force exit code one.
    pub fail_on: Vec<u8>,
    /// Exit code for the clean case.
    pub exit_clean: i32,
    /// Exit code for blocking findings.
    pub exit_findings: i32,
    /// Exit code for bootstrap failures.
    pub exit_bootstrap: i32,
    /// Exit code for usage errors.
    pub exit_usage: i32,
    /// Configured rules in include order.
    pub rules: Vec<Rule>,
}

/// A bootstrap failure before or during configuration, scan, or document loading.
#[derive(Debug, Clone)]
pub struct BootstrapError {
    /// Coded failure class one through eight of section 50.4.
    pub code: u8,
    /// Human-readable detail string.
    pub detail: String,
}

impl BootstrapError {
    /// Builds a bootstrap error with the given code from section 50.4.
    pub fn new(code: u8, detail: impl Into<String>) -> Self {
        BootstrapError { code, detail: detail.into() }
    }

    /// Formats the error as the mandated `CEP-LINT-BOOT-<n>` string.
    pub fn format(&self) -> String {
        format!("CEP-LINT-BOOT-{}: {}", self.code, self.detail)
    }
}

/// CEP:WHAT: Expands placeholder occurrences in a message template with substitution values.
/// CEP:WHY: Message templates are configuration data; substitution keeps engine code mechanism-only.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Templates use brace-delimited keys; keys absent from values are left intact.
/// CEP:COST: cold; linear in template length.
/// CEP:EVIDENCE: report format checked by cep_lint self-lint.
pub fn substitute(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (key, val) in values {
        let pat = format!("{{{}}}", key);
        out = out.replace(&pat, val);
    }
    out
}

/// CEP:WHAT: Sorts findings deterministically by file, line, rule, then message.
/// CEP:WHY: Section 50.1 mandates sorted, locale-independent output.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: O(n log n) comparisons.
/// CEP:EVIDENCE: unit test cep_lint::model::tests::ordering_is_deterministic.
pub fn sort_findings(findings: &mut [Finding]) {
    findings.sort_by(|a, b| {
        (&a.file, a.line, &a.rule, &a.message).cmp(&(&b.file, b.line, &b.rule, &b.message))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sample line number used by the ordering test.
    const SAMPLE_LINE: usize = 3;

    /// Sample index used by the ordering test.
    const SAMPLE_INDEX: usize = 2;

/// CEP:WHAT: Verifies that findings sort by file, then line, then rule, then message.
/// CEP:WHY: Deterministic ordering is a contract feature of section 50.1.
/// CEP:STATUS: complete
/// CEP:FAILURE: Fails the test binary on regression.
/// CEP:ASSUMES: none
/// CEP:COST: cold; unit test only.
/// CEP:EVIDENCE: cargo test -p cep_lint.
    #[test]
    fn ordering_is_deterministic() {
        let mut fs = vec![
            Finding { severity: 1, rule: "B".into(), file: "a".into(), line: SAMPLE_LINE, message: "m".into() },
            Finding { severity: 1, rule: "A".into(), file: "a".into(), line: SAMPLE_LINE, message: "m".into() },
            Finding { severity: 1, rule: "Z".into(), file: "a".into(), line: 1, message: "m".into() },
        ];
        sort_findings(&mut fs);
        assert_eq!(fs[0].line, 1);
        assert_eq!(fs[1].rule, "A");
        assert_eq!(fs[SAMPLE_INDEX].rule, "B");
    }
}
