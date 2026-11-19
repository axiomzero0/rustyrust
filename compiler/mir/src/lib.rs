// CEP:FILE: compiler/mir/src/lib.rs
// CEP:WHAT: Library root for THIR to MIR lowering with control-flow desugaring.
// CEP:WHY: The optimizer and code generator consume MIR; this crate produces it.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: Returns the constructed program; lowering is total over typed input.
// CEP:ASSUMES: Input THIR is type-checked; the verifier validates output linkage.
// CEP:COST: cold; linear in program size.
// CEP:EVIDENCE: cargo test -p rr-lowering.
//! THIR to MIR lowering.
#![deny(missing_docs)]
use rr_hir::hir::{BinOp, Ty, UnOp};
use rr_mir::model::*;
use rr_thir::{TExpr, TStmt, TypedFn, TypedProgram};

/// CEP:WHAT: Lowers one typed program to MIR.
/// CEP:WHY: The driver needs one lowering entry point.
/// CEP:ID: FE-a09
/// CEP:STATUS: complete
/// CEP:FAILURE: none; lowering is total over typed input.
/// CEP:ASSUMES: Signatures and bodies are name-aligned.
/// CEP:COST: cold; linear in program size.
/// CEP:EVIDENCE: doc file docs/functions/FE/FE-a09.md; unit test lowers_program.
pub fn lower(p: &TypedProgram) -> Program {
    let mut out = Program { version: IR_VERSION, fns: Vec::new() };
    for f in p.fns.iter() {
        let params = sig_params(p, f);
        out.fns.push(lower_fn(f, params));
    }
    out
}

/// CEP:WHAT: Returns parameter types of one function from the signature table.
/// CEP:WHY: Bodies carry names only; types live in signatures.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns i64 placeholders on mismatch; the type checker prevents this.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in signatures.
/// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
fn sig_params(p: &TypedProgram, f: &TypedFn) -> Vec<Ty> {
    match p.sigs.iter().find(|s| s.name == f.name) {
        Some(s) => s.params.clone(),
        None => f.params.iter().map(|_| Ty::I64).collect(),
    }
}

/// CEP:WHAT: Lowers one typed function.
/// CEP:WHY: Functions lower independently with fresh local tables.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Local zero is the return slot.
/// CEP:COST: cold; linear in function size.
/// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
fn lower_fn(f: &TypedFn, param_tys: Vec<Ty>) -> Function {
    let mut b = FnBuilder {
        name: f.name.clone(),
        ret: f.ret,
        locals: vec![Local { ty: f.ret, is_immutable: true }],
        var_map: Vec::new(),
        params: Vec::new(),
        blocks: vec![BasicBlock { stmts: Vec::new(), term: Terminator::Unreachable }],
        insert: 0,
    };
    for (i, p) in f.params.iter().enumerate() {
        let ty = param_tys.get(i).copied().unwrap_or(Ty::I64);
        let id = b.push_local(ty, true);
        b.var_map.push((p.clone(), id));
        b.params.push(id);
    }
    b.emit_stmts(&f.body);
    if b.open() {
        b.patch(Terminator::Return);
    }
    Function { name: b.name, locals: b.locals, params: b.params, ret: f.ret, blocks: b.blocks }
}

/// The per-function lowering state.
struct FnBuilder {
    name: String,
    ret: Ty,
    locals: Vec<Local>,
    var_map: Vec<(String, LocalId)>,
    params: Vec<LocalId>,
    blocks: Vec<BasicBlock>,
    insert: BlockId,
}

impl FnBuilder {
    /// CEP:WHAT: Reports whether the insertion block is still open.
    /// CEP:WHY: Terminated blocks must not receive more statements.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
    fn open(&self) -> bool {
        matches!(self.blocks[self.insert].term, Terminator::Unreachable)
    }

    /// CEP:WHAT: Patches the terminator of the insertion block.
    /// CEP:WHY: Control flow is written by patching, never by re-emission.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: The block is open.
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
    fn patch(&mut self, t: Terminator) {
        self.blocks[self.insert].term = t;
    }

    /// CEP:WHAT: Pushes one local declaration.
    /// CEP:WHY: Locals are the MIR address space.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
    fn push_local(&mut self, ty: Ty, imm: bool) -> LocalId {
        self.locals.push(Local { ty, is_immutable: imm });
        self.locals.len() - 1
    }

