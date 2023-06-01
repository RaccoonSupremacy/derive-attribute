mod shared;
pub use shared::*;

// #[cfg(not(any(feature = "syn_1", feature = "syn_2")))]
// compile_error!("Must use choose a syn crate version as a feature");

#[cfg(feature = "syn_1")]
mod syn_1;
#[cfg(feature = "syn_1")]
pub use syn_1::Syn1;

#[cfg(feature = "syn_2")]
mod syn_2;
#[cfg(feature = "syn_2")]
pub use syn_2::Syn2;

pub mod reexports {
    pub use proc_macro2;
}

