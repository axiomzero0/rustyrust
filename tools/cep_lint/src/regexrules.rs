// CEP:FILE: tools/cep_lint/src/regexrules.rs
// CEP:WHAT: Lexical rule primitives: comment regexes, string literal regexes, flat code regexes, numeric literals, macro prefixes, and banned tokens.
// CEP:WHY: Rules CEP-LINT-TODO-OWNER through CEP-LINT-HOT-BANNED share these scanners.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: none; checks are total over scanned files.
// CEP:ASSUMES: Patterns are validated as regexes at configuration load time.
// CEP:COST: cold; linear in file size per rule.
// CEP:EVIDENCE: self-test scenario set for the lexical rules.

use crate::model::{Finding, Rule};
use crate::scan::SourceFile;

/// Hexadecimal radix for numeric literal parsing.
const HEX_RADIX: u32 = 16;

/// Fallback clause that never matches, used when no suppression is configured.
const NEVER_MATCH: &str = "^$\\B";

/// CEP:WHAT: Builds a finding by substituting the detail slot of the rule message.
/// CEP:WHY: Message templates live in configuration; checks supply one detail string.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in template length.
/// CEP:EVIDENCE: cep_lint self-lint.
fn mk(rule: &Rule, file: &SourceFile, line: usize, detail: String) -> Finding {
    let msg = crate::model::substitute(&rule.message, &[("detail", &detail)]);
    Finding { severity: rule.severity, rule: rule.id.clone(), file: file.path.clone(), line, message: msg }
}

/// CEP:WHAT: Reads the main pattern parameter of a rule.
/// CEP:WHY: Lexical rules are pattern-driven by configuration.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: cep_lint self-lint.
fn param_pattern(rule: &Rule) -> String {
    rule.params.get("pattern").and_then(|v| v.as_str()).unwrap_or("").to_string()
}

