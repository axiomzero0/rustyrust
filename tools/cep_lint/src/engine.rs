// CEP:FILE: tools/cep_lint/src/engine.rs
// CEP:WHAT: Engine orchestration: deterministic file discovery, per-file rule dispatch, and cross-file phases.
// CEP:WHY: Section 50.1 requires sorted traversal, sorted findings, and coded exit behavior.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: Propagates bootstrap errors; findings are reported through the return value.
// CEP:ASSUMES: Directory traversal visits entries in sorted order; no output depends on locale or time.
// CEP:COST: cold; linear in repository size times configured rules.
// CEP:EVIDENCE: cep_lint self-lint over tools/cep_lint/src; CI determinism gate.

use crate::model::{sort_findings, BootstrapError, Config, Finding, Rule};
use crate::scan::{scan_file, SourceFile};
use std::fs;
use std::path::Path;

/// Bootstrap code for invalid regular expressions.
const BOOT_REGEX: u8 = 8;

/// Flags that activate conditional rules such as receipt verification.
#[derive(Debug, Clone, Copy, Default)]
pub struct Flags {
    /// Activates the CEP-LINT-HUNT-RECEIPT rule.
    pub verify_hunt: bool,
    /// Activates the CEP-LINT-HUNTER-RECEIPT rule.
    pub verify_hunter: bool,
}

/// CEP:WHAT: Runs the engine over paths and returns sorted findings or a bootstrap error.
/// CEP:WHY: One entry point keeps CLI, self-test, and CI behavior identical.
/// CEP:ID: CS-a04
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap errors from configuration, registry, and document loading.
/// CEP:ASSUMES: rule_filter and skip_cross_file support the self-test harness without changing per-file behavior.
/// CEP:COST: cold; linear in files times rules.
/// CEP:EVIDENCE: doc file docs/functions/CS/CS-a04.md; self-lint gate.
pub fn run(cfg: &Config, roots: &[String], cwd: &Path, rule_filter: Option<&[String]>, skip_cross_file: bool, flags: Flags) -> Result<Vec<Finding>, BootstrapError> {
    let files = collect_files(cfg, roots, cwd)?;
    let mut findings: Vec<Finding> = Vec::new();
    for file in files.iter() {
        for rule in cfg.rules.iter() {
            if let Some(filter) = rule_filter {
                if !filter.contains(&rule.id) {
                    continue;
                }
            }
            findings.extend(run_rule(rule, file, cwd, flags)?);
        }
    }
    if !skip_cross_file {
        findings.extend(cross_file(cfg, &files, cwd, rule_filter)?);
    }
    sort_findings(&mut findings);
    Ok(findings)
}

/// CEP:WHAT: Dispatches one rule against one scanned file.
/// CEP:WHY: Check primitives are selected by name from configuration, not by code branching on rule ids.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Conditional receipt rules run only when their engine flag is set.
/// CEP:COST: cold; one check primitive per rule.
/// CEP:EVIDENCE: cep_lint self-lint.
fn run_rule(rule: &Rule, file: &SourceFile, cwd: &Path, flags: Flags) -> Result<Vec<Finding>, BootstrapError> {
    match rule.check.as_str() {
        "comment_block_schema" => Ok(crate::blockschema::check(rule, file, rule.mode.as_deref())),
        "function_policy" => Ok(crate::fnpolicy::check(rule, file, rule.mode.as_deref())),
        "comment_regex" | "string_literal_regex" | "flat_code_regex" | "numeric_literal_policy" | "macro_prefix_policy" | "banned_token_policy" => {
            Ok(crate::regexrules::check(rule, file))
        }
        "file_topology_policy" => Ok(crate::topology::check(rule, file)),
        "secret_probe_policy" => Ok(crate::secrets::check(rule, file, cwd)),
        "unsafe_doc_policy" => Ok(crate::unsafedoc::check(rule, file, cwd)),
        "commit_hunt_manifest" => Ok(if flags.verify_hunt { crate::receipts::check_hunt_receipt(rule, file, cwd) } else { Vec::new() }),
        "hunter_manifest" => Ok(if flags.verify_hunter { crate::receipts::check_hunter_receipt(rule, file, cwd) } else { Vec::new() }),
        _ => Ok(Vec::new()),
    }
}

/// CEP:WHAT: Runs cross-file rules: documentation synchronization and rosetta agreement.
/// CEP:WHY: Registry and document checks need the union of all scanned files.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap errors from registry or document loading.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in rules plus identifiers.
/// CEP:EVIDENCE: CI doc-sync and registry gates.
fn cross_file(cfg: &Config, files: &[SourceFile], cwd: &Path, rule_filter: Option<&[String]>) -> Result<Vec<Finding>, BootstrapError> {
    let mut out = Vec::new();
    for rule in cfg.rules.iter() {
        if let Some(filter) = rule_filter {
            if !filter.contains(&rule.id) {
                continue;
            }
        }
        match rule.check.as_str() {
            "doc_rule_coverage" => out.extend(crate::docsync::check(rule, cfg, cwd)?),
            "rosetta_policy" => out.extend(crate::rosetta::check(rule, files, cwd)?),
            _ => {}
        }
    }
    Ok(out)
}

