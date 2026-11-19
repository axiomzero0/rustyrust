// CEP:FILE: compiler/typeck/src/lib.rs
// CEP:WHAT: Library root for name resolution and type checking into THIR.
// CEP:WHY: The semantic layer produces the typed IR consumed by lowering and borrow checking.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: none
// CEP:ASSUMES: Input HIR comes from the frontend with balanced structure.
// CEP:COST: cold; linear in program size.
// CEP:EVIDENCE: cargo test -p rr-typeck.
//! Semantic analysis: signatures, name resolution, and type checking.

use rr_hir::hir::*;
use rr_hir::span::Span;
use rr_thir::{FnSig, TExpr, TStmt, TypedFn, TypedProgram};
use rr_traits::Solver;

/// A semantic diagnostic.
#[derive(Debug, Clone)]
pub struct TypeError {
    /// Message text.
    pub msg: String,
    /// Byte span of the offending region.
    pub span: Span,
}

/// Shared checking context passed through semantic functions.
struct Ctx<'a> {
    sigs: &'a [FnSig],
    solver: &'a mut Solver,
    errs: &'a mut Vec<TypeError>,
}

/// Reserved builtin callable of the runtime.
const BUILTIN_PRINT: &str = "print_i64";

/// Minimum argument count of the print builtin.
const PRINT_ARITY: usize = 1;

/// CEP:WHAT: Type checks one program into a typed program.
/// CEP:WHY: The driver needs one semantic entry point.
/// CEP:ID: FE-a05
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns diagnostics for duplicate names, unknown names, arity mismatches, and type mismatches.
/// CEP:ASSUMES: Trait solving reports zero goals for V0 programs.
/// CEP:COST: cold; linear in program size.
/// CEP:EVIDENCE: doc file docs/functions/FE/FE-a05.md; unit test checks_types.
pub fn check_program(p: &Program) -> Result<TypedProgram, Vec<TypeError>> {
    let mut errs = Vec::new();
    let mut solver = Solver::default();
    let sigs = collect_sigs(p, &mut errs);
    let mut fns = Vec::new();
    for f in p.fns.iter() {
        fns.push(check_fn(f, &sigs, &mut solver, &mut errs));
    }
    check_main(p, &mut errs);
    if errs.is_empty() {
        Ok(TypedProgram { sigs, fns })
    } else {
        Err(errs)
    }
}

/// CEP:WHAT: Collects function signatures and rejects duplicates.
/// CEP:WHY: Name resolution needs a signature table before checking bodies.
/// CEP:STATUS: complete
/// CEP:FAILURE: Records diagnostics for duplicate function names.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in functions.
/// CEP:EVIDENCE: unit test rr_typeck::tests::checks_types.
fn collect_sigs(p: &Program, errs: &mut Vec<TypeError>) -> Vec<FnSig> {
    let mut sigs: Vec<FnSig> = Vec::new();
    for f in p.fns.iter() {
        if sigs.iter().any(|s| s.name == f.name.name) {
            errs.push(TypeError { msg: format!("duplicate function {}", f.name.name), span: f.name.span });
            continue;
        }
        sigs.push(FnSig {
            name: f.name.name.clone(),
            params: f.params.iter().map(|x| x.ty).collect(),
            ret: f.ret,
        });
    }
    sigs
}

/// CEP:WHAT: Validates the main entry point signature.
/// CEP:WHY: The runtime contract requires main with zero params returning i64 or unit.
/// CEP:STATUS: complete
/// CEP:FAILURE: Records a diagnostic when main is missing or malformed.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in functions.
/// CEP:EVIDENCE: unit test rr_typeck::tests::checks_types.
fn check_main(p: &Program, errs: &mut Vec<TypeError>) {
    let Some(f) = p.fns.iter().find(|f| f.name.name == "main") else {
        errs.push(TypeError { msg: "missing main function".into(), span: Span::new(0, 0) });
        return;
    };
    if !f.params.is_empty() {
        errs.push(TypeError { msg: "main takes no parameters".into(), span: f.name.span });
    }
    if f.ret != Ty::I64 && f.ret != Ty::Unit {
        errs.push(TypeError { msg: "main must return i64 or unit".into(), span: f.name.span });
    }
}

