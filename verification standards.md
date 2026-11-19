

# **1. Primary project goals**

**G1. Semantic correctness**

Mandatory

Every accepted source program must compile into a target program that preserves its specified observable behavior.

**G2. Memory safety**

Mandatory

Generated programs must not perform invalid memory accesses, use-after-free, invalid pointer arithmetic, or other prohibited memory operations, assuming the specified runtime and target contracts hold.

**G3. Compiler self-correctness**

Mandatory

The compiler itself must satisfy its own safety and correctness specifications, including its parser, optimizer, code generator, and runtime interfaces.

**G4. Optimization correctness**

Mandatory

Every optimization must preserve semantics, including edge cases, undefined behavior boundaries, concurrency, exceptions, and observable effects.

**G5. Performance non-regression**

Mandatory for accepted optimizations

An optimization must not introduce an unacceptable performance regression under its declared workload and measurement model.

**G6. Optimization effectiveness**

Mandatory

Optimization effort must prioritize transformations with the highest expected performance benefit relative to implementation, compilation, and runtime costs.

**G7. Reproducibility**

Mandatory

Identical inputs, compiler configurations, target descriptions, and toolchain versions must produce reproducible results within the defined determinism contract.

# **2. Formal correctness goals**

These are the properties I'd require the verification system to establish.

## **G1. Source-to-target semantic equivalence**

Define the semantics of the source language and target machine independently.

Let:

- \\(P\\) be a source program.
- \\(C\\) be the compiler.
- \\(C(P)\\) be its generated target program.
- \\(\llbracket P \rrbracket_S\\) be source semantics.
- \\(\llbracket C(P) \rrbracket_T\\) be target semantics.

The fundamental correctness requirement is:

\\[ \forall P,\quad \llbracket P \rrbracket_S = \llbracket C(P) \rrbracket_T \\]

The equality must be defined over observable behavior, not merely final return values.

Specify exactly what observable behavior includes:

- Return values.
- Memory-visible state.
- I/O.
- Exceptions and traps.
- Allocation behavior where observable.
- Atomic operations and synchronization.
- Termination and divergence.
- Foreign-function interactions.
- Required ordering of externally visible effects.

For nondeterministic or concurrent programs, equality may need to be defined as equality or refinement of sets of possible behaviors.

**Goal:** Prove semantic preservation for every accepted source program, under explicit source, target, and runtime assumptions.

## **G2. Memory safety under all inputs**

This is particularly important.

The property should not be:

"The compiler passed a million memory-safety tests."

It should be:

\\[ \forall P \in \text{WellTypedPrograms}, \forall I \in \text{ValidInputs}, \quad \operatorname{Safe}(C(P),I) \\]

Define safety precisely:

1. No out-of-bounds reads or writes.
2. No use-after-free.
3. No double-free.
4. No invalid deallocation.
5. No uninitialized memory reads where prohibited.
6. No invalid pointer dereferences.
7. No pointer arithmetic outside permitted bounds.
8. No data races under the memory model.
9. No invalid object lifetime transitions.
10. No violation of alignment requirements.
11. No integer overflow leading to invalid memory operations.
12. No invalid stack or heap accesses.
13. No violation of aliasing and provenance rules.

There is a catch: memory safety is not a property the compiler can guarantee independently of every external component.

Your theorem needs explicit assumptions about:

- The target ISA and its semantics.
- The runtime and allocator.
- Foreign functions and imported libraries.
- Unsafe or explicitly trusted source-language operations.
- The operating system and execution environment.

A compiler cannot prove an arbitrary C library safe merely because it called it politely.

## **G3. Type safety**

Establish a preservation and progress theorem.

**Preservation:**

