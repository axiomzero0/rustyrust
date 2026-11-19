// CEP:FILE: tools/cep_lint/src/scan.rs
// CEP:WHAT: Source file scanner producing comment blocks, stripped code, string literals, and function records.
// CEP:WHY: Every check primitive consumes one shared scan model so rule behavior is consistent and deterministic.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: none; scanning is total over UTF-8 text and never panics on malformed source.
// CEP:ASSUMES: Comment syntax is `//` line comments; block comments are stripped but not treated as field blocks (documented limit, section 50.5).
// CEP:COST: cold; linear scan per file with bounded backtracking for statement windows.
// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract; self-lint gate over tools/cep_lint/src.

/// A contiguous full-line comment block with parsed `CEP:` fields.
#[derive(Debug, Clone)]
pub struct CommentBlock {
    /// One-based first line of the block.
    pub start_line: usize,
    /// One-based last line of the block.
    pub end_line: usize,
    /// Parsed CEP fields in source order.
    pub fields: Vec<(String, String)>,
}

/// A function record extracted from stripped source text.
#[derive(Debug, Clone)]
pub struct Function {
    /// Function identifier from the signature.
    pub name: String,
    /// One-based line of the signature.
    pub decl_line: usize,
    /// Lexical statement count over the body.
    pub statement_count: usize,
    /// True when the body holds no statements.
    pub body_is_empty: bool,
    /// True when the body is a single constant return.
    pub constant_return: bool,
    /// Associated comment block, excluding the file header.
    pub block: Option<CommentBlock>,
}

/// A comment segment found on a source line with its content.
#[derive(Debug, Clone)]
pub struct CommentSeg {
    /// One-based source line of the segment.
    pub line: usize,
    /// Comment text after the marker.
    pub text: String,
}

/// A string literal with its start line and unquoted content.
#[derive(Debug,Clone)]
pub struct StrLit {
    /// One-based source line of the literal start.
    pub line: usize,
    /// Literal content without quotes.
    pub text: String,
}

/// The shared per-file scan model consumed by all check primitives.
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// Path relative to the working directory.
    pub path: String,
    /// Raw source lines.
    pub lines: Vec<String>,
    /// All full-line comment blocks in source order.
    pub blocks: Vec<CommentBlock>,
    /// First block when it starts within the header limit.
    pub header_block: Option<CommentBlock>,
    /// Declared CEP:CLASS value when present.
    pub class: Option<String>,
    /// Code-only lines with comments and strings erased.
    pub stripped_lines: Vec<String>,
    /// Comment segments with line numbers.
    pub comment_segs: Vec<CommentSeg>,
    /// String literals with line numbers.
    pub strings: Vec<StrLit>,
    /// Flattened stripped code for structural regex rules.
    pub flattened: String,
    /// Per-character source line map of the flattened text.
    pub flat_line_of: Vec<usize>,
    /// Extracted function records.
    pub functions: Vec<Function>,
}

/// CEP:WHAT: Parses a CEP field from comment content; returns the full CEP key and value.
/// CEP:WHY: Checks compare full field names such as CEP:CLASS against configuration data.
/// CEP:STATUS: complete
/// CEP:FAILURE: none; non-field lines yield None.
/// CEP:ASSUMES: Fields use the CEP:NAME: value shape.
/// CEP:COST: cold; constant per line.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract.
pub fn parse_field(content: &str) -> Option<(String, String)> {
    let trimmed = content.trim();
    let rest = trimmed.strip_prefix("CEP:")?;
    let idx = rest.find(':')?;
    let name = rest[..idx].trim().to_string();
    let value = rest[idx + 1..].trim().to_string();
    Some((format!("CEP:{}", name), value))
}

