//! Asset pipeline: sass compilation and image processing.
//!
//! Both submodules are gated behind cargo features (`sass`, `images`). When
//! the feature is disabled the corresponding `compile_*` function is a no-op
//! that returns `Ok(())`.

pub mod sass;
pub mod images;
