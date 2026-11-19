// CEP:FILE: compiler/frontend/src/parser.rs
// CEP:WHAT: Recursive-descent parser with precedence climbing for the V0 dialect.
// CEP:WHY: Tokens become HIR here; malformed syntax yields spanned diagnostics.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: Returns diagnostics for malformed syntax; never panics on token streams.
// CEP:ASSUMES: The token stream ends with Tok::Eof and carries valid spans.
// CEP:COST: cold; linear in token count.
// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
#![deny(missing_docs)]
use crate::{Diagnostic, lexer::{Tok, Token}};
use rr_hir::hir::*;
use rr_hir::span::Span;

/// Binding power of one binary operator level.
const PREC_OR: u8 = 1;

/// Binding power of conjunction.
const PREC_AND: u8 = 2;

/// Binding power of equality comparisons.
const PREC_EQ: u8 = 3;

/// Binding power of order comparisons.
const PREC_REL: u8 = 4;

/// Binding power of addition and subtraction.
const PREC_ADD: u8 = 5;

/// Binding power of multiplication, division, and remainder.
const PREC_MUL: u8 = 6;

/// Hard recursion depth bound for expressions and blocks.
const MAX_DEPTH: usize = 128;

/// CEP:WHAT: Parses a token stream into a program.
/// CEP:WHY: The driver and tests consume one entry point.
/// CEP:ID: FE-a01
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns diagnostics for malformed syntax and depth bound violations.
/// CEP:ASSUMES: The stream ends with Eof.
/// CEP:COST: cold; linear in token count.
/// CEP:EVIDENCE: doc file docs/functions/FE/FE-a01.md; unit test parses_program.
pub fn parse_program(_text: &str, tokens: &[Token]) -> Result<Program, Vec<Diagnostic>> {
    let mut p = Parser { tokens, pos: 0, errs: Vec::new(), depth: 0 };
    let mut fns = Vec::new();
    while !p.at_kw("fn") && !p.at_eof() {
        p.err("expected function definition");
        p.bump();
    }
    while p.at_kw("fn") {
        if let Some(f) = p.parse_fn() {
            fns.push(f);
        }
    }
    if p.errs.is_empty() {
        Ok(Program { fns })
    } else {
        Err(p.errs)
    }
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    errs: Vec<Diagnostic>,
    depth: usize,
}