/// CEP:WHAT: Builds contiguous full-line comment blocks from raw lines.
/// CEP:WHY: Field blocks are the anchor for headers, statuses, and function documentation.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: A blank or code line terminates a block; documented in scan model limits.
/// CEP:COST: cold; linear in file lines.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract.
fn build_blocks(lines: &[String]) -> Vec<CommentBlock> {
    let mut blocks: Vec<CommentBlock> = Vec::new();
    let mut cur: Vec<(usize, String)> = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            cur.push((idx, trimmed.trim_start_matches('/').trim().to_string()));
        } else if !cur.is_empty() {
            blocks.push(finish_block(&mut cur));
        }
    }
    if !cur.is_empty() {
        blocks.push(finish_block(&mut cur));
    }
    blocks
}

/// CEP:WHAT: Converts accumulated comment lines into a block with parsed fields.
/// CEP:WHY: Field extraction must be identical for header and function blocks.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: The accumulator is non-empty; enforced by callers.
/// CEP:COST: cold; linear in block size.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract.
fn finish_block(cur: &mut Vec<(usize, String)>) -> CommentBlock {
    let start_line = cur[0].0 + 1;
    let end_line = cur[cur.len() - 1].0 + 1;
    let mut fields: Vec<(String, String)> = Vec::new();
    for (_, text) in cur.iter() {
        if let Some(f) = parse_field(text) {
            fields.push(f);
        }
    }
    cur.clear();
    CommentBlock { start_line, end_line, fields }
}

/// Advance applied after scanning a two-character sequence such as an escape or comment opener.
const SEQ_ADVANCE: usize = 2;

/// Replacement used to erase lines or literals that carry no code meaning.
/// Blank character used to erase code positions that carry no meaning after stripping.
const BLANK: char = ' ';

/// Marker character standing in for an erased literal so structural checks see non-empty bodies.
const LIT_MARK: char = '\u{7f}';

/// Closing marker of a Rust block comment.
const BLOCK_END: &str = "*/";

/// Offset from a quote to the character after a plain one-character literal.
const PLAIN_CHAR_LEN: usize = 3;

/// Offset from a quote to the payload character of a one-character literal.
const QUOTE_LOOKAHEAD: usize = 2;

/// Offset from an escape backslash to its payload character.
const ESC_PAYLOAD_OFFSET: usize = 2;

/// CEP:WHAT: Removes comment and string contents from one line, preserving code characters.
/// CEP:WHY: Code-only scanning is required by the banned token and numeric literal rules.
/// CEP:ID: CS-a07
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Unterminated strings are treated as line-scoped; documented in scan model limits.
/// CEP:COST: cold; linear in line length.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract.
fn strip_line(line: &str) -> (String, Option<StrLit>, Vec<CommentSeg>, bool) {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut segs: Vec<CommentSeg> = Vec::new();
    let mut lit: Option<StrLit> = None;
    let mut i = 0;
    let mut in_block = false;
    while i < chars.len() {
        let c = chars[i];
        if in_block {
            if c == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                in_block = false;
                i += SEQ_ADVANCE;
            } else {
                i += 1;
            }
            continue;
        }
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            let content: String = chars[i + SEQ_ADVANCE..].iter().collect();
            segs.push(CommentSeg { line: 0, text: content });
            break;
        }
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            in_block = true;
            i += SEQ_ADVANCE;
            continue;
        }
        if c == '"' {
            let (content, next) = scan_string(&chars, i, '"');
            lit = Some(StrLit { line: 0, text: content });
            out.push(LIT_MARK);
            i = next;
            continue;
        }
        if c == '\'' {
            let (is_char, next) = scan_char(&chars, i);
            if is_char {
                out.push(LIT_MARK);
                i = next;
            } else {
                out.push(c);
                i += 1;
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    if in_block {
        out.push(BLANK);
    }
    (out, lit, segs, in_block)
}

/// CEP:WHAT: Scans a string literal starting at quote position; returns content and next index.
/// CEP:WHY: String contents feed the placeholder text and secret scan rules.
/// CEP:STATUS: complete
/// CEP:FAILURE: none; an unterminated literal consumes the rest of the line.
/// CEP:ASSUMES: Multiline strings are not used in first-party sources; documented limit.
/// CEP:COST: cold; linear in literal length.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract.
fn scan_string(chars: &[char], start: usize, quote: char) -> (String, usize) {
    let mut content = String::new();
    let mut i = start + 1;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() {
            content.push(c);
            content.push(chars[i + 1]);
            i += SEQ_ADVANCE;
            continue;
        }
        if c == quote {
            return (content, i + 1);
        }
        content.push(c);
        i += 1;
    }
    (content, chars.len())
}