\\[ \Gamma \vdash e : T \land e \rightarrow e' \implies \Gamma' \vdash e' : T \\]

The exact form depends on your language's typing and state semantics.

**Progress:** A well-typed program is either a value, can take a valid execution step, or reaches an explicitly permitted terminal state.

Specify:

- Type soundness.
- Generic instantiation correctness.
- Subtyping, if supported.
- Trait/interface dispatch.
- Lifetime and ownership rules, if supported.
- Refinement types, if supported.
- Runtime type representation.
- Type erasure and specialization correctness.

## **G4. Compiler pass correctness**

Every pass gets its own proof obligation.

For a transformation \\(T\\):

\\[ \forall x \in D_T,\quad \llbracket T(x) \rrbracket = \llbracket x \rrbracket \\]

Here, \\(D_T\\) is the precisely defined valid input domain of the pass.

For example:

| Pass                             | Required proof                                                    |
| -------------------------------- | ----------------------------------------------------------------- |
| Constant folding                 | Same result, including overflow and exceptional cases             |
| Dead-code elimination            | Removed instructions have no observable effects                   |
| Inlining                         | Call semantics, stack behavior, exceptions, and effects preserved |
| Loop unrolling                   | Iteration count and effects preserved                             |
| Common subexpression elimination | Memory and effect dependencies preserved                          |
| Register allocation              | Values and calling conventions preserved                          |
| Instruction selection            | Selected instruction sequence implements the original operation   |
| Vectorization                    | Lane semantics, memory behavior, and ordering preserved           |
| Code layout                      | Control-flow behavior preserved                                   |
| Link-time optimization           | Cross-module semantics preserved                                  |

No pass gets to claim correctness because it "usually works." That is how compilers acquire folklore instead of specifications.

# **3. Optimization and performance verification**

This is where I'd make your compiler particularly unusual.

You want more than proof that an optimization is correct. You want proof that it is worthwhile.

I'd split performance verification into four separate goals.

## **G5. Semantic equivalence after every optimization**

Suppose your optimization pipeline is:

\\[ IR_0 \xrightarrow{P_1} IR_1 \xrightarrow{P_2} IR_2 \xrightarrow{P_3} IR_3 \xrightarrow{P_4} IR_4 \\]

Every transformation must preserve the semantics of its input.

\\[ \forall i,\quad \llbracket IR_i \rrbracket = \llbracket IR\_{i+1} \rrbracket \\]

This gives you compositional correctness:

\\[ \llbracket IR_0 \rrbracket = \llbracket IR_n \rrbracket \\]

You should be able to identify precisely which transformation introduced an invalid result.

**Acceptance criteria:**

- Every optimization has a formal precondition.
- Every optimization has a formal postcondition.
- Every optimization has a semantic preservation proof.
- Every optimization has explicitly defined exceptional cases.
- Pass composition is proven correct.
- Invalid transformations produce counterexamples or failed proof obligations.
- No unverified optimization silently enters the trusted compilation pipeline.

## **G6. Performance non-regression**

This needs a more careful definition.

You cannot prove that a pass never regresses performance on every conceivable processor and every conceivable workload without making extremely restrictive assumptions.

You can prove performance properties relative to a cost model.

Define:

\\[ C(P,x,A) \\]

Where:

- \\(P\\) is a program.
- \\(x\\) is an input.
- \\(A\\) is a target architecture.
- \\(C\\) is the measured or modeled execution cost.

Your optimization \\(T\\) should satisfy a declared performance contract.

For example:

\\[ C(T(P),x,A) \leq C(P,x,A) \\]

for all programs and inputs within a formally established domain.

Possible cost dimensions:

| Metric                    | What you want to establish                     |
| ------------------------- | ---------------------------------------------- |
| Dynamic instruction count | No increase under a specified execution model  |
| Static instruction count  | Code-size non-regression                       |
| Memory accesses           | Reduced or bounded memory traffic              |
| Allocations               | No additional allocations                      |
| Register pressure         | No unacceptable increase                       |
| Branches                  | Reduced branch count or misprediction exposure |
| Compilation time          | Bounded additional compilation cost            |
| Peak memory               | Bounded compiler memory consumption            |
| Execution cycles          | Improvement under a specified processor model  |

Important: reducing instruction count does not automatically reduce execution time. Dependencies, cache behavior, instruction-level parallelism, branch prediction, and memory latency all matter.

I would establish three levels of performance evidence:

Level 1

**Formal cost-model proof**

Prove that the transformation improves or preserves a specified abstract cost metric.

Level 2

**Microarchitectural analysis**

Analyze instruction selection, dependencies, register pressure, cache behavior, and processor-specific execution characteristics.

Level 3

**Empirical verification**

Benchmark representative workloads across supported hardware and detect statistically significant regressions.

These levels complement each other. None should be misrepresented as a substitute for the others.

## **G7. Optimization effectiveness**

This is the goal behind your biggest-bang-for-buck requirement.

You don't want 50 optimization passes that collectively produce a 2% improvement while making compilation 40% slower.

You want to maximize actual performance improvement per unit of engineering and compilation cost.

Define an optimization utility function:

\\[ U(T)= \frac{ B(T)\cdot R(T)\cdot F(T) }{ K(T) } \\]

Where:

- \\(B(T)\\): expected performance benefit.
- \\(R(T)\\): reliability of that benefit across relevant workloads.
- \\(F(T)\\): frequency of applicable opportunities.
- \\(K(T)\\): total cost, including implementation, proof, compile-time, memory, and maintenance costs.

This is a prioritization heuristic, not a mathematical law. Its inputs need measured or otherwise justified estimates.

I would require every optimization to have an optimization dossier containing:

1. The opportunity it exploits.
2. Its applicability conditions.
3. The programs and workloads that benefit.
4. The expected execution-time improvement.
5. The expected code-size impact.
6. The expected compilation-time impact.
7. The register and memory-pressure impact.
8. The formal semantic preservation proof.
9. The performance evidence.
10. Its interaction with other passes.

### **Optimization selection policy**

Your optimizer should distinguish between:

- **Always-beneficial transformations:** proven improvements under a specified cost model.
- **Conditionally beneficial transformations:** useful only when certain properties hold.
- **Profile-guided transformations:** require workload evidence.
- **Target-specific transformations:** depend on processor characteristics.
- **Potentially harmful transformations:** must be justified by a cost-benefit model.

For example, loop unrolling may reduce branch overhead but increase code size and register pressure.

Inlining may remove call overhead but destroy instruction-cache locality.

Vectorization may improve throughput but increase setup costs for small inputs.

The optimizer should not blindly apply these transformations.

### **G8. Optimization ordering and interaction**

Individual passes can be correct and still interact badly from a performance perspective.

For example:

\\[ P_1(P_2(IR)) \neq P_2(P_1(IR)) \\]

This does not necessarily mean their semantics differ. It means their optimization opportunities and resulting machine code can differ.

Establish goals for:

- Pass ordering.
- Pass commutativity where applicable.
- Idempotence.
- Redundant transformation elimination.
- Optimization phase ordering.
- Cyclic optimization detection.
- Compile-time budget allocation.
- Global versus local optimization.
- Target-specific optimization scheduling.

Ideally, your optimizer should reason about optimization sequences rather than treating each pass as an isolated checkbox.

**Stretch goal:** Use a formally verified search-based optimizer to explore alternative equivalent representations and select the best candidate under a target-specific cost model.

This is where superoptimization becomes interesting.

## **G9. Code generation quality**

Your final machine code deserves its own verification goals.

Require:

- Correct instruction selection.
- Correct register allocation.
- Correct register spilling.
- Correct calling convention implementation.
- Correct stack-frame construction.
- Correct ABI compliance.
- Correct instruction encoding.
- Correct relocation handling.
- Correct exception/unwind metadata.
- Correct atomic instruction lowering.
- Correct SIMD lowering.
- Correct target-feature handling.
- Correct linker interaction.

Additionally:

**The generated binary must not contain unnecessary operations introduced by the compiler when a demonstrably cheaper equivalent implementation exists within the optimizer's supported search domain.**

That last qualification matters. Otherwise, you're claiming global optimality, which is a much larger problem.

## **G10. Target-specific optimization**

A compiler should not assume that all processors have identical performance characteristics.

Define architecture-specific cost models for:

- Instruction latency.
- Instruction throughput.
- Dependency chains.
- Branch prediction characteristics.
- Cache hierarchy.
- Memory bandwidth.
- SIMD width.
- Register availability.
- Instruction fusion.
- Load/store behavior.
- Atomic operations.

The compiler should distinguish between:

1. Correctness that is independent of the target.
2. Correctness that depends on the target ISA.
3. Performance that depends on the target microarchitecture.

A binary optimized for one processor should not accidentally be treated as universally optimal.

# **4. Compiler reliability goals**

## **G11. Compiler termination**

For every compiler phase, define whether termination is guaranteed.

You should establish:

- Parser termination.
- Type-checker termination.
- Generic instantiation termination or explicit limits.
- Optimization termination.
- Code-generation termination.
- Link-time termination or explicit resource limits.

Optimization passes deserve particular attention.

A fixed-point optimizer must either:

- Prove convergence.
- Use a well-founded termination measure.
- Have a rigorously defined iteration limit and fallback.
- Or explicitly support a bounded search strategy.

No infinite optimization loops.

## **G12. Compiler memory safety**

The compiler itself must be safe independently of the programs it compiles.

Prove:

- No invalid compiler-side memory access.
- No use-after-free.
- No invalid ownership transitions.
- No out-of-bounds compiler data structures.
- No invalid arena lifetimes.
- No integer-overflow-induced allocation errors.
- No invalid serialization/deserialization.
- No malformed IR causing unsafe compiler behavior.

This should cover adversarial inputs, not merely valid source programs.

In particular, malformed source files should not be capable of triggering memory corruption.

## **G13. Compiler resource safety**

A compiler can be memory-safe and still be a spectacular resource-management disaster.

Specify limits and guarantees for:

- Peak memory usage.
- Maximum recursion depth.
- Maximum generic instantiation depth.
- Maximum IR size.
- Maximum optimization search space.
- Maximum compile time.
- Maximum generated object size.
- Maximum diagnostic output.

For untrusted source inputs, resource exhaustion should be handled as a defined compilation failure, not an uncontrolled crash.

## **G14. Deterministic compilation**

Given identical:

\\[ (P,\ Configuration,\ Target,\ Toolchain) \\]

the compiler should produce identical outputs, where deterministic output is part of the declared build contract.

Specify:

- Deterministic symbol ordering.
- Deterministic optimization decisions.
- Deterministic register allocation tie-breaking.
- Deterministic object-file generation.
- Deterministic linking.
- Stable diagnostics.
- Reproducible build metadata.

If profile-guided optimization or nondeterministic search is used, define the seed and reproducibility requirements.

# **5. Verification infrastructure goals**

These are especially important because your compiler is going to have a large number of interacting proofs.

## **G15. Proof completeness**

Every property that is claimed to be verified must have an explicit theorem and proof obligation.

Create a verification matrix:

| Component            | Required verification              |
| -------------------- | ---------------------------------- |
| Lexer                | Tokenization correctness           |
| Parser               | Grammar conformance                |
| AST                  | Structural invariants              |
| Type checker         | Type soundness                     |
| Generic system       | Instantiation correctness          |
| IR                   | Structural and semantic invariants |
| Optimization passes  | Semantic preservation              |
| Register allocator   | Allocation correctness             |
| Instruction selector | Instruction equivalence            |
| Code generator       | Machine-code correctness           |
| Runtime              | Runtime invariant preservation     |
| Linker integration   | Linking and relocation correctness |
| Bootstrap compiler   | Self-hosting correctness           |

Do not confuse test coverage with proof coverage.

## **G16. Trusted computing base minimization**

Your trusted computing base (TCB) is everything that must be trusted for your verification claims to hold.

Minimize it.

Explicitly identify:

- The proof kernel.
- The theorem prover.
- The compiler's specification.
- The formal semantics.
- The target-machine model.
- The runtime assumptions.
- The bootstrapping assumptions.
- Any external unverified tools.

A large verified compiler with an enormous unverified proof infrastructure is a questionable achievement.

## **G17. Counterexample generation**

When a proof fails, the system should produce a useful counterexample.

Ideally, every failed verification obligation identifies:

- The violated property.
- The relevant source program or IR.
- The execution path.
- The failing input.
- The violated precondition or postcondition.
- The first point of semantic divergence.

This is particularly valuable for optimizer verification.

A counterexample should be small enough to understand and reproduce. Counterexample minimization is a worthwhile goal in its own right.

## **G18. Independent verification**

Where possible, avoid having one component certify itself without independent checking.

Require:

- Independently checkable proof artifacts.
- Proof replay.
- Cross-validation of semantic models.
- Differential testing against reference implementations.
- Independent machine-code validation where feasible.
- Reproducible proof results.

This makes the verification process less dependent on trust in the implementation being verified.

# **6. Language and runtime guarantees**

Your compiler cannot be more formally defined than the language it implements.

These are language-level goals, not merely compiler implementation details.

## **G19. Complete language semantics**

Every language feature must have a precise definition.

No unspecified behavior unless explicitly intended. No vague wording such as "implementation-defined performance" without an actual contract.

Define:

- Value semantics.
- Object identity.
- Evaluation order.
- Expression semantics.
- Statement semantics.
- Control flow.
- Function calls.
- Closures.
- Generics.
- Modules.
- Namespaces.
- Type conversions.
- Integer representations.
- Floating-point semantics.
- String representation.
- Array semantics.
- Object layout.
- Mutability.
- Ownership and borrowing, if applicable.
- Concurrency semantics.
- Atomic memory operations.
- Exception handling.
- Undefined behavior.
- Foreign-function interfaces.
- Runtime initialization.
- Memory allocation and reclamation.

Every feature must have a formal semantic definition and corresponding implementation obligations.

## **G20. Runtime correctness**

The runtime is part of the trusted execution environment.

Verify:

- Allocator correctness.
- Object lifetime correctness.
- Garbage collector correctness, if applicable.
- Stack management.
- Thread management.
- Synchronization primitives.
- Exception propagation.
- Dynamic dispatch.
- Runtime type information.
- Array and string operations.
- Standard library contracts.
- FFI boundaries.

A formally verified compiler that emits code calling an incorrectly implemented runtime is not a formally verified execution system.

## **G21. Concurrency correctness**

For a general-purpose language, this deserves its own specification.

Define:

- Memory consistency model.
- Atomicity.
- Synchronization.
- Happens-before relations.
- Data-race semantics.
- Thread creation and destruction.
- Locking semantics.
- Memory reclamation.
- Weak-memory behavior.
- Compiler reordering constraints.

Prove that compiler transformations preserve the concurrency semantics.

This is particularly important because an optimization that is perfectly valid in sequential execution can be invalid under concurrent execution.

# **7. Production engineering goals**

## **G22. Diagnostics correctness**

The compiler should provide accurate and reproducible diagnostics.

- Correct source locations.
- Correct error classification.
- No misleading diagnostics.
- No silently ignored errors.
- No malformed source causing compiler memory corruption.
- Stable diagnostic output.
- Useful explanations of failed constraints.

## **G23. Incremental compilation**

If supported, incremental compilation must preserve the correctness guarantees of full compilation.

Prove or establish:

- Cache validity.
- Dependency invalidation correctness.
- Module interface consistency.
- Incremental IR correctness.
- Reuse of previously verified artifacts.
- Correct invalidation after semantic changes.

## **G24. Linking and whole-program correctness**

Define how separately compiled modules compose.

Verify:

- Symbol resolution.
- ABI compatibility.
- Relocations.
- Global initialization.
- Cross-module optimization.
- Dynamic linking.
- Shared-library interactions.
- Link-time dead-code elimination.
- Whole-program specialization.

## **G25. Self-hosting correctness**

Eventually, the compiler should be able to compile itself.

But self-hosting is not proof of correctness.

Your goal should be stronger:

1. The initial compiler is independently trusted or verified.
2. It compiles the compiler's source code.
3. The resulting compiler is verified against the same specifications.
4. The bootstrap chain is documented.
5. Repeated builds satisfy the reproducibility contract.
6. Compiler versions can be compared using semantic equivalence checks.

A particularly strong goal is a verified bootstrap chain where the trust assumptions are explicit and the resulting compiler can independently check its own proof obligations.

# **8. The optimization hierarchy**

I'd establish an explicit hierarchy so optimization development doesn't become a collection of random clever tricks.

## **Optimization priority model**

1. **Level 1: Eliminate unnecessary work**
   - Dead-code elimination
   - Constant propagation
   - Constant folding
   - Redundant computation elimination
   - Unreachable-code elimination
   - Compile-time evaluation
2. **Level 2: Improve algorithmic structure**
   - Loop transformations
   - Strength reduction
   - Common subexpression elimination
   - Sparse conditional constant propagation
   - Induction-variable optimization
   - Allocation elimination
3. **Level 3: Improve data movement**
   - Memory access optimization
   - Cache-aware transformations
   - Data layout optimization
   - Scalar replacement
   - Register promotion
   - Redundant load/store elimination
4. **Level 4: Improve machine execution**
   - Instruction selection
   - Register allocation
   - Instruction scheduling
   - SIMD vectorization
   - Branch optimization
   - Instruction fusion
5. **Level 5: Global optimization**
   - Interprocedural optimization
   - Whole-program analysis
   - Profile-guided optimization
   - Link-time optimization
   - Function specialization
   - Partial evaluation
6. **Level 6: Search-based optimization**
   - Superoptimization
   - Equality saturation
   - Verified rewrite search
   - Target-specific instruction sequence search
   - Cost-model-guided exploration

This is a prioritization hierarchy, not a mandatory execution order. For example, allocation elimination can have enormous benefits and should not be artificially delayed until later.

I'd also give the optimizer an explicit budget:

\\[ \max_T \quad \frac{\text{Expected workload benefit}(T)} {\text{Compilation cost}(T)+\text{Engineering cost}(T)} \\]

Subject to:

\\[ \operatorname{SemanticsPreserved}(T) \\]

\\[ \operatorname{SafetyPreserved}(T) \\]

\\[ \operatorname{ResourceConstraintsSatisfied}(T) \\]

This is a useful foundation for deciding where to spend optimization effort.

# **9. Overall measurable success criteria**

I'd turn the entire project into a collection of acceptance targets.

| Area               | Success criterion                                          |
| ------------------ | ---------------------------------------------------------- |
| Language semantics | Every supported feature formally specified                 |
| Type safety        | Soundness theorem established                              |
| Memory safety      | Universal theorem under explicit assumptions               |
| Pass correctness   | Every production pass verified                             |
| Pass composition   | Pipeline semantic preservation established                 |
| Code generation    | Machine-code correctness established                       |
| Runtime            | Critical runtime components verified                       |
| Concurrency        | Memory model and preservation proofs established           |
| Performance        | Declared cost-model contracts satisfied                    |
| Benchmarks         | No unexplained significant regressions                     |
| Optimization       | Benefit measured against compilation and runtime costs     |
| Resource usage     | Defined bounds and failure behavior                        |
| Reproducibility    | Identical builds under declared conditions                 |
| Verification       | Independently checkable proof artifacts                    |
| Bootstrapping      | Verified bootstrap chain                                   |
| Security           | Malformed inputs cannot violate compiler safety invariants |

I'd add a few particularly ambitious stretch goals:

- Machine-checkable proofs for the entire supported language implementation.
- Verified instruction selection against formal ISA semantics.
- Verified register allocation.
- Verified linker and relocation handling.
- Verified runtime and standard library.
- Proof-carrying compiler outputs.
- Formally verified optimization search.
- Automated discovery of counterexamples for incorrect optimization passes.
- Proof-preserving incremental compilation.
- Reproducible, independently verifiable bootstrap builds.

# **10. The most important architectural decision: separate correctness from optimization**

I would make this a foundational architectural principle.

## **Proposed compiler verification architecture**

**Source language**

Formal grammar, typing rules, operational semantics

**Verified frontend**

Parsing, type checking, semantic validation

**Canonical intermediate representation**

Formal semantic contract

Well-defined values, effects, memory, control flow and invariants

**Optimization engine**

Transformation

Semantic proof

Cost analysis

Performance evidence

Only accepted transformations enter the production pipeline

**Verified lowering**

Machine-independent IR to target-specific IR

**Verified backend**

Instruction selection, register allocation, code emission

**Executable binary**

Verified semantic preservation

Correctness certificate and build provenance

The critical idea is that optimization must not be allowed to weaken correctness guarantees.

An optimization can fail its performance contract and be rejected.

It can fail its semantic proof and be rejected.

It can exceed its resource budget and be rejected.

The compiler should never silently trade correctness for speed.

# **11. My recommended project-wide requirements**

I'd actually adopt these as the overarching rules of the project:

1. **Correctness is universal:** prove properties for all programs and inputs within explicitly defined domains, not just tested examples.
2. **Semantics come first:** no implementation feature exists without a formal specification.
3. **Every transformation is accountable:** every pass must establish semantic preservation.
4. **Performance is measurable:** optimization claims require explicit cost models and empirical evidence.
5. **No gratuitous complexity:** every optimization must justify its implementation, verification, and maintenance costs.
6. **No hidden assumptions:** external dependencies and trusted components must be explicitly identified.
7. **Failures are defined:** invalid input, resource exhaustion, and failed verification must have specified outcomes.
8. **Proofs compose:** correctness of individual components must establish correctness of the complete pipeline.
9. **The optimizer is not omniscient:** optimization quality must be measured against declared workloads, targets, and search limits.
10. **Self-hosting is not self-certification:** bootstrap correctness must be independently established.
11. **No performance regression gets ignored:** regressions require investigation, explanation, and explicit acceptance.
12. **The specification is authoritative:** implementation behavior must conform to the specification, never the other way around.

## **12. One additional goal I'd strongly recommend**



Critical

**G26. Verified compiler evolution**



Your compiler will change. Its language will grow, its IR will evolve, its optimizer will acquire new passes, and its backend will target new processors.

You need to verify not only individual versions, but also the correctness of changes between versions.

Establish:

- Specification versioning.
- Proof dependency tracking.
- Regression detection.
- Compatibility verification.
- IR migration correctness.
- Proof invalidation when assumptions change.
- Re-verification of affected components.
- Historical proof reproducibility.

In particular, changing an IR invariant should automatically identify every theorem and optimization whose proof depends on that invariant.

Otherwise, you end up with a compiler whose old proofs are sitting in a folder looking reassuring while no longer proving what you think they prove.