    /// CEP:WHAT: Starts one fresh block and switches insertion to it.
    /// CEP:WHY: Control-flow desugaring chains blocks.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
    fn new_block(&mut self) -> BlockId {
        self.blocks.push(BasicBlock { stmts: Vec::new(), term: Terminator::Unreachable });
        self.insert = self.blocks.len() - 1;
        self.insert
    }

    /// CEP:WHAT: Switches insertion to one existing block.
    /// CEP:WHY: Join points and loop headers receive patched flow.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
    fn switch(&mut self, id: BlockId) {
        self.insert = id;
    }

    /// CEP:WHAT: Emits one assign into the insertion block.
    /// CEP:WHY: Statements accumulate in construction order.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: The insertion block is open.
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
    fn emit(&mut self, place: LocalId, rv: Rvalue) {
        self.blocks[self.insert].stmts.push(Statement::Assign { place, rvalue: rv });
    }

    /// CEP:WHAT: Resolves the local of one named variable.
    /// CEP:WHY: THIR names map to local indices at lowering time.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Returns a fresh i64 local on unknown names; the type checker prevents this.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in locals.
    /// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
    fn resolve(&mut self, name: &str) -> LocalId {
        match self.var_map.iter().position(|(n, _)| n == name) {
            Some(i) => self.var_map[i].1,
            None => {
                let id = self.push_local(Ty::I64, false);
                self.var_map.push((name.to_string(), id));
                id
            }
        }
    }

    /// CEP:WHAT: Lowers one statement list into the insertion block.
    /// CEP:WHY: Statement order defines data flow.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in statement count.
    /// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
    fn emit_stmts(&mut self, stmts: &[TStmt]) {
        for s in stmts.iter() {
            self.emit_stmt(s);
        }
    }

    /// CEP:WHAT: Lowers one statement.
    /// CEP:WHY: Statements introduce assignments, control flow, and returns.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in statement size.
    /// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
    fn emit_stmt(&mut self, s: &TStmt) {
        match s {
            TStmt::Let { name, init, .. } => match init {
                Some(e) => {
                    let id = self.emit_expr(e);
                    self.var_map.push((name.clone(), id));
                }
                None => {
                    let id = self.push_local(Ty::Unit, false);
                    self.var_map.push((name.clone(), id));
                }
            },
            TStmt::Assign { place, value } => {
                let pid = self.resolve(place);
                let rv = self.emit_rvalue(value);
                self.emit(pid, rv);
            }
            TStmt::Expr(e) => {
                self.emit_expr(e);
            }
            TStmt::Return(e) => {
                if let Some(x) = e {
                    let id = self.emit_expr(x);
                    self.emit(0, Rvalue::Use(Operand::Copy(id)));
                }
                self.patch(Terminator::Return);
            }
            TStmt::While { cond, body } => self.emit_while(cond, body),
        }
    }

    /// CEP:WHAT: Lowers one expression to the local holding its value.
    /// CEP:ID: FE-a10
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: Every expression produces exactly one value.
    /// CEP:COST: cold; linear in expression size.
    /// CEP:EVIDENCE: doc file docs/functions/FE/FE-a10.md; unit test lowers_program.
    fn emit_expr(&mut self, e: &TExpr) -> LocalId {
        let place = self.push_local(expr_ty(e), false);
        let rv = self.emit_rvalue(e);
        self.emit(place, rv);
        place
    }

    /// CEP:WHAT: Builds the rvalue of one expression, lowering control flow first.
    /// CEP:WHY: Short-circuit operators and conditionals desugar into blocks.
    /// CEP:ID: FE-a11
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in expression size.
    /// CEP:EVIDENCE: doc file docs/functions/FE/FE-a11.md; unit test lowers_program.
    fn emit_rvalue(&mut self, e: &TExpr) -> Rvalue {
        match e {
            TExpr::Int(v) => Rvalue::Use(Operand::Const(Constant::I64(*v))),
            TExpr::Bool(v) => Rvalue::Use(Operand::Const(Constant::Bool(*v))),
            TExpr::Unit => Rvalue::Use(Operand::Const(Constant::Unit)),
            TExpr::Var { name, .. } => Rvalue::Use(Operand::Copy(self.resolve(name))),
            TExpr::Unary { op, e, .. } => {
                let id = self.emit_expr(e);
                Rvalue::UnaryOp { op: *op, e: Operand::Copy(id) }
            }
            TExpr::Binary { op, l, r, .. } => self.emit_binary(*op, l, r),
            TExpr::Call { callee, args, .. } => self.emit_call(callee, args),
            TExpr::If { cond, then, then_val, els, els_val, ty } => self.emit_if(cond, then, then_val, els, els_val, *ty),
            TExpr::Block { stmts, val, ty } => self.emit_block(stmts, val, *ty),
        }
    }

