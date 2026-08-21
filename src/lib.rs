//! Venture OS Kernel - The core decision-making engine
//!
//! This crate implements the Venture OS Kernel, a modular monolith written in Rust
//! that transforms business ideas into executable plans while maintaining strict
//! governance, provenance, and contract adherence.

// Core modules
pub mod capabilities;
pub mod contracts;
pub mod core;
pub mod runtime;

// Re-export key components
pub use core::governance::Governance;
pub use core::kernel::Kernel;
pub use core::provenance::Provenance;
pub use core::registry::CapabilityRegistry;
pub use core::semantic_model::SemanticModel;

/// The main entry point for the Venture OS Kernel
pub fn new() -> Kernel {
    Kernel::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_creates_kernel() {
        let _kernel = new();
        // Basic test to ensure kernel creation works - removed trivial assertion
    }
}
