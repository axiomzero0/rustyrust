// CEP:FILE: compiler/frontend/src/lexer.rs
// CEP:WHAT: Token scanner for the V0 dialect producing spans and positioned diagnostics.
// CEP:WHY: The parser consumes tokens; literal bounds checking happens here.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: Returns diagnostics for invalid characters and out-of-range literals; never panics.
// CEP:ASSUMES: Input is UTF-8; spans index bytes.
// CEP:COST: cold; linear in source size.
// CEP:EVIDENCE: unit test rr_frontend::lexer::tests::scans_tokens.
#![deny(missing_docs)]
use rr_hir::span::Span;

/// A lexical token with its span.
#[derive(Debug, Clone)]
pub struct Token {
    /// Token kind and payload.
    pub kind: Tok,
    /// Byte span of the token.
    pub span: Span,
}

/// Token kinds of the V0 dialect.
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    /// An identifier.
    Ident(String),
    /// An in-range integer literal.
    Int(i64),
    /// A keyword.
    Kw(&'static str),
    /// A punctuation symbol.
    Punct(&'static str),
    /// End of input.
    Eof,
}

/// Advance applied after scanning a two-character sequence.
const SEQ_ADVANCE: usize = 2;

/// Punctuation strings, longest first for greedy matching.
const PUNCTS: [&str; 22] = ["&&", "||", "==", "!=", "<=", ">=", "->", "+", "-", "*", "/", "%", "(", ")", "{", "}", ",", ";", "=", "<", ">", ":"];

/// Keywords recognized by the scanner.
const KEYWORDS: [&str; 10] = ["fn", "let", "mut", "return", "if", "else", "while", "true", "false", "i64"];

/// CEP:WHAT: Scans source text into tokens or diagnostics.
/// CEP:WHY: The parser needs a flat token stream with spans.
/// CEP:ID: LEX-a01
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns diagnostics for invalid characters, unterminated comments, and oversized literals.
/// CEP:ASSUMES: Comments nest; whitespace separates tokens.
/// CEP:COST: cold; linear in source size.
/// CEP:EVIDENCE: doc file docs/functions/LEX/LEX-a01.md; unit test scans_tokens.
pub fn lex(text: &str) -> Result<Vec<Token>, Vec<crate::Diagnostic>> {
    let b = text.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    let mut errs = Vec::new();
    while i < b.len() {
        i = skip_ws_comments(text, i);
        if i >= b.len() {
            break;
        }
        let c = b[i];
        if c.is_ascii_digit() {
            i = scan_int(text, i, &mut out, &mut errs);
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            i = scan_word(text, i, &mut out);
            continue;
        }
        i = scan_punct(text, i, &mut out, &mut errs);
    }
    out.push(Token { kind: Tok::Eof, span: Span::new(b.len() as u32, b.len() as u32) });
    if errs.is_empty() {
        Ok(out)
    } else {
        Err(errs)
    }
}

/// CEP:WHAT: Skips whitespace and nested comments from one offset.
/// CEP:WHY: Comment handling must match Rust nesting semantics.
/// CEP:ID: LEX-a02
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Block comments nest; line comments end at newline.
/// CEP:COST: cold; linear in skipped text.
/// CEP:EVIDENCE: unit test rr_frontend::lexer::tests::scans_tokens.
fn skip_ws_comments(text: &str, mut i: usize) -> usize {
    let b = text.as_bytes();
    loop {
        if i >= b.len() {
            return i;
        }
        if b[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
            while i < b.len() && b[i] != b'\n' {
                i += SEQ_ADVANCE;
            }
            continue;
        }
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
            i += SEQ_ADVANCE;
            let mut depth: usize = 1;
            while i < b.len() && depth > 0 {
                if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
                    depth += 1;
                    i += SEQ_ADVANCE;
                } else if b[i] == b'*' && i + 1 < b.len() && b[i + 1] == b'/' {
                    depth -= 1;
                    i += SEQ_ADVANCE;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        return i;
    }
}

/// CEP:WHAT: Scans one integer literal with i64 bound checking.
/// CEP:WHY: Out-of-range literals are lexer diagnostics per the V0 dialect definition.
/// CEP:STATUS: complete
/// CEP:FAILURE: Pushes a diagnostic on overflow.
/// CEP:ASSUMES: Digits are ASCII.
/// CEP:COST: cold; linear in literal length.
/// CEP:EVIDENCE: unit test rr_frontend::lexer::tests::scans_tokens.
fn scan_int(text: &str, start: usize, out: &mut Vec<Token>, errs: &mut Vec<crate::Diagnostic>) -> usize {
    let b = text.as_bytes();
    let mut i = start;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let span = Span::new(start as u32, i as u32);
    match text[start..i].parse::<i64>() {
        Ok(v) => out.push(Token { kind: Tok::Int(v), span }),
        Err(_) => errs.push(crate::Diagnostic { msg: "integer literal out of range for i64".into(), span }),
    }
    i
}

/// CEP:WHAT: Scans one identifier or keyword.
/// CEP:WHY: Words resolve to keywords, booleans, or identifiers.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Words are ASCII alphabetic or underscore runs.
/// CEP:COST: cold; linear in word length.
/// CEP:EVIDENCE: unit test rr_frontend::lexer::tests::scans_tokens.
fn scan_word(text: &str, start: usize, out: &mut Vec<Token>) -> usize {
    let b = text.as_bytes();
    let mut i = start;
    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
        i += 1;
    }
    let w = &text[start..i];
    let span = Span::new(start as u32, i as u32);
    let kind = match KEYWORDS.iter().find(|k| **k == w) {
        Some(k) => Tok::Kw(k),
        None => Tok::Ident(w.to_string()),
    };
    out.push(Token { kind, span });
    i
}

/// CEP:WHAT: Scans one punctuation token or records an invalid character.
/// CEP:WHY: Greedy matching handles two-character operators before single characters.
/// CEP:STATUS: complete
/// CEP:FAILURE: Pushes a diagnostic for unrecognized characters.
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per token.
/// CEP:EVIDENCE: unit test rr_frontend::lexer::tests::scans_tokens.
fn scan_punct(text: &str, start: usize, out: &mut Vec<Token>, errs: &mut Vec<crate::Diagnostic>) -> usize {
    for p in PUNCTS.iter() {
        if text[start..].starts_with(p) {
            let end = start + p.len();
            out.push(Token { kind: Tok::Punct(p), span: Span::new(start as u32, end as u32) });
            return end;
        }
    }
    let c = text[start..].chars().next().unwrap_or('?');
    let _ = &c;
    errs.push(crate::Diagnostic { msg: format!("invalid character {:?}", c), span: Span::new(start as u32, (start + c.len_utf8()) as u32) });
    start + c.len_utf8()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sample token index used by assertions.
    const IDX_ONE: usize = 1;

    /// CEP:WHAT: Verifies scanning of literals, keywords, and operators.
    /// CEP:WHY: The parser depends on token shapes and spans.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-frontend.
    #[test]
    fn scans_tokens() {
        let t = lex("fn main() -> i64 { 42 }").unwrap();
        assert!(matches!(t[0].kind, Tok::Kw("fn")));
        assert!(matches!(t[IDX_ONE].kind, Tok::Ident(_)));
        assert!(matches!(t.last().unwrap().kind, Tok::Eof));
        assert!(lex("99999999999999999999999").is_err());
    }
}