/// CEP:WHAT: Reads the suppression pattern parameter of a rule if present.
/// CEP:WHY: Suppression patterns declare sanctioned forms.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: cep_lint self-lint.
fn param_suppression(rule: &Rule) -> Option<String> {
    rule.params.get("suppression_pattern").and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// CEP:WHAT: Reads the icase flag parameter of a rule.
/// CEP:WHY: Marker rules are configured case-insensitively per section 50.3.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: cep_lint self-lint.
fn param_icase(rule: &Rule) -> bool {
    rule.params.get("icase").and_then(|v| v.as_bool()).unwrap_or(false)
}

/// CEP:WHAT: Compiles a configured pattern with the icase flag applied.
/// CEP:WHY: Patterns are validated at load; compilation here cannot fail.
/// CEP:STATUS: complete
/// CEP:FAILURE: Panics only on patterns that load-time validation already rejected.
/// CEP:ASSUMES: Load-time validation compiled the same pattern.
/// CEP:COST: cold; one regex compilation per rule per file.
/// CEP:EVIDENCE: cep_lint self-lint.
fn compile(pattern: &str, icase: bool) -> regex::Regex {
    let src = if icase { format!("(?i){}", pattern) } else { pattern.to_string() };
    regex::Regex::new(&src).expect("rule pattern validated at load")
}

/// CEP:WHAT: Applies a line-scoped regex over all comment segments with optional suppression.
/// CEP:WHY: Work-marker ownership, reserved-status comments, and commented code are comment-text rules.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Comment segments are extracted by scan.rs outside strings.
/// CEP:COST: cold; linear in comment segments.
/// CEP:EVIDENCE: self-test scenarios for comment_regex rules.
pub fn check_comment_regex(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    let re = compile(&param_pattern(rule), param_icase(rule));
    let sup_src = param_suppression(rule).unwrap_or_else(|| NEVER_MATCH.to_string());
    let sup = compile(&sup_src, false);
    let mut out = Vec::new();
    for seg in file.comment_segs.iter() {
        if sup.is_match(&seg.text) {
            continue;
        }
        if re.is_match(&seg.text) {
            out.push(mk(rule, file, seg.line, format!("comment text {}", seg.text.trim())));
        }
    }
    out
}

/// CEP:WHAT: Applies a regex over string literal contents.
/// CEP:WHY: Reserved-status markers must fail loudly rather than leak into runtime messages.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: String literals are extracted by scan.rs.
/// CEP:COST: cold; linear in string literals.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-PLACEHOLDER-TEXT.
pub fn check_string_literal(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    let re = compile(&param_pattern(rule), param_icase(rule));
    let mut out = Vec::new();
    for lit in file.strings.iter() {
        if re.is_match(&lit.text) {
            out.push(mk(rule, file, lit.line, "string literal contains a marker".to_string()));
        }
    }
    out
}

/// CEP:WHAT: Applies a regex over flattened stripped code with name validation when configured.
/// CEP:WHY: Empty catch and empty branch rules are structural patterns; naming rules validate captured names.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: The flattened text preserves character counts per line.
/// CEP:COST: cold; linear in file size.
/// CEP:EVIDENCE: self-test scenarios for flat_code_regex rules.
pub fn check_flat_code_regex(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    let re = compile(&param_pattern(rule), false);
    let reject = rule.params.get("reject_pattern").and_then(|v| v.as_str()).map(compile_ctor());
    let name_group = rule.params.get("name_group").and_then(|v| v.as_i64());
    let name_re = rule.params.get("name_pattern").and_then(|v| v.as_str())
        .map(|p| compile(p, false));
    let mut out = Vec::new();
    for m in re.captures_iter(&file.flattened) {
        let whole = m.get(0).expect("capture zero");
        if let Some(rre) = reject.as_ref() {
            if rre.is_match(whole.as_str()) {
                continue;
            }
        }
        if let (Some(g), Some(nre)) = (name_group, name_re.as_ref()) {
            if let Some(cap) = m.get(g as usize) {
                if nre.is_match(cap.as_str()) {
                    continue;
                }
            }
        }
        let line = line_of(&file.flat_line_of, whole.start());
        out.push(mk(rule, file, line, format!("matched pattern for {}", rule.id)));
    }
    out
}

/// CEP:WHAT: Returns a constructor that compiles a validated pattern.
/// CEP:WHY: Optional pattern parameters share the load-validated compilation path.
/// CEP:STATUS: complete
/// CEP:FAILURE: Panics only on patterns that load-time validation already rejected.
/// CEP:ASSUMES: Load-time validation compiled the same pattern.
/// CEP:COST: cold; one compilation per call.
/// CEP:EVIDENCE: cep_lint self-lint.
fn compile_ctor() -> fn(&str) -> regex::Regex {
    |p: &str| regex::Regex::new(p).expect("optional pattern validated at load")
}

/// CEP:WHAT: Maps a flattened character offset to its source line.
/// CEP:WHY: Flat code findings must point at real source lines.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: cep_lint self-lint.
fn line_of(map: &[usize], offset: usize) -> usize {
    map.get(offset).copied().unwrap_or(0)
}

/// CEP:WHAT: Flags numeric literals outside the allow-list that lack a sanctioned context.
/// CEP:WHY: Law 7 bans hard-coded assumptions; the mechanical form is the numeric literal policy.
/// CEP:ID: CS-a02
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Exemption windows join at most twelve statement lines; mirrors section 50.3.
/// CEP:COST: cold; linear in numeric literals with bounded window joins.
/// CEP:EVIDENCE: doc file docs/functions/CS/CS-a02.md; self-test scenarios.
pub fn check_numeric_literals(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    let allowed = rule.params.get("allowed_values")
        .and_then(|v| v.as_arr())
        .map(|items| items.iter().filter_map(|v| v.as_i64()).collect::<Vec<i64>>())
        .unwrap_or_default();
    let contexts = rule.params.get("exempt_contexts")
        .and_then(|v| v.as_arr())
        .map(|items| items.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
        .unwrap_or_default();
    let re = regex::Regex::new(r"\b(0x[0-9a-fA-F_]+|[0-9][0-9_]*)\b").expect("numeric literal regex");
    let mut out = Vec::new();
    for (idx, line) in file.stripped_lines.iter().enumerate() {
        for m in re.find_iter(line) {
            if let Some(f) = literal_finding(rule, file, idx, m.as_str(), &allowed, &contexts) {
                out.push(f);
            }
        }
    }
    out
}

/// CEP:WHAT: Decides one numeric literal occurrence against the allow-list and exemption contexts.
/// CEP:WHY: Splitting the decision keeps the scan loop within statement limits.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; bounded window join per literal.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-MAGIC-NUMBER.
fn literal_finding(
    rule: &Rule,
    file: &SourceFile,
    idx: usize,
    raw: &str,
    allowed: &[i64],
    contexts: &[String],
) -> Option<Finding> {
    let value = parse_literal(raw);
    if allowed.contains(&value) {
        return None;
    }
    let window = statement_window(&file.stripped_lines, idx);
    let sanctioned = contexts.iter().any(|c| {
        regex::Regex::new(c).expect("exempt context validated at load").is_match(&window)
    });
    if sanctioned {
        return None;
    }
    Some(mk(rule, file, idx + 1, format!("literal {} lacks a sanctioned context", raw)))
}

/// CEP:WHAT: Parses a numeric literal with hexadecimal and underscore forms.
/// CEP:WHY: Value comparison must treat 0x0 and 0 identically per section 50.3.
/// CEP:STATUS: complete
/// CEP:FAILURE: none; unparsable literals clamp to the largest value and never match allow-lists.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in literal length.
/// CEP:EVIDENCE: cep_lint self-lint.
fn parse_literal(raw: &str) -> i64 {
    let cleaned = raw.replace('_', "");
    if let Some(hex) = cleaned.strip_prefix("0x") {
        return i64::from_str_radix(hex, HEX_RADIX).unwrap_or(i64::MAX);
    }
    cleaned.parse::<i64>().unwrap_or(i64::MAX)
}

/// CEP:WHAT: Joins the statement text around a line, bounded to the twelve-line window of section 50.3.
/// CEP:WHY: Sanctioned forms may split a declaration across adjacent lines.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: The window covers the statement; tighter boundaries rely on review per section 50.3.
/// CEP:COST: cold; bounded by the window size.
/// CEP:EVIDENCE: cep_lint self-lint.
fn statement_window(lines: &[String], idx: usize) -> String {
    let start = idx.saturating_sub(STATEMENT_WINDOW);
    let end = (idx + STATEMENT_WINDOW).min(lines.len());
    lines[start..end].join(" ")
}

/// Statement window size mandated by section 50.3 for exemption context joining.
const STATEMENT_WINDOW: usize = 12;

/// CEP:WHAT: Flags macro definitions whose names lack the sanctioned prefix.
/// CEP:WHY: Sections 33.16 and 8.4 restrict macro naming; Rust macro definitions map to the same policy.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in file size.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-MACRO-PREFIX.
pub fn check_macro_prefix(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    let name_pattern = rule.params.get("name_pattern").and_then(|v| v.as_str()).unwrap_or("^CEP_[A-Z0-9_]+$").to_string();
    let guard_pattern = rule.params.get("guard_pattern").and_then(|v| v.as_str()).unwrap_or("^_?CEP_").to_string();
    let name_re = regex::Regex::new(&name_pattern).expect("macro name pattern validated at load");
    let guard_re = regex::Regex::new(&guard_pattern).expect("macro guard pattern validated at load");
    let find_re = regex::Regex::new(r"macro_rules!\s+([A-Za-z_][A-Za-z0-9_]*)").expect("macro definition regex");
    let mut out = Vec::new();
    for (idx, line) in file.stripped_lines.iter().enumerate() {
        for cap in find_re.captures_iter(line) {
            let name = cap.get(1).expect("capture one").as_str();
            if !name_re.is_match(name) && !guard_re.is_match(name) {
                out.push(mk(rule, file, idx + 1, format!("macro {} lacks the sanctioned prefix", name)));
            }
        }
    }
    out
}

/// CEP:WHAT: Flags banned tokens in code of files whose declared class is in the applies-to list.
/// CEP:WHY: Cycle-exact classes ban allocation, exceptions, and runtime machinery on sight.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Tokens are matched against stripped code so comments and strings never trigger findings.
/// CEP:COST: cold; linear in tokens times file size.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-HOT-BANNED.
pub fn check_banned_tokens(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    let applies = str_param(rule, "applies_to_classes");
    let class_field = rule.params.get("class_field").and_then(|v| v.as_str()).unwrap_or("CEP:CLASS").to_string();
    let tokens = str_param(rule, "tokens");
    let declared = file.header_block.as_ref()
        .and_then(|b| b.fields.iter().find(|(k, _)| k == &class_field).map(|(_, v)| v.clone()));
    let Some(declared) = declared else { return Vec::new() };
    if !applies.iter().any(|a| a == &declared) {
        return Vec::new();
    }
    banned_token_findings(rule, file, &declared, &tokens)
}

/// CEP:WHAT: Produces banned token findings for one qualifying file.
/// CEP:WHY: Splitting the scan keeps the entry check within statement limits.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in tokens times lines.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-HOT-BANNED.
fn banned_token_findings(rule: &Rule, file: &SourceFile, declared: &str, tokens: &[String]) -> Vec<Finding> {
    let mut out = Vec::new();
    for token in tokens.iter() {
        for (idx, line) in file.stripped_lines.iter().enumerate() {
            if token_hit(line, token) {
                out.push(mk(rule, file, idx + 1, format!("token {} is banned in class {}", token, declared)));
            }
        }
    }
    out
}

/// CEP:WHAT: Reads a string list parameter from rule params.
/// CEP:WHY: Token and class lists are configuration data.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Non-string items are ignored deterministically.
/// CEP:COST: cold; linear in list length.
/// CEP:EVIDENCE: cep_lint self-lint.
fn str_param(rule: &Rule, key: &str) -> Vec<String> {
    rule.params.get(key)
        .and_then(|v| v.as_arr())
        .map(|items| items.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default()
}

/// CEP:WHAT: Reports whether a token occurs in a code line with word boundaries for plain words.
/// CEP:WHY: Word-bounded matching prevents token substrings inside identifiers.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Phrase tokens containing punctuation match literally.
/// CEP:COST: cold; linear in line length.
/// CEP:EVIDENCE: cep_lint self-lint.
fn token_hit(line: &str, token: &str) -> bool {
    if token.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        let pattern = format!("\\b{}\\b", regex::escape(token));
        return regex::Regex::new(&pattern).expect("token regex").is_match(line);
    }
    line.contains(token)
}

/// CEP:WHAT: Dispatches a lexical rule by its check primitive name.
/// CEP:WHY: One dispatch point keeps engine code free of rule identifiers.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Unknown primitives yield no findings, which configuration validation prevents.
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: cep_lint self-lint.
pub fn check(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    match rule.check.as_str() {
        "comment_regex" => check_comment_regex(rule, file),
        "string_literal_regex" => check_string_literal(rule, file),
        "flat_code_regex" => check_flat_code_regex(rule, file),
        "numeric_literal_policy" => check_numeric_literals(rule, file),
        "macro_prefix_policy" => check_macro_prefix(rule, file),
        "banned_token_policy" => check_banned_tokens(rule, file),
        _ => Vec::new(),
    }
}