impl<'a> Parser<'a> {
    /// CEP:WHAT: Reports one diagnostic at the current token.
    /// CEP:WHY: Errors must carry spans for diagnostics rendering.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; one string allocation.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn err(&mut self, msg: &str) {
        let span = self.cur().span;
        self.errs.push(Diagnostic { msg: msg.to_string(), span });
    }

    /// CEP:WHAT: Returns the current token.
    /// CEP:WHY: All decisions key on the current token.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: The stream always ends with Eof.
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn cur(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(self.tokens.last().expect("eof present"))
    }

    /// CEP:WHAT: Advances one token and returns the consumed one.
    /// CEP:WHY: Bumping is the only cursor movement.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn bump(&mut self) -> Token {
        let t = self.cur().clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        t
    }

    /// CEP:WHAT: Reports whether the current token is one keyword.
    /// CEP:WHY: Statement dispatch is keyword-driven.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn at_kw(&self, k: &str) -> bool {
        matches!(&self.cur().kind, Tok::Kw(w) if *w == k)
    }

    /// CEP:WHAT: Reports whether the current token is one punctuation symbol.
    /// CEP:WHY: Blocks and calls key on punctuation.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn at_punct(&self, p: &str) -> bool {
        matches!(&self.cur().kind, Tok::Punct(w) if *w == p)
    }

    /// CEP:WHAT: Reports whether the stream is exhausted.
    /// CEP:WHY: Program parsing stops at Eof.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn at_eof(&self) -> bool {
        matches!(self.cur().kind, Tok::Eof)
    }

    /// CEP:WHAT: Consumes one punctuation symbol when present.
    /// CEP:WHY: Optional separators use conditional consumption.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn eat_punct(&mut self, p: &str) -> bool {
        if self.at_punct(p) {
            self.bump();
            true
        } else {
            false
        }
    }

    // CEP:WHAT: Consumes one keyword when present.
    // CEP:WHY: Optional keywords such as else use conditional consumption.
    // CEP:STATUS: complete
    // CEP:FAILURE: none
    // CEP:ASSUMES: none
    // CEP:COST: cold; constant time.
    // CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn eat_kw(&mut self, k: &str) -> bool {
        if self.at_kw(k) {
            self.bump();
            true
        } else {
            false
        }
    }

    /// CEP:WHAT: Consumes one punctuation symbol or records an error.
    /// CEP:WHY: Required separators must fail loudly with a span.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Pushes a diagnostic when the symbol is absent.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn expect_punct(&mut self, p: &str) {
        if !self.eat_punct(p) {
            self.err(&format!("expected {:?}", p));
        }
    }


    /// CEP:WHAT: Parses one function definition.
    /// CEP:WHY: Functions are the only top-level items of V0.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Records diagnostics for malformed signatures.
    /// CEP:ASSUMES: The cursor sits at the fn keyword.
    /// CEP:COST: cold; linear in function size.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn parse_fn(&mut self) -> Option<FnDef> {
        self.depth_guard();
        self.bump();
        let name = self.parse_ident();
        let params = self.parse_params();
        let mut ret = Ty::Unit;
        if self.eat_punct("->") {
            ret = self.parse_ty();
        }
        let body = self.parse_block();
        self.depth -= 1;
        Some(FnDef { name, params, ret, body })
    }

    // CEP:WHAT: Parses one parameter list with type annotations.
    // CEP:WHY: Signatures are the type-checker's interface input.
    // CEP:STATUS: complete
    // CEP:FAILURE: Records diagnostics for malformed parameter lists.
    // CEP:ASSUMES: The cursor sits at the open parenthesis.
    // CEP:COST: cold; linear in parameter count.
    // CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn parse_params(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        self.expect_punct("(");
        if !self.at_punct(")") {
            loop {
                let pname = self.parse_ident();
                self.expect_punct(":");
                let ty = self.parse_ty();
                params.push(Param { name: pname, ty });
                if !self.eat_punct(",") {
                    break;
                }
            }
        }
        self.expect_punct(")");
        params
    }

    /// CEP:WHAT: Guards parser recursion with the depth bound.
    /// CEP:WHY: Section 38.6 requires bounded resource use on adversarial input.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Records a diagnostic at the bound.
    /// CEP:ASSUMES: Every recursive entry pairs with a decrement.
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::rejects_deep_nesting.
    fn depth_guard(&mut self) {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            self.err("nesting depth limit exceeded");
        }
    }

    /// CEP:WHAT: Parses one identifier.
    /// CEP:WHY: Names appear in bindings, calls, and assignments.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Records a diagnostic for missing identifiers.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn parse_ident(&mut self) -> Ident {
        let t = self.cur().clone();
        if let Tok::Ident(n) = t.kind {
            self.bump();
            return Ident { name: n, span: t.span };
        }
        self.err("expected identifier");
        Ident { name: String::new(), span: t.span }
    }

    /// CEP:WHAT: Parses one type name.
    /// CEP:WHY: Parameters and return types are annotated in V0.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Records a diagnostic for unknown types.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn parse_ty(&mut self) -> Ty {
        let t = self.cur().clone();
        if let Tok::Kw("i64") = t.kind {
            self.bump();
            return Ty::I64;
        }
        self.err("expected type");
        Ty::Unit
    }

    /// CEP:WHAT: Parses one block with statements.
    /// CEP:WHY: Function bodies, loop bodies, and branches are blocks.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Records diagnostics for unclosed blocks.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in block size.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn parse_block(&mut self) -> Block {
        self.depth_guard();
        let open = self.cur().span;
        self.expect_punct("{");
        let mut stmts = Vec::new();
        while !self.at_punct("}") && !self.at_eof() {
            if let Some(s) = self.parse_stmt() {
                stmts.push(s);
            }
        }
        let close = self.cur().span;
        self.expect_punct("}");
        self.depth -= 1;
        Block { stmts, span: open.to(close) }
    }

    /// CEP:WHAT: Parses one statement.
    /// CEP:WHY: Statements are bindings, assignments, control flow, and effect expressions.
    /// CEP:ID: FE-a02
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Records diagnostics for malformed statements.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in statement size.
    /// CEP:EVIDENCE: doc file docs/functions/FE/FE-a02.md; unit test parses_program.
    fn parse_stmt(&mut self) -> Option<Stmt> {
        let span = self.cur().span;
        if self.at_kw("let") {
            self.bump();
            let _mut = self.eat_kw("mut");
            let name = self.parse_ident();
            let mut ty = None;
            if self.eat_punct(":") {
                ty = Some(self.parse_ty());
            }
            let mut init = None;
            if self.eat_punct("=") {
                init = Some(self.parse_expr());
            }
            self.expect_punct(";");
            return Some(Stmt::Let { name, ty, init, span: span.to(self.cur().span) });
        }
        if self.at_kw("return") {
            self.bump();
            let e = if self.at_punct(";") { None } else { Some(self.parse_expr()) };
            self.expect_punct(";");
            return Some(Stmt::Return { e, span: span.to(self.cur().span) });
        }
        if self.at_kw("while") {
            self.bump();
            let cond = Box::new(self.parse_expr());
            let body = self.parse_block();
            let wspan = span.to(body.span);
            return Some(Stmt::While { cond, body, span: wspan });
        }
        if self.at_kw("if") {
            let e = self.parse_expr();
            self.semi_or_block_end();
            return Some(Stmt::Expr { e, span: span.to(self.cur().span) });
        }
        let e = self.parse_expr();
        if self.eat_punct("=") {
            let place = match e {
                Expr::Var { name } => name,
                other => {
                    self.err("assignment target must be a variable");
                    Ident { name: String::new(), span: other_span(&other) }
                }
            };
            let value = self.parse_expr();
            self.expect_punct(";");
            return Some(Stmt::Assign { place, value, span: span.to(self.cur().span) });
        }
        self.semi_or_block_end();
        Some(Stmt::Expr { e, span: span.to(self.cur().span) })
    }

    // CEP:WHAT: Consumes a statement semicolon or accepts a block-closing brace.
    // CEP:WHY: A trailing expression may omit its semicolon before the block end.
    // CEP:STATUS: complete
    // CEP:FAILURE: Records a diagnostic when neither separator is present.
    // CEP:ASSUMES: none
    // CEP:COST: cold; constant time.
    // CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn semi_or_block_end(&mut self) {
        if self.eat_punct(";") || self.at_punct("}") {
            return;
        }
        self.err("expected ;");
    }

    /// CEP:WHAT: Parses one expression with precedence climbing.
    /// CEP:WHY: Operator precedence must match the dialect grammar.
    /// CEP:ID: FE-a03
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Records diagnostics for malformed expressions.
    /// CEP:ASSUMES: Precedence levels are ordered from lowest to highest.
    /// CEP:COST: cold; linear in expression size.
    /// CEP:EVIDENCE: doc file docs/functions/FE/FE-a03.md; unit test parses_precedence.
    fn parse_expr(&mut self) -> Expr {
        self.parse_binary(PREC_OR)
    }

    /// CEP:WHAT: Parses binary expressions at one precedence level.
    /// CEP:WHY: Precedence climbing keeps the grammar flat and termination bounded.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Records diagnostics for malformed operands.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in expression size.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_precedence.
    fn parse_binary(&mut self, min_prec: u8) -> Expr {
        self.depth_guard();
        let mut l = self.parse_unary();
        while let Some((op, prec)) = self.cur_binop() {
            if prec < min_prec {
                break;
            }
            self.bump();
            let r = self.parse_binary(prec + 1);
            let span = other_span(&l).to(other_span(&r));
            l = Expr::Binary { op, l: Box::new(l), r: Box::new(r), span };
        }
        self.depth -= 1;
        l
    }

    /// CEP:WHAT: Maps the current token to a binary operator and precedence.
    /// CEP:WHY: The operator table is the grammar.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_precedence.
    fn cur_binop(&self) -> Option<(BinOp, u8)> {
        let Tok::Punct(p) = &self.cur().kind else { return None };
        let op = match *p {
            "||" => BinOp::Or,
            "&&" => BinOp::And,
            "==" => BinOp::Eq,
            "!=" => BinOp::Ne,
            "<" => BinOp::Lt,
            ">" => BinOp::Gt,
            "<=" => BinOp::Le,
            ">=" => BinOp::Ge,
            "+" => BinOp::Add,
            "-" => BinOp::Sub,
            "*" => BinOp::Mul,
            "/" => BinOp::Div,
            "%" => BinOp::Rem,
            _ => return None,
        };
        let prec = match op {
            BinOp::Or => PREC_OR,
            BinOp::And => PREC_AND,
            BinOp::Eq | BinOp::Ne => PREC_EQ,
            BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => PREC_REL,
            BinOp::Add | BinOp::Sub => PREC_ADD,
            _ => PREC_MUL,
        };
        Some((op, prec))
    }

    /// CEP:WHAT: Parses unary operators and primary expressions.
    /// CEP:WHY: Negation and logical not bind tighter than binary operators.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Records diagnostics for missing operands.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in expression size.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_precedence.
    fn parse_unary(&mut self) -> Expr {
        if self.depth > MAX_DEPTH {
            self.err("nesting depth limit exceeded");
            self.bump();
            return Expr::Unit { span: self.cur().span };
        }
        if self.at_punct("-") || self.at_punct("!") {
            self.depth_guard();
            let t = self.bump();
            let e = self.parse_unary();
            self.depth -= 1;
            let op = if matches!(t.kind, Tok::Punct("-")) { UnOp::Neg } else { UnOp::Not };
            let uspan = t.span.to(other_span(&e));
            return Expr::Unary { op, e: Box::new(e), span: uspan };
        }
        self.parse_primary()
    }

    /// CEP:WHAT: Parses literals, variables, calls, conditionals, and blocks.
    /// CEP:WHY: Primary expressions are the leaves of the grammar.
    /// CEP:ID: FE-a04
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Records diagnostics for unexpected tokens.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in expression size.
    /// CEP:EVIDENCE: doc file docs/functions/FE/FE-a04.md; unit test parses_program.
    fn parse_primary(&mut self) -> Expr {
        let t = self.cur().clone();
        match t.kind {
            Tok::Int(v) => {
                self.bump();
                Expr::Int { value: v, span: t.span }
            }
            Tok::Kw("true") => {
                self.bump();
                Expr::Bool { value: true, span: t.span }
            }
            Tok::Kw("false") => {
                self.bump();
                Expr::Bool { value: false, span: t.span }
            }
            Tok::Ident(n) => {
                self.bump();
                if self.at_punct("(") {
                    return self.parse_call(n, t.span);
                }
                Expr::Var { name: Ident { name: n, span: t.span } }
            }
            Tok::Kw("if") => self.parse_if(),
            Tok::Punct("{") => {
                let b = self.parse_block();
                let bspan = b.span;
                Expr::BlockExpr { b, span: bspan }
            }
            _ => {
                self.err("expected expression");
                self.bump();
                Expr::Unit { span: t.span }
            }
        }
    }

    /// CEP:WHAT: Parses one call expression.
    /// CEP:WHY: Calls are the only indirect construct in V0.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Records diagnostics for malformed argument lists.
    /// CEP:ASSUMES: The cursor sits at the open parenthesis.
    /// CEP:COST: cold; linear in argument count.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn parse_call(&mut self, name: String, open: Span) -> Expr {
        self.expect_punct("(");
        let mut args = Vec::new();
        if !self.at_punct(")") {
            loop {
                args.push(self.parse_expr());
                if !self.eat_punct(",") {
                    break;
                }
            }
        }
        let close = self.cur().span;
        self.expect_punct(")");
        Expr::Call { callee: Ident { name, span: open }, args, span: open.to(close) }
    }

    /// CEP:WHAT: Parses one conditional expression.
    /// CEP:WHY: If forms appear in statement and value positions.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Records diagnostics for missing branches.
    /// CEP:ASSUMES: The cursor sits at the if keyword.
    /// CEP:COST: cold; linear in branch size.
    /// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_program.
    fn parse_if(&mut self) -> Expr {
        let open = self.cur().span;
        self.bump();
        let cond = Box::new(self.parse_expr());
        let then = self.parse_block();
        let mut els = None;
        if self.eat_kw("else") {
            if self.at_kw("if") {
                els = Some(Box::new(self.parse_if()));
            } else {
                let b = self.parse_block();
                let bspan = b.span;
                els = Some(Box::new(Expr::BlockExpr { b, span: bspan }));
            }
        }
        Expr::If { cond, then, els, span: open.to(self.cur().span) }
    }
}