/// CEP:WHAT: Decides whether a quote is a char literal or a lifetime; returns is_char and next index.
/// CEP:WHY: Lifetimes share the quote character with char literals; misclassification would erase code.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Escaped char literals close on the same line; documented limit.
/// CEP:COST: cold; linear scan in the escape case.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract.
fn scan_char(chars: &[char], start: usize) -> (bool, usize) {
    if start + QUOTE_LOOKAHEAD < chars.len() && chars[start + 1] != '\\' && chars[start + QUOTE_LOOKAHEAD] == '\'' {
        return (true, start + PLAIN_CHAR_LEN);
    }
    if start + 1 < chars.len() && chars[start + 1] == '\\' {
        let mut i = start + ESC_PAYLOAD_OFFSET;
        while i < chars.len() {
            if chars[i] == '\'' {
                return (true, i + 1);
            }
            i += 1;
        }
        return (false, start + 1);
    }
    (false, start + 1)
}

/// CEP:WHAT: Strips comments and strings from all lines, tracking multi-line block comments.
/// CEP:WHY: Code-only views are shared by every lexical rule; one pass keeps them consistent.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Block comment close markers are detected on raw lines; code after a close marker on the same line is treated as comment.
/// CEP:COST: cold; linear in file size.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract.
fn strip_source(lines: &[String]) -> (Vec<String>, Vec<StrLit>, Vec<CommentSeg>) {
    let mut out: Vec<String> = Vec::new();
    let mut lits: Vec<StrLit> = Vec::new();
    let mut segs: Vec<CommentSeg> = Vec::new();
    let mut in_block = false;
    for (idx, line) in lines.iter().enumerate() {
        if in_block {
            let blank: String = std::iter::repeat_n(BLANK, line.chars().count()).collect();
            out.push(blank);
            if line.contains(BLOCK_END) {
                in_block = false;
            }
            continue;
        }
        strip_normal_line(idx, line, &mut out, &mut lits, &mut segs, &mut in_block);
    }
    (out, lits, segs)
}

/// CEP:WHAT: Strips one non-block-comment line and records its literals and comment segments.
/// CEP:WHY: Splitting keeps strip_source within statement limits and single responsibility.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: The line is not inside a block comment; enforced by the caller.
/// CEP:COST: cold; linear in line length.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract.
fn strip_normal_line(
    idx: usize,
    line: &str,
    out: &mut Vec<String>,
    lits: &mut Vec<StrLit>,
    segs: &mut Vec<CommentSeg>,
    in_block: &mut bool,
) {
    let (code, lit, mut line_segs, still_open) = strip_line(line);
    out.push(code);
    if let Some(mut l) = lit {
        l.line = idx + 1;
        lits.push(l);
    }
    for s in line_segs.iter_mut() {
        s.line = idx + 1;
    }
    segs.append(&mut line_segs);
    if still_open {
        *in_block = true;
    }
}

/// CEP:WHAT: Flattens stripped lines into one code string with a per-character line map.
/// CEP:WHY: Flat code regex rules need one search space plus offset to line mapping.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in file size.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract.
fn flatten(lines: &[String]) -> (String, Vec<usize>) {
    let mut flat = String::new();
    let mut map: Vec<usize> = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        for _ in line.chars() {
            map.push(idx + 1);
        }
        flat.push_str(line);
        map.push(idx + 1);
        flat.push(' ');
    }
    (flat, map)
}

