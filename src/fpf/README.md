# First Principles Framework (FPF) - Core

A pure-Rust implementation of the **First Principles Framework (2026 Spec)**. This is the "Constitutional Layer" of the agency, providing the formal ontology and calculus for auditable intelligence.

## 🏛️ Holonic Ontology (`holon.rs`, `context.rs`)

The system follows a strict hierarchy of primitives:
- **U.Entity**: The baseline primitive of distinction.
- **U.Holon**: Units of composition (Entity + Boundary).
- **U.System**: Operational/physical holons capable of action.
- **U.Episteme**: Knowledge holons representing passive content (claims, evidence).

## ⚖️ Governance & Assurance (`assurance.rs`, `governance.rs`)

- **Boundary Norm Square**: Segregates claims into **L**aws, **A**dmissibility, **D**eontics, and **W**ork-Effects.
- **F-G-R Triad**: Calculates trust through **F**ormality (ordinal), **G**-Scope (coverage), and **R**eliability (ratio).
- **Adjudication**: Formal logic for independent peer-review of agent work.

## 🏗️ Composition & Aggregation (`aggregation.rs`, `mereology.rs`)

- **Universal Algebra (Γ)**: Provides lawful operators for combining holons.
- **Weakest-Link Bound**: Ensures the whole never outperforms its frailest component in safety-critical paths.
- **Invariant Quintet**: Enforces Idempotence, Commutativity, Locality, Weakest-Link, and Monotonicity across all aggregators.

## 📈 Multi-View Publication Kit (MVPK) (`mvpk.rs`)

Standardizes how morphisms are projected to different audiences:
- **PlainView (P)**: Explanatory prose for humans.
- **TechCard (T)**: Typed catalog entries for machines.
- **AssuranceLane (A)**: Evidence bindings and R-scores.