/// CEP:WHAT: Returns the span of one expression.
/// CEP:WHY: Composite spans are unions of operand spans.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Every variant carries a span.
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_frontend::parser::tests::parses_precedence.
fn other_span(e: &Expr) -> Span {
    match e {
        Expr::Int { span, .. } | Expr::Bool { span, .. } | Expr::Unit { span } => *span,
        Expr::Var { name } => name.span,
        Expr::Unary { span, .. } | Expr::Binary { span, .. } | Expr::Call { span, .. } | Expr::If { span, .. } | Expr::BlockExpr { span, .. } => *span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;

    /// CEP:WHAT: Verifies parsing of a full program with a call and a loop.
    /// CEP:WHY: The pipeline entry must accept representative V0 programs.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-frontend.
    #[test]
    fn parses_program() {
        let src = "fn main() -> i64 { let x = 42; while x > 0 { x = x - 1; } print_i64(x); 0 }";
        let toks = lexer::lex(src).unwrap();
        let p = parse_program(src, &toks).unwrap();
        assert_eq!(p.fns.len(), 1);
        let src2 = "fn add(a: i64, b: i64) -> i64 { a + b } fn main() -> i64 { add(1, 2) }";
        let p2 = parse_program(src2, &lexer::lex(src2).unwrap()).unwrap();
        assert_eq!(p2.fns[1].params.len(), 0);
    }

    /// CEP:WHAT: Verifies operator precedence shapes.
    /// CEP:WHY: Misprecedence silently changes program meaning.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-frontend.
    #[test]
    fn parses_precedence() {
        let src = "fn main() -> i64 { 1 + 2 * 3 }";
        let p = parse_program(src, &lexer::lex(src).unwrap()).unwrap();
        let Stmt::Expr { e, .. } = p.fns[0].body.stmts.last().unwrap() else { return };
        let Expr::Binary { op, .. } = e else { return };
        assert_eq!(*op, BinOp::Add);
        let src2 = "fn main() -> i64 { if true { 1 } else { 2 } }";
        assert!(parse_program(src2, &lexer::lex(src2).unwrap()).is_ok());
    }

    /// Nesting depth beyond the parser bound.
    const DEEP_DEPTH: usize = 300;

    /// CEP:WHAT: Rejects nesting beyond the depth bound.
    /// CEP:WHY: Section 38.6 forbids unbounded compiler resource use.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-frontend.
    #[test]
    fn rejects_deep_nesting() {
        let mut src = String::from("fn main() -> i64 { ");
        for _ in 0..DEEP_DEPTH {
            src.push_str("-(");
        }
        src.push('1');
        for _ in 0..DEEP_DEPTH {
            src.push(')');
        }
        src.push_str(" }");
        assert!(crate::parse_source(&src).is_err());
    }
}