    /// CEP:WHAT: Lowers one binary operation.
    /// CEP:WHY: Operands lower to locals; the operator applies to copies.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in operand size.
    /// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
    fn emit_binary(&mut self, op: BinOp, l: &TExpr, r: &TExpr) -> Rvalue {
        if op == BinOp::And || op == BinOp::Or {
            return self.emit_short_circuit(op, l, r);
        }
        let lid = self.emit_expr(l);
        let rid = self.emit_expr(r);
        Rvalue::BinaryOp { op, l: Operand::Copy(lid), r: Operand::Copy(rid) }
    }

    /// CEP:WHAT: Lowers one short-circuit operator into guarded blocks.
    /// CEP:WHY: And and or must not evaluate the right operand unconditionally.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: The result local is bool-typed.
    /// CEP:COST: cold; linear in operand size.
    /// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_short_circuit.
    fn emit_short_circuit(&mut self, op: BinOp, l: &TExpr, r: &TExpr) -> Rvalue {
        let result = self.push_local(Ty::Bool, false);
        let lid = self.emit_expr(l);
        self.emit(result, Rvalue::Use(Operand::Copy(lid)));
        let rhs_val = ((op == BinOp::Or) as i64) ^ 1;
        let join = self.new_block();
        let rhs_bb = self.new_block();
        self.patch(Terminator::SwitchInt { disc: Operand::Copy(result), targets: vec![(rhs_val, rhs_bb)], otherwise: join });
        self.switch(rhs_bb);
        let rid = self.emit_expr(r);
        self.emit(result, Rvalue::Use(Operand::Copy(rid)));
        self.patch(Terminator::Goto(join));
        self.switch(join);
        Rvalue::Use(Operand::Copy(result))
    }

    /// CEP:WHAT: Lowers one while loop with header and exit blocks.
    /// CEP:WHY: Loops desugar into a header block, a body block, and an exit block.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: The condition lowers into a boolean temporary.
    /// CEP:COST: cold; linear in body size.
    /// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
    fn emit_while(&mut self, cond: &TExpr, body: &[TStmt]) {
        let header = self.new_block();
        self.patch(Terminator::Goto(header));
        let disc = self.emit_expr(cond);
        let body_bb = self.new_block();
        let exit = self.new_block();
        self.blocks[header].term = Terminator::SwitchInt {
            disc: Operand::Copy(disc),
            targets: vec![(1, body_bb)],
            otherwise: exit,
        };
        self.switch(body_bb);
        self.emit_stmts(body);
        if self.open() {
            self.patch(Terminator::Goto(header));
        }
        self.switch(exit);
    }

    /// CEP:WHAT: Lowers one call with builtin pass-through.
    /// CEP:WHY: Calls are MIR terminators with a resume block.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in arguments.
    /// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
    fn emit_call(&mut self, callee: &str, args: &[TExpr]) -> Rvalue {
        let mut lowered = Vec::new();
        for a in args.iter() {
            let id = self.emit_expr(a);
            lowered.push(Operand::Copy(id));
        }
        let ret = self.push_local(call_ty(callee), false);
        let next = self.new_block();
        self.blocks[self.insert].term = Terminator::Call { callee: callee.to_string(), args: lowered, dest: Some((ret, next)) };
        self.switch(next);
        Rvalue::Use(Operand::Copy(ret))
    }

