mod source;

pub use source::*;

pub trait Model: Send + Sync {}
