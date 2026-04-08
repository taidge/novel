pub mod container;
pub mod file_embed;
pub mod highlight;
pub mod parser;

pub use crate::plugin::ContainerDirective;
pub use parser::{MarkdownProcessor, collect_internal_links};