/// CEP:WHAT: Finds the closing brace offset in flattened code starting at an open brace offset.
/// CEP:WHY: Function body extraction requires balanced brace matching over stripped text.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns None when braces are unbalanced; the function record is skipped.
/// CEP:ASSUMES: Braces inside string literals are erased by stripping; enforced by scan order.
/// CEP:COST: cold; linear in remaining input per call.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract.
fn find_body_end(flat: &[char], open: usize) -> Option<usize> {
    let mut depth = 0;
    let mut i = open;
    while i < flat.len() {
        match flat[i] {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// CEP:WHAT: Extracts function records from raw and stripped lines with comment-block association.
/// CEP:WHY: All five function_policy rules share one extractor so their limits stay aligned.
/// CEP:ID: CS-a01
/// CEP:STATUS: complete
/// CEP:FAILURE: none; malformed or brace-unbalanced functions are skipped deterministically.
/// CEP:ASSUMES: Signatures fit on one stripped line; brace matching runs over the flattened stripped text.
/// CEP:COST: cold; linear in file size per file plus linear per function.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract; doc file docs/functions/CS/CS-a01.md.
fn extract_functions(lines: &[String], stripped: &[String], blocks: &[CommentBlock]) -> Vec<Function> {
    let (flat, _map) = flatten(stripped);
    let byte_flat: Vec<char> = flat.chars().collect();
    let mut fns: Vec<Function> = Vec::new();
    let sig_re = regex::Regex::new(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)").expect("fn signature regex");
    let cr_re = regex::Regex::new(r"^\s*(return\s+)?(true|false)\s*;?\s*$").expect("constant return regex");
    for (idx, line) in stripped.iter().enumerate() {
        let Some(cap) = sig_re.captures(line) else { continue };
        let name = cap.get(1).unwrap().as_str().to_string();
        let char_prefix: usize = stripped[..idx].iter().map(|l| l.chars().count() + 1).sum();
        let local_off = flat_char_offset(line, cap.get(0).unwrap().start());
        let global_off = char_prefix + local_off;
        let Some(brace_rel) = find_open_brace(&byte_flat, global_off) else { continue };
        if has_semicolon_before(&byte_flat, global_off, brace_rel) {
            continue;
        }
        let Some(end_rel) = find_body_end(&byte_flat, brace_rel) else { continue };
        let body: String = byte_flat[brace_rel + 1..end_rel].iter().collect();
        let statement_count = body.matches(';').count();
        let body_is_empty = body.trim().is_empty();
        let constant_return = cr_re.is_match(&body);
        let decl_line = idx + 1;
        let block = assoc_block(decl_line, lines, blocks);
        fns.push(Function { name, decl_line, statement_count, body_is_empty, constant_return, block });
    }
    fns
}

/// CEP:WHAT: Computes the character offset of a byte position within one line.
/// CEP:WHY: Regex byte offsets must map to flattened character offsets.
/// CEP:STATUS: complete
/// CEP:FAILURE: none; byte positions from regex matches are always at char boundaries.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in prefix length.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract.
fn flat_char_offset(line: &str, byte_pos: usize) -> usize {
    line[..byte_pos].chars().count()
}

/// CEP:WHAT: Finds the first open brace at or after an offset in flattened code.
/// CEP:WHY: Function bodies start at the first brace after the signature.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns None when a semicolon ends the item first; the record is skipped.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in remaining input.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract.
fn find_open_brace(flat: &[char], from: usize) -> Option<usize> {
    let mut i = from;
    while i < flat.len() {
        if flat[i] == '{' {
            return Some(i);
        }
        if flat[i] == ';' {
            return None;
        }
        i += 1;
    }
    None
}

/// CEP:WHAT: Reports whether a semicolon occurs between two offsets in flattened code.
/// CEP:WHY: Declarations without bodies end in a semicolon and must not be treated as functions.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in span length.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract.
fn has_semicolon_before(flat: &[char], from: usize, to: usize) -> bool {
    flat[from..to].contains(&';')
}

/// CEP:WHAT: Returns the comment block associated with a declaration line, skipping gap lines.
/// CEP:WHY: Attributes and visibility qualifiers may sit between a block and its function.
/// CEP:STATUS: complete
/// CEP:FAILURE: none; returns None when no contiguous block precedes the declaration.
/// CEP:ASSUMES: Gap lines match the configured attribute pattern; function blocks end directly above the gap run.
/// CEP:COST: cold; linear in gap run length.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract.
fn assoc_block(decl_line: usize, lines: &[String], blocks: &[CommentBlock]) -> Option<CommentBlock> {
    let gap_re = regex::Regex::new(r#"^\s*(#\[[^\]]*\]|pub(\([^)]*\))?|unsafe|extern(\s+"[^"]+")?|async|const(\s+[A-Za-z_][A-Za-z0-9_]*(:[^=]+)?=[^;]*;)?)\s*$"#).expect("gap regex");
    let cursor = skip_gap_lines(lines, decl_line - 1, &gap_re);
    if cursor == 0 {
        return None;
    }
    let above = lines[cursor - 1].trim_start();
    if !above.starts_with("//") {
        return None;
    }
    let target = cursor;
    blocks.iter().rev().find(|b| b.end_line == target).cloned()
}

/// CEP:WHAT: Walks upward over gap lines and returns the stop index.
/// CEP:WHY: Attributes and declarations may sit between a comment block and its function.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Blank lines terminate the walk per the block contiguity model.
/// CEP:COST: cold; linear in gap run length.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract.
fn skip_gap_lines(lines: &[String], start: usize, gap_re: &regex::Regex) -> usize {
    let mut cursor = start;
    while cursor > 0 {
        let l = lines[cursor - 1].trim();
        if l.is_empty() || !gap_re.is_match(l) {
            break;
        }
        cursor -= 1;
    }
    cursor
}

/// CEP:WHAT: Scans one source file into the shared scan model.
/// CEP:WHY: Every check consumes this single model for consistent, deterministic behavior.
/// CEP:STATUS: complete
/// CEP:FAILURE: none; scanning is total over UTF-8 text.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in file size.
/// CEP:EVIDENCE: unit test cep_lint::scan::tests::scan_and_extract.
pub fn scan_file(path: &str, text: &str) -> SourceFile {
    let lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    let blocks = build_blocks(&lines);
    let header_block = blocks.first().cloned();
    let class = header_block.as_ref().and_then(|b| {
        b.fields.iter().find(|(k, _)| k == "CEP:CLASS").map(|(_, v)| v.clone())
    });
    let (stripped, strings, segs) = strip_source(&lines);
    let (flattened, flat_line_of) = flatten(&stripped);
    let functions = extract_functions(&lines, &stripped, &blocks);
    SourceFile {
        path: path.to_string(),
        lines,
        blocks,
        header_block,
        class,
        stripped_lines: stripped,
        comment_segs: segs,
        strings,
        flattened,
        flat_line_of,
        functions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Expected function count in the sample source.
    const SAMPLE_FN_COUNT: usize = 2;

    /// Expected header field count in the sample source.
    const SAMPLE_FIELD_COUNT: usize = 2;

    /// Expected statement count of the second sample function.
    const SAMPLE_STMT_COUNT: usize = 2;

/// CEP:WHAT: Verifies block parsing, stripping, and function extraction on a small sample.
/// CEP:WHY: The scan model is the base of every rule; regressions must fail fast.
/// CEP:STATUS: complete
/// CEP:FAILURE: Fails the test binary on regression.
/// CEP:ASSUMES: none
/// CEP:COST: cold; unit test only.
/// CEP:EVIDENCE: cargo test -p cep_lint.
    #[test]
    fn scan_and_extract() {
        let src = "// CEP:FILE: x.rs\n// CEP:CLASS: CEP-1\nfn one() { a(); }\n// CEP:WHAT: d\nfn two() {\n  a();\n  b();\n}\n";
        let f = scan_file("x.rs", src);
        assert_eq!(f.header_block.as_ref().unwrap().fields.len(), SAMPLE_FIELD_COUNT);
        assert_eq!(f.class.as_deref(), Some("CEP-1"));
        assert_eq!(f.functions.len(), SAMPLE_FN_COUNT);
        assert_eq!(f.functions[1].statement_count, SAMPLE_STMT_COUNT);
    }
}
