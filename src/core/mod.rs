//! Core components of the Venture OS Kernel

pub mod governance;
pub mod kernel;
pub mod provenance;
pub mod registry;
pub mod semantic_model;

pub use governance::Governance;
pub use kernel::{Kernel, KernelError};
pub use provenance::Provenance;
pub use registry::CapabilityRegistry;
pub use semantic_model::SemanticModel;
