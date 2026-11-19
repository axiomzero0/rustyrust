// CEP:FILE: ir/thir/src/lib.rs
// CEP:WHAT: Library root for the typed high-level IR.
// CEP:WHY: Type checking output and MIR lowering input share one definition.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: none
// CEP:ASSUMES: none
// CEP:COST: cold; type definitions only.
// CEP:EVIDENCE: cargo build -p rr-thir.
//! Typed high-level IR produced by type checking.
#![deny(missing_docs)]
use rr_hir::hir::Ty;

/// A typed function signature.
#[derive(Debug, Clone)]
pub struct FnSig {
    /// Function name.
    pub name: String,
    /// Parameter types in order.
    pub params: Vec<Ty>,
    /// Return type.
    pub ret: Ty,
}

/// A typed program.
#[derive(Debug, Clone)]
pub struct TypedProgram {
    /// Signatures of every function, including main.
    pub sigs: Vec<FnSig>,
    /// Typed function bodies in declaration order.
    pub fns: Vec<TypedFn>,
}

/// A typed function body.
#[derive(Debug, Clone)]
pub struct TypedFn {
    /// Function name.
    pub name: String,
    /// Parameter names in order.
    pub params: Vec<String>,
    /// Return type.
    pub ret: Ty,
    /// Typed body statements.
    pub body: Vec<TStmt>,
}

/// A typed statement.
#[derive(Debug, Clone)]
pub enum TStmt {
    /// A binding with a fully inferred type.
    Let {
        /// Binding name.
        name: String,
        /// Inferred or annotated type.
        ty: Ty,
        /// Initializer.
        init: Option<TExpr>,
    },
    /// An assignment to an existing local.
    Assign {
        /// Target name.
        place: String,
        /// Assigned value.
        value: TExpr,
    },
    /// An expression evaluated for effect.
    Expr(TExpr),
    /// An early return.
    Return(Option<TExpr>),
    /// A while loop with a typed condition.
    While {
        /// Condition.
        cond: TExpr,
        /// Body statements.
        body: Vec<TStmt>,
    },
}

/// A typed expression.
#[derive(Debug, Clone)]
pub enum TExpr {
    /// An integer literal.
    Int(i64),
    /// A boolean literal.
    Bool(bool),
    /// The unit value.
    Unit,
    /// A local variable reference with its resolved type.
    Var {
        /// Variable name.
        name: String,
        /// Resolved type.
        ty: Ty,
    },
    /// A unary operation with its result type.
    Unary {
        /// Operator.
        op: rr_hir::hir::UnOp,
        /// Operand.
        e: Box<TExpr>,
        /// Result type.
        ty: Ty,
    },
    /// A binary operation with its result type.
    Binary {
        /// Operator.
        op: rr_hir::hir::BinOp,
        /// Left operand.
        l: Box<TExpr>,
        /// Right operand.
        r: Box<TExpr>,
        /// Result type.
        ty: Ty,
    },
    /// A call to a resolved function.
    Call {
        /// Callee name.
        callee: String,
        /// Typed arguments.
        args: Vec<TExpr>,
        /// Result type.
        ty: Ty,
    },
    /// A conditional with both branches typed.
    If {
        /// Condition.
        cond: Box<TExpr>,
        /// Then statements.
        then: Vec<TStmt>,
        /// Then value when present.
        then_val: Option<Box<TExpr>>,
        /// Else statements.
        els: Vec<TStmt>,
        /// Else value when present.
        els_val: Option<Box<TExpr>>,
        /// Result type.
        ty: Ty,
    },
    /// A block with statements and a trailing value.
    Block {
        /// Statements.
        stmts: Vec<TStmt>,
        /// Trailing value when present.
        val: Option<Box<TExpr>>,
        /// Result type.
        ty: Ty,
    },
}

impl TExpr {
    /// CEP:WHAT: Returns the result type of the expression.
    /// CEP:WHY: Lowering and diagnostics need the type at each node.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: Every typed node carries its type.
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_thir::tests::types_are_present.
    pub fn ty(&self) -> Ty {
        match self {
            TExpr::Int(_) => Ty::I64,
            TExpr::Bool(_) => Ty::Bool,
            TExpr::Unit => Ty::Unit,
            TExpr::Var { ty, .. } => *ty,
            TExpr::Unary { ty, .. } | TExpr::Binary { ty, .. } | TExpr::Call { ty, .. } => *ty,
            TExpr::If { ty, .. } | TExpr::Block { ty, .. } => *ty,
        }
    }
}

/// Sample assertion verifying type presence on typed nodes.
#[cfg(test)]
mod tests {
    use super::*;

    /// CEP:WHAT: Verifies that typed nodes carry their types.
    /// CEP:WHY: Lowering depends on node-level types.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-thir.
    #[test]
    fn types_are_present() {
        let e = TExpr::Int(SAMPLE_LITERAL);
        assert_eq!(e.ty(), Ty::I64);
        let b = TExpr::Block { stmts: Vec::new(), val: None, ty: Ty::Unit };
        assert_eq!(b.ty(), Ty::Unit);
        let v = TExpr::Var { name: "x".into(), ty: Ty::I64 };
        assert_eq!(v.ty(), Ty::I64);
    }
}