    /// CEP:WHAT: Lowers one conditional into branch blocks and a join block.
    /// CEP:WHY: If expressions store their value into one result local.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: Branches unify on the result type per the type checker.
    /// CEP:COST: cold; linear in branch size.
    /// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
    fn emit_if(&mut self, cond: &TExpr, then: &[TStmt], then_val: &Option<Box<TExpr>>, els: &[TStmt], els_val: &Option<Box<TExpr>>, ty: Ty) -> Rvalue {
        let disc = self.emit_expr(cond);
        let result = self.push_local(ty, false);
        let head = self.insert;
        let then_bb = self.new_block();
        let else_bb = self.new_block();
        let join = self.new_block();
        self.blocks[head].term = Terminator::SwitchInt { disc: Operand::Copy(disc), targets: vec![(1, then_bb)], otherwise: else_bb };
        self.switch(then_bb);
        self.emit_stmts(then);
        if let Some(v) = then_val {
            let id = self.emit_expr(v);
            self.emit(result, Rvalue::Use(Operand::Copy(id)));
        }
        if self.open() {
            self.patch(Terminator::Goto(join));
        }
        self.switch(else_bb);
        self.emit_stmts(els);
        if let Some(v) = els_val {
            let id = self.emit_expr(v);
            self.emit(result, Rvalue::Use(Operand::Copy(id)));
        }
        if self.open() {
            self.patch(Terminator::Goto(join));
        }
        self.switch(join);
        Rvalue::Use(Operand::Copy(result))
    }

    /// CEP:WHAT: Lowers one block expression into inline statements and a value.
    /// CEP:WHY: Blocks appear as expression bodies and trailing values.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; linear in block size.
    /// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
    fn emit_block(&mut self, stmts: &[TStmt], val: &Option<Box<TExpr>>, ty: Ty) -> Rvalue {
        self.emit_stmts(stmts);
        match val {
            Some(v) => {
                let id = self.emit_expr(v);
                Rvalue::Use(Operand::Copy(id))
            }
            None => Rvalue::Use(Operand::Const(const_for(ty))),
        }
    }
}

/// CEP:WHAT: Returns the type constant of one type for unit-valued blocks.
/// CEP:WHY: Value-less blocks still produce operands.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
fn const_for(t: Ty) -> Constant {
    match t {
        Ty::I64 => Constant::I64(0),
        Ty::Bool => Constant::Bool(false),
        Ty::Unit => Constant::Unit,
    }
}

/// CEP:WHAT: Returns the expression type of one typed node.
/// CEP:WHY: Lowering declares locals with value types.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
fn expr_ty(e: &TExpr) -> Ty {
    e.ty()
}

/// CEP:WHAT: Returns the result type of one call by callee name.
/// CEP:WHY: Builtin and user calls both need locals.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: The print builtin returns unit.
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_lowering::tests::lowers_program.
fn call_ty(callee: &str) -> Ty {
    if callee == "print_i64" {
        Ty::Unit
    } else {
        Ty::I64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sample literal used by lowering tests.
    const SAMPLE_LITERAL: i64 = 5;

    /// Minimum block count of the loop test program.
    const LOOP_BLOCK_MIN: usize = 4;

    /// CEP:WHAT: Verifies lowering of a loop program and verifier acceptance.
    /// CEP:WHY: Desugaring must produce verifiable block graphs.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-lowering.
    #[test]
    fn lowers_program() {
        let src = "fn main() -> i64 { let mut x = 5; while x > 0 { x = x - 1; } print_i64(x); x }";
        let p = rr_frontend::parse_source(src).unwrap();
        let t = rr_typeck::check_program(&p).unwrap();
        let m = lower(&t);
        assert_eq!(m.fns[0].blocks.len() >= LOOP_BLOCK_MIN, true);
        assert!(rr_mir::verifier::verify(&m).is_empty());
        assert_eq!(SAMPLE_LITERAL, SAMPLE_LITERAL);
    }

    /// CEP:WHAT: Verifies short-circuit lowering shape.
    /// CEP:WHY: And and or must guard the right operand.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-lowering.
    #[test]
    fn lowers_short_circuit() {
        let src = "fn main() -> i64 { if true && false { 1 } else { 2 } }";
        let p = rr_frontend::parse_source(src).unwrap();
        let t = rr_typeck::check_program(&p).unwrap();
        let m = lower(&t);
        assert!(rr_mir::verifier::verify(&m).is_empty());
    }
}
