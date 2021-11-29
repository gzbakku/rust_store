

pub mod parser;
pub mod reader;
pub mod workers;

pub use reader::{Reader,PointerType,Write};
use std::time::Instant;