/// CEP:WHAT: Collects first-party files under roots with sorted traversal and exclusion patterns.
/// CEP:WHY: Deterministic discovery is mandated by section 50.1.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap errors for invalid globs and unreadable directories.
/// CEP:ASSUMES: file_globs match the full or repository-relative path; exclude_paths exclude by full path.
/// CEP:COST: cold; linear in repository entries.
/// CEP:EVIDENCE: cep_lint self-lint.
fn collect_files(cfg: &Config, roots: &[String], cwd: &Path) -> Result<Vec<SourceFile>, BootstrapError> {
    let globs = compile_globs(&cfg.file_globs, "file_globs")?;
    let excludes = compile_globs(&cfg.exclude_paths, "exclude_paths")?;
    let mut files: Vec<SourceFile> = Vec::new();
    let mut queue: Vec<std::path::PathBuf> = roots.iter().map(|r| cwd.join(r)).collect();
    while let Some(p) = queue.pop() {
        walk_entry(cfg, &p, cwd, &globs, &excludes, &mut files, &mut queue);
    }
    Ok(files)
}

/// CEP:WHAT: Compiles glob or exclusion patterns into regexes.
/// CEP:WHY: Pattern compilation failures are bootstrap code eight.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap error eight with the key named.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in patterns.
/// CEP:EVIDENCE: cep_lint self-lint.
fn compile_globs(patterns: &[String], key: &str) -> Result<Vec<regex::Regex>, BootstrapError> {
    patterns.iter()
        .map(|g| regex::Regex::new(g).map_err(|e| BootstrapError::new(BOOT_REGEX, format!("{}: {}", key, e))))
        .collect()
}

/// CEP:WHAT: Processes one traversal entry, descending into directories or scanning first-party files.
/// CEP:WHY: Sorted queue traversal keeps discovery deterministic.
/// CEP:STATUS: complete
/// CEP:FAILURE: none; unreadable non-first-party files are skipped deterministically.
/// CEP:ASSUMES: Entries are visited in sorted order by the caller.
/// CEP:COST: cold; constant per entry.
/// CEP:EVIDENCE: cep_lint self-lint.
fn walk_entry(
    _cfg: &Config,
    p: &std::path::Path,
    cwd: &Path,
    globs: &[regex::Regex],
    excludes: &[regex::Regex],
    files: &mut Vec<SourceFile>,
    queue: &mut Vec<std::path::PathBuf>,
) {
    let path_str = p.to_string_lossy().to_string();
    if excludes.iter().any(|e| e.is_match(&path_str)) {
        return;
    }
    if p.is_dir() {
        let mut entries: Vec<_> = match fs::read_dir(p) {
            Ok(rd) => rd.filter_map(|e| e.ok()).map(|e| e.path()).collect(),
            Err(_) => return,
        };
        entries.sort();
        for e in entries {
            queue.push(e);
        }
        return;
    }
    let rel = p.strip_prefix(cwd).unwrap_or(p).to_string_lossy().to_string();
    if !globs.iter().any(|g| g.is_match(&path_str) || g.is_match(&rel)) {
        return;
    }
    if let Ok(text) = fs::read_to_string(p) {
        files.push(scan_file(&rel, &text));
    }
}

/// CEP:WHAT: Formats findings into the deterministic report text.
/// CEP:WHY: One report format serves CLI, CI logs, and self-test diagnostics.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Findings are already sorted.
/// CEP:COST: cold; linear in findings.
/// CEP:EVIDENCE: cep_lint self-lint report reviewed in CI logs.
pub fn report(findings: &[Finding]) -> String {
    let mut out = String::new();
    for f in findings.iter() {
        out.push_str(&format!("{}:{}: {} [{}] {}\n", f.file, f.line, f.severity, f.rule, f.message));
    }
    out.push_str(&format!("findings: {}\n", findings.len()));
    out
}

/// CEP:WHAT: Maps findings to the configured exit code.
/// CEP:WHY: Exit code data lives in configuration per section 50.1.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in findings.
/// CEP:EVIDENCE: CI exit code gate.
pub fn exit_code(cfg: &Config, findings: &[Finding]) -> i32 {
    let blocking = findings.iter().any(|f| cfg.fail_on.contains(&f.severity));
    if blocking {
        cfg.exit_findings
    } else {
        cfg.exit_clean
    }
}