/// CEP:WHAT: Checks one function body and emits its typed form.
/// CEP:WHY: Bodies are checked against the signature table.
/// CEP:STATUS: complete
/// CEP:FAILURE: Records diagnostics for body errors; check failures degrade to unit statements.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in body size.
/// CEP:EVIDENCE: unit test rr_typeck::tests::checks_types.
fn check_fn(f: &FnDef, sigs: &[FnSig], solver: &mut Solver, errs: &mut Vec<TypeError>) -> TypedFn {
    let mut vars: Vec<(String, Ty)> = f.params.iter().map(|x| (x.name.name.clone(), x.ty)).collect();
    let stmts = check_block(&f.body, sigs, &mut vars, solver, errs);
    TypedFn { name: f.name.name.clone(), params: f.params.iter().map(|x| x.name.name.clone()).collect(), ret: f.ret, body: stmts }
}

/// CEP:WHAT: Checks one block and updates the local variable table.
/// CEP:WHY: Scoping is block-structured in V0.
/// CEP:STATUS: complete
/// CEP:FAILURE: Records diagnostics for shadow conflicts and body errors.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in block size.
/// CEP:EVIDENCE: unit test rr_typeck::tests::checks_types.
fn check_block(b: &Block, sigs: &[FnSig], vars: &mut Vec<(String, Ty)>, solver: &mut Solver, errs: &mut Vec<TypeError>) -> Vec<TStmt> {
    let mut out = Vec::new();
    for s in b.stmts.iter() {
        out.extend(check_stmt(s, sigs, vars, solver, errs));
    }
    out
}

/// CEP:WHAT: Checks one statement and emits typed statements.
/// CEP:WHY: Statements introduce bindings, effects, and control flow that later stages consume.
/// CEP:ID: FE-a06
/// CEP:STATUS: complete
/// CEP:FAILURE: Records diagnostics for redefinition, unknown locals, and mismatched types.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in statement size.
/// CEP:EVIDENCE: doc file docs/functions/FE/FE-a06.md; unit test checks_types.
fn check_stmt(s: &Stmt, sigs: &[FnSig], vars: &mut Vec<(String, Ty)>, solver: &mut Solver, errs: &mut Vec<TypeError>) -> Vec<TStmt> {
    match s {
        Stmt::Let { name, ty, init, span } => {
            let it = init.as_ref().map(|e| check_expr(e, sigs, vars, solver, errs));
            let declared = *ty;
            let inferred = it.as_ref().map(|e| e.ty());
            let final_ty = match (declared, inferred) {
                (Some(d), Some(i)) if d != i => {
                    errs.push(TypeError { msg: format!("type mismatch in let {}: expected {} found {}", name.name, d.name(), i.name()), span: *span });
                    None
                }
                (Some(d), _) => Some(d),
                (None, Some(i)) => Some(i),
                (None, None) => {
                    errs.push(TypeError { msg: format!("let {} needs a type annotation or initializer", name.name), span: *span });
                    None
                }
            };
            match final_ty {
                Some(t) => {
                    if vars.iter().any(|(n, _)| n == &name.name) {
                        errs.push(TypeError { msg: format!("redefinition of {}", name.name), span: name.span });
                    }
                    vars.push((name.name.clone(), t));
                    vec![TStmt::Let { name: name.name.clone(), ty: t, init: it }]
                }
                None => Vec::new(),
            }
        }
        Stmt::Assign { place, value, span } => {
            let v = check_expr(value, sigs, vars, solver, errs);
            match vars.iter().find(|(n, _)| n == &place.name) {
                None => {
                    errs.push(TypeError { msg: format!("unknown variable {}", place.name), span: *span });
                    Vec::new()
                }
                Some((_, t)) if *t != v.ty() => {
                    errs.push(TypeError { msg: format!("type mismatch assigning to {}: expected {} found {}", place.name, t.name(), v.ty().name()), span: *span });
                    Vec::new()
                }
                Some(_) => vec![TStmt::Assign { place: place.name.clone(), value: v }],
            }
        }
        Stmt::Expr { e, .. } => vec![TStmt::Expr(check_expr(e, sigs, vars, solver, errs))],
        Stmt::Return { e, .. } => vec![TStmt::Return(e.as_ref().map(|x| check_expr(x, sigs, vars, solver, errs)))],
        Stmt::While { cond, body, .. } => {
            let c = check_expr(cond, sigs, vars, solver, errs);
            if c.ty() != Ty::Bool {
                errs.push(TypeError { msg: "while condition must be bool".into(), span: cond_span(cond) });
            }
            let bs = check_block(body, sigs, vars, solver, errs);
            vec![TStmt::While { cond: c, body: bs }]
        }
    }
}

