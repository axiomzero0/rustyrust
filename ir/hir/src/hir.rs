// CEP:FILE: ir/hir/src/hir.rs
// CEP:WHAT: Source syntax of the V0 dialect: functions, statements, expressions, and operators.
// CEP:WHY: The parser emits this syntax; the type checker consumes it before lowering.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: none
// CEP:ASSUMES: The dialect is the V0 subset documented in docs/architecture/RR_LANGUAGE_V0.md.
// CEP:COST: compile-time only; plain data.
// CEP:EVIDENCE: unit test rr_hir::hir::tests::syntax_smoke.
#![deny(missing_docs)]
use crate::span::Span;

/// Source-level types of the V0 dialect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    /// Signed 64-bit integer with wrapping semantics.
    I64,
    /// Boolean.
    Bool,
    /// The unit type.
    Unit,
}

impl Ty {
    /// CEP:WHAT: Renders the type name for diagnostics.
    /// CEP:WHY: Type errors must name the offending types.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; small allocation per call.
    /// CEP:EVIDENCE: unit test rr_hir::hir::tests::syntax_smoke.
    pub fn name(self) -> &'static str {
        match self {
            Ty::I64 => "i64",
            Ty::Bool => "bool",
            Ty::Unit => "unit",
        }
    }
}

/// An identifier with its source span.
#[derive(Debug, Clone)]
pub struct Ident {
    /// The identifier text.
    pub name: String,
    /// Span of the identifier.
    pub span: Span,
}

/// A parameter declaration.
#[derive(Debug, Clone)]
pub struct Param {
    /// Parameter name.
    pub name: Ident,
    /// Declared type; required in V0.
    pub ty: Ty,
}

/// A function definition.
#[derive(Debug, Clone)]
pub struct FnDef {
    /// Function name.
    pub name: Ident,
    /// Parameters in declaration order.
    pub params: Vec<Param>,
    /// Return type; defaults to unit when absent.
    pub ret: Ty,
    /// Body block.
    pub body: Block,
}

/// A parsed program.
#[derive(Debug, Clone)]
pub struct Program {
    /// Top-level functions.
    pub fns: Vec<FnDef>,
}

/// A brace-delimited block of statements.
#[derive(Debug, Clone)]
pub struct Block {
    /// Statements in source order.
    pub stmts: Vec<Stmt>,
    /// Span of the block.
    pub span: Span,
}

/// A statement of the V0 dialect.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// A local binding with optional type annotation.
    Let {
        /// Binding name.
        name: Ident,
        /// Declared type when annotated.
        ty: Option<Ty>,
        /// Initializer when present.
        init: Option<Expr>,
        /// Span of the statement.
        span: Span,
    },
    /// An assignment to a local variable.
    Assign {
        /// Assignment target.
        place: Ident,
        /// Assigned value.
        value: Expr,
        /// Span of the statement.
        span: Span,
    },
    /// An expression evaluated for effect.
    Expr {
        /// The expression.
        e: Expr,
        /// Span of the statement.
        span: Span,
    },
    /// An early return with optional value.
    Return {
        /// Returned value when present.
        e: Option<Expr>,
        /// Span of the statement.
        span: Span,
    },
    /// A while loop.
    While {
        /// Loop condition.
        cond: Box<Expr>,
        /// Loop body.
        body: Block,
        /// Span of the statement.
        span: Span,
    },
}

/// A binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    /// Wrapping addition.
    Add,
    /// Wrapping subtraction.
    Sub,
    /// Wrapping multiplication.
    Mul,
    /// Truncating division; zero divisor traps.
    Div,
    /// Truncating remainder; zero divisor traps.
    Rem,
    /// Equality comparison.
    Eq,
    /// Inequality comparison.
    Ne,
    /// Less-than comparison.
    Lt,
    /// Greater-than comparison.
    Gt,
    /// Less-or-equal comparison.
    Le,
    /// Greater-or-equal comparison.
    Ge,
    /// Short-circuit conjunction.
    And,
    /// Short-circuit disjunction.
    Or,
}

impl BinOp {
    /// CEP:WHAT: Reports whether the operator yields i64 from i64 operands.
    /// CEP:WHY: Type checking groups arithmetic and comparison operators differently.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: compile-time only; plain data.
    /// CEP:EVIDENCE: unit test rr_hir::hir::tests::syntax_smoke.
    pub fn is_arith(self) -> bool {
        matches!(self, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem)
    }

    /// CEP:WHAT: Reports whether the operator compares two i64 values.
    /// CEP:WHY: Comparisons have a fixed bool result type.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: compile-time only; plain data.
    /// CEP:EVIDENCE: unit test rr_hir::hir::tests::syntax_smoke.
    pub fn is_cmp(self) -> bool {
        matches!(self, BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge)
    }

    /// CEP:WHAT: Renders the operator for diagnostics.
    /// CEP:WHY: Parse errors must show the operator text.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; small allocation per call.
    /// CEP:EVIDENCE: unit test rr_hir::hir::tests::syntax_smoke.
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Gt => ">",
            BinOp::Le => "<=",
            BinOp::Ge => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        }
    }
}

/// A unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    /// Arithmetic negation with wrapping semantics.
    Neg,
    /// Logical negation.
    Not,
}

/// An expression of the V0 dialect.
#[derive(Debug, Clone)]
pub enum Expr {
    /// An integer literal in range for i64.
    Int {
        /// Literal value.
        value: i64,
        /// Span of the literal.
        span: Span,
    },
    /// A boolean literal.
    Bool {
        /// Literal value.
        value: bool,
        /// Span of the literal.
        span: Span,
    },
    /// The unit literal.
    Unit {
        /// Span of the literal.
        span: Span,
    },
    /// A variable reference.
    Var {
        /// Referenced name.
        name: Ident,
    },
    /// A unary operation.
    Unary {
        /// The operator.
        op: UnOp,
        /// The operand.
        e: Box<Expr>,
        /// Span of the expression.
        span: Span,
    },
    /// A binary operation.
    Binary {
        /// The operator.
        op: BinOp,
        /// Left operand.
        l: Box<Expr>,
        /// Right operand.
        r: Box<Expr>,
        /// Span of the expression.
        span: Span,
    },
    /// A call expression.
    Call {
        /// Callee name.
        callee: Ident,
        /// Arguments in source order.
        args: Vec<Expr>,
        /// Span of the expression.
        span: Span,
    },
    /// A conditional expression.
    If {
        /// Condition.
        cond: Box<Expr>,
        /// Then block.
        then: Block,
        /// Else branch; required when the expression value is used.
        els: Option<Box<Expr>>,
        /// Span of the expression.
        span: Span,
    },
    /// A block expression.
    BlockExpr {
        /// The block.
        b: Block,
        /// Span of the expression.
        span: Span,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CEP:WHAT: Verifies operator classification and type naming.
    /// CEP:WHY: Type checking depends on operator classes.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-hir.
    #[test]
    fn syntax_smoke() {
        assert!(BinOp::Add.is_arith());
        assert!(BinOp::Eq.is_cmp());
        assert!(BinOp::And.is_cmp() == false);
        assert_eq!(Ty::I64.name(), "i64");
    }
}
