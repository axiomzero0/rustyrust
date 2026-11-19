// CEP:FILE: ir/lir/src/lib.rs
// CEP:WHAT: Library root for the low-level IR.
// CEP:WHY: Instruction selection emits it; register allocation and the encoder consume it.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: none
// CEP:ASSUMES: The instruction set models the x86-64 subset used by the V0 backend.
// CEP:COST: cold; type definitions only.
// CEP:EVIDENCE: cargo build -p rr-lir.
//! Low-level IR with virtual and physical operands.
#![deny(missing_docs)]

/// Condition codes for branches and flags materialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cond {
    /// Equal.
    Eq,
    /// Not equal.
    Ne,
    /// Less (signed).
    Lt,
    /// Less or equal (signed).
    Le,
    /// Greater (signed).
    Gt,
    /// Greater or equal (signed).
    Ge,
}

/// A physical register in the reserved subset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phys {
    /// Accumulator and division dividend.
    Rax,
    /// Sign-extension target for division.
    Rdx,
    /// Argument one and scratch.
    Rdi,
    /// Argument two and scratch.
    Rsi,
    /// Scratch.
    Rc8,
    /// Callee pointer.
    RipRel,
}

/// An operand of a low-level instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    /// A virtual register awaiting assignment.
    Vr(u32),
    /// A physical register.
    Phys(Phys),
    /// An rbp-relative stack slot for a local.
    Slot(i32),
    /// A signed 64-bit immediate.
    Imm(i64),
}

/// A label identifier within one function image.
pub type LabelId = u32;

/// A low-level instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inst {
    /// Move between operand forms.
    Mov {
        /// Destination.
        dst: Operand,
        /// Source.
        src: Operand,
    },
    /// Wrapping addition.
    Add {
        /// Destination and left operand.
        dst: Operand,
        /// Right operand.
        src: Operand,
    },
    /// Wrapping subtraction.
    Sub {
        /// Destination and left operand.
        dst: Operand,
        /// Right operand.
        src: Operand,
    },
    /// Wrapping multiplication.
    Imul {
        /// Destination and left operand.
        dst: Operand,
        /// Right operand.
        src: Operand,
    },
    /// Signed division with the dividend in RAX and RDX sign-extended.
    Idiv {
        /// Divisor operand.
        divisor: Operand,
    },
    /// Sign-extend RAX into RDX.
    Cqo,
    /// Compare two operands.
    Cmp {
        /// Left operand.
        l: Operand,
        /// Right operand.
        r: Operand,
    },
    /// Materialize a condition code as zero or one.
    Setcc {
        /// Condition.
        cond: Cond,
        /// Destination.
        dst: Operand,
    },
    /// Conditional jump to a label.
    Jcc {
        /// Condition.
        cond: Cond,
        /// Target label.
        target: LabelId,
    },
    /// Unconditional jump to a label.
    Jmp(LabelId),
    /// Call a named function.
    Call(String),
    /// Return from the function.
    Ret,
    /// Push the frame pointer.
    PushRbp,
    /// Copy the stack pointer to the frame pointer.
    MovRbpRsp,
    /// Reserve stack space for the frame.
    SubRsp(i32),
    /// Release the frame.
    MovRspRbp,
    /// Pop the frame pointer.
    PopRbp,
    /// A label target.
    Label(LabelId),
}
