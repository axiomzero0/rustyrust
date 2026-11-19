// CEP:FILE: compiler/frontend/src/lib.rs
// CEP:WHAT: Library root for the frontend: lexer and parser.
// CEP:WHY: Source text becomes HIR here; diagnostics carry spans.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: none
// CEP:ASSUMES: Input is UTF-8 text; malformed input yields diagnostics, never panics.
// CEP:COST: cold; linear in source size.
// CEP:EVIDENCE: cargo test -p rr-frontend.
//! Frontend of RustyRust: lexing and parsing for the V0 dialect.

/// Token definition and scanning.
pub mod lexer;
/// Recursive-descent parser with precedence climbing.
pub mod parser;

use rr_hir::hir::Program;

/// A diagnostic with a source span.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Message text.
    pub msg: String,
    /// Byte span of the offending region.
    pub span: rr_hir::span::Span,
}

/// CEP:WHAT: Parses one source file into a program or diagnostics.
/// CEP:WHY: One entry point keeps driver and tests on the same path.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns diagnostics for malformed source; never panics on input.
/// CEP:ASSUMES: The input is a complete source file.
/// CEP:COST: cold; linear in source size.
/// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
pub fn parse_source(text: &str) -> Result<Program, Vec<Diagnostic>> {
    let tokens = lexer::lex(text)?;
    parser::parse_program(text, &tokens)
}
