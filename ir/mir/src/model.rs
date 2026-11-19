// CEP:FILE: ir/mir/src/model.rs
// CEP:WHAT: The MIR program model: functions, locals, basic blocks, and terminators.
// CEP:WHY: The IR contract of section 38.17 requires an explicit, target-independent program shape.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: none
// CEP:ASSUMES: Local zero is the return value; parameters occupy locals one through n.
// CEP:COST: cold; plain data.
// CEP:EVIDENCE: unit test rr_mir::model::tests::program_shape.
#![deny(missing_docs)]
use rr_hir::hir::{BinOp, Ty, UnOp};

/// Stable version of the MIR format for hashing and serialization.
pub const IR_VERSION: u32 = 1;

/// Identifier of a basic block.
pub type BlockId = usize;

/// Identifier of a local.
pub type LocalId = usize;

/// A MIR program.
#[derive(Debug, Clone)]
pub struct Program {
    /// IR format version for stability and serialization.
    pub version: u32,
    /// Functions in deterministic order.
    pub fns: Vec<Function>,
}

/// A MIR function.
#[derive(Debug, Clone)]
pub struct Function {
    /// Function name.
    pub name: String,
    /// Local declarations; local zero is the return slot.
    pub locals: Vec<Local>,
    /// Parameter locals in order.
    pub params: Vec<LocalId>,
    /// Return type.
    pub ret: Ty,
    /// Basic blocks; block zero is the entry.
    pub blocks: Vec<BasicBlock>,
}

/// A local declaration.
#[derive(Debug, Clone)]
pub struct Local {
    /// Local type.
    pub ty: Ty,
    /// True for parameters and the return slot.
    pub is_immutable: bool,
}

/// A basic block.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Statements executed in order.
    pub stmts: Vec<Statement>,
    /// The terminating transfer.
    pub term: Terminator,
}

/// A MIR statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Assign an rvalue to a place.
    Assign {
        /// Destination local.
        place: LocalId,
        /// The computed value.
        rvalue: Rvalue,
    },
    /// A no-op retained for stable block numbering.
    Nop,
}

/// An rvalue computation.
#[derive(Debug, Clone, PartialEq)]
pub enum Rvalue {
    /// Move or copy an operand.
    Use(Operand),
    /// A binary operation with wrapping or comparison semantics.
    BinaryOp {
        /// Operator.
        op: BinOp,
        /// Left operand.
        l: Operand,
        /// Right operand.
        r: Operand,
    },
    /// A unary operation.
    UnaryOp {
        /// Operator.
        op: UnOp,
        /// Operand.
        e: Operand,
    },
}

/// An operand of an rvalue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operand {
    /// A copy of a local value.
    Copy(LocalId),
    /// A constant.
    Const(Constant),
}

/// A constant value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Constant {
    /// An integer constant.
    I64(i64),
    /// A boolean constant.
    Bool(bool),
    /// The unit value.
    Unit,
}

/// A block terminator.
#[derive(Debug, Clone, PartialEq)]
pub enum Terminator {
    /// Fall through to one block.
    Goto(BlockId),
    /// Branch on a boolean discriminator.
    SwitchInt {
        /// Discriminant operand.
        disc: Operand,
        /// Targets keyed by constant value.
        targets: Vec<(i64, BlockId)>,
        /// Fallback target.
        otherwise: BlockId,
    },
    /// Call a function and continue in one block.
    Call {
        /// Callee name.
        callee: String,
        /// Arguments.
        args: Vec<Operand>,
        /// Destination local and resume block.
        dest: Option<(LocalId, BlockId)>,
    },
    /// Return from the function using local zero.
    Return,
    /// A trap such as an undefined branch.
    Unreachable,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CEP:WHAT: Verifies program shape construction.
    /// CEP:WHY: Every stage shares this shape; regressions must fail fast.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-mir.
    #[test]
    fn program_shape() {
        let p = Program { version: IR_VERSION, fns: Vec::new() };
        assert_eq!(p.version, 1);
    }
}