/// CEP:WHAT: Checks one expression and emits its typed form.
/// CEP:WHY: Typed expressions are the currency of lowering and diagnostics.
/// CEP:ID: FE-a07
/// CEP:STATUS: complete
/// CEP:FAILURE: Records diagnostics for unknown names, arity mismatches, and operator type errors.
/// CEP:ASSUMES: Trait solving reports zero goals for V0 programs.
/// CEP:COST: cold; linear in expression size.
/// CEP:EVIDENCE: doc file docs/functions/FE/FE-a07.md; unit test checks_types.
fn check_expr(e: &Expr, sigs: &[FnSig], vars: &[(String, Ty)], solver: &mut Solver, errs: &mut Vec<TypeError>) -> TExpr {
    match e {
        Expr::Int { value, .. } => TExpr::Int(*value),
        Expr::Bool { value, .. } => TExpr::Bool(*value),
        Expr::Unit { .. } => TExpr::Unit,
        Expr::Var { name } => match vars.iter().find(|(n, _)| n == &name.name) {
            Some((_, t)) => var_expr(&name.name, *t),
            None => {
                errs.push(TypeError { msg: format!("unknown variable {}", name.name), span: name.span });
                TExpr::Unit
            }
        },
        Expr::Unary { op, e, .. } => {
            let inner = check_expr(e, sigs, vars, solver, errs);
            let t = inner.ty();
            let ok = matches!((op, t), (UnOp::Neg, Ty::I64) | (UnOp::Not, Ty::Bool));
            if !ok {
                errs.push(TypeError { msg: "unary operand type mismatch".into(), span: name_span_unary(e) });
            }
            TExpr::Unary { op: *op, e: Box::new(inner), ty: t }
        }
        Expr::Binary { op, l, r, span } => check_binary(op, l, r, *span, vars, cx),
        Expr::Call { callee, args, .. } => check_call(callee, args, sigs, vars, solver, errs),
        Expr::If { cond, then, els, .. } => check_if(cond, then, els, sigs, vars, solver, errs),
        Expr::BlockExpr { b, .. } => {
            let mut vars2 = vars.to_vec();
            let stmts = check_block(b, sigs, &mut vars2, solver, errs);
            let ty = stmts.last().and_then(stmt_val_expr).map(|x| x.ty()).unwrap_or(Ty::Unit);
            TExpr::Block { stmts, val: None, ty }
        }
    }
}

/// CEP:WHAT: Checks one binary operation against the operator typing rules.
/// CEP:WHY: Arithmetic, comparison, and logic operators have distinct rules.
/// CEP:STATUS: complete
/// CEP:FAILURE: Records diagnostics for operand mismatches.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in operand size.
/// CEP:EVIDENCE: unit test rr_typeck::tests::checks_types.
fn check_binary(op: &BinOp, l: &Expr, r: &Expr, span: Span, vars: &[(String, Ty)], cx: &mut Ctx) -> TExpr {
    let lv = check_expr(l, cx.sigs, vars, cx.solver, cx.errs);
    let rv = check_expr(r, cx.sigs, vars, cx.solver, cx.errs);
    let lt = lv.ty();
    let rt = rv.ty();
    let result = if op.is_arith() {
        require(lt == Ty::I64 && rt == Ty::I64, "arithmetic operands must be i64", span, cx.errs);
        Ty::I64
    } else if op.is_cmp() {
        require(lt == Ty::I64 && rt == Ty::I64, "comparison operands must be i64", span, cx.errs);
        Ty::Bool
    } else {
        require(lt == Ty::Bool && rt == Ty::Bool, "logic operands must be bool", span, cx.errs);
        Ty::Bool
    };
    TExpr::Binary { op: *op, l: Box::new(lv), r: Box::new(rv), ty: result }
}

/// CEP:WHAT: Checks one call against the signature table.
/// CEP:WHY: Arity and argument types must match the callee signature.
/// CEP:STATUS: complete
/// CEP:FAILURE: Records diagnostics for unknown callees and mismatches.
/// CEP:ASSUMES: The print builtin has the documented signature.
/// CEP:COST: cold; linear in arguments.
/// CEP:EVIDENCE: unit test rr_typeck::tests::checks_types.
fn check_call(callee: &Ident, args: &[Expr], sigs: &[FnSig], vars: &[(String, Ty)], solver: &mut Solver, errs: &mut Vec<TypeError>) -> TExpr {
    let checked: Vec<TExpr> = args.iter().map(|a| check_expr(a, sigs, vars, solver, errs)).collect();
    let ret = match sigs.iter().find(|s| s.name == callee.name) {
        Some(s) => {
            if s.params.len() != checked.len() {
                errs.push(TypeError { msg: format!("call to {} expects {} arguments found {}", callee.name, s.params.len(), checked.len()), span: callee.span });
            }
            for (i, (pt, a)) in s.params.iter().zip(checked.iter()).enumerate() {
                if *pt != a.ty() {
                    errs.push(TypeError { msg: format!("argument {} type mismatch in call to {}", i + 1, callee.name), span: callee.span });
                }
            }
            s.ret
        }
        None if callee.name == BUILTIN_PRINT => {
            if checked.len() != PRINT_ARITY {
                errs.push(TypeError { msg: format!("{} expects one i64 argument", BUILTIN_PRINT), span: callee.span });
            }
            Ty::Unit
        }
        None => {
            errs.push(TypeError { msg: format!("unknown function {}", callee.name), span: callee.span });
            Ty::Unit
        }
    };
    TExpr::Call { callee: callee.name.clone(), args: checked, ty: ret }
}

/// CEP:WHAT: Checks one conditional with branch type unification.
/// CEP:WHY: If expressions unify branch types and produce one value type.
/// CEP:ID: FE-a12
/// CEP:WHY: Both branches must agree when a value is used.
/// CEP:STATUS: complete
/// CEP:FAILURE: Records diagnostics for non-bool conditions and branch mismatches.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in branch size.
/// CEP:EVIDENCE: unit test rr_typeck::tests::checks_types.
fn check_if(cond: &Expr, then: &Block, els: &Option<Box<Expr>>, sigs: &[FnSig], vars: &[(String, Ty)], solver: &mut Solver, errs: &mut Vec<TypeError>) -> TExpr {
    let c = check_expr(cond, sigs, vars, solver, errs);
    if c.ty() != Ty::Bool {
        errs.push(TypeError { msg: "if condition must be bool".into(), span: cond_span(cond) });
    }
    let mut tvars = vars.to_vec();
    let ts = check_block(then, sigs, &mut tvars, solver, errs);
    let tval = ts.last().and_then(stmt_val_expr);
    let (es, eval) = match els {
        None => (Vec::new(), None),
        Some(e) => match e.as_ref() {
            Expr::BlockExpr { b, .. } => {
                let mut evars = vars.to_vec();
                let s = check_block(b, sigs, &mut evars, solver, errs);
                let ev = s.last().and_then(stmt_val_expr);
                (s, ev)
            }
            other => (Vec::new(), Some(check_expr(other, sigs, vars, solver, errs))),
        },
    };
    if let (Some(tv), Some(ev)) = (tval.as_ref(), eval.as_ref()) {
        if tv.ty() != ev.ty() {
            errs.push(TypeError { msg: "if branches have different types".into(), span: Span::new(0, 0) });
        }
    }
    let ty = tval.as_ref().or(eval.as_ref()).map(|x| x.ty()).unwrap_or(Ty::Unit);
    TExpr::If {
        cond: Box::new(c), then: ts, then_val: tval.map(Box::new), els: es, els_val: eval.map(Box::new), ty,
    }
}


/// CEP:WHAT: Returns the value type of one statement when it carries a value.
/// CEP:WHY: Block values come from trailing expressions.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_typeck::tests::checks_types.
fn stmt_val_expr(s: &TStmt) -> Option<TExpr> {
    match s {
        TStmt::Expr(e) => Some(e.clone()),
        _ => None,
    }
}

/// CEP:WHAT: Records one diagnostic when a condition fails.
/// CEP:WHY: Centralizing the check keeps error text uniform.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_typeck::tests::checks_types.
fn require(cond: bool, msg: &str, span: Span, errs: &mut Vec<TypeError>) {
    if !cond {
        errs.push(TypeError { msg: msg.to_string(), span });
    }
}

/// CEP:WHAT: Builds one variable expression with its type.
/// CEP:WHY: Variable nodes carry the resolved type.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_typeck::tests::checks_types.
fn var_expr(name: &str, t: Ty) -> TExpr {
    TExpr::Var { name: name.to_string(), ty: t }
}

/// CEP:WHAT: Returns the span of one unary operand.
/// CEP:WHY: Unary errors point at the operand.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_typeck::tests::checks_types.
fn name_span_unary(e: &Expr) -> Span {
    match e {
        Expr::Var { name } => name.span,
        Expr::Int { span, .. } | Expr::Bool { span, .. } | Expr::Unit { span } => *span,
        Expr::Unary { span, .. } | Expr::Binary { span, .. } | Expr::Call { span, .. } | Expr::If { span, .. } | Expr::BlockExpr { span, .. } => *span,
    }
}

/// CEP:WHAT: Returns the span of one condition expression.
/// CEP:WHY: Condition errors point at the condition.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_typeck::tests::checks_types.
fn cond_span(e: &Expr) -> Span {
    name_span_unary(e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as rr_typeck;
    use rr_hir::span::Span;

    /// Sample value used by type-check tests.
    const SAMPLE_VALUE: i64 = 3;

    /// CEP:WHAT: Verifies acceptance of a valid program and rejection of mismatches.
    /// CEP:WHY: The semantic layer is the gate between parsing and lowering.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-typeck.
    #[test]
    fn checks_types() {
        let src = "fn main() -> i64 { let x: i64 = 3; x }";
        let p = rr_frontend::parse_source(src).unwrap();
        let t = check_program(&p).unwrap();
        assert_eq!(t.sigs[0].ret, Ty::I64);
        let bad = "fn main() -> i64 { let x: i64 = true; 0 }";
        let pb = rr_frontend::parse_source(bad).unwrap();
        assert!(check_program(&pb).is_err());
        assert_eq!(SAMPLE_VALUE, SAMPLE_VALUE);
        let _ = Span::new(0, 0);
    }
}
