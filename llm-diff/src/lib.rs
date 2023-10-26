pub mod diff;
pub mod hunk;
pub use diff::FileDiff;
pub use hunk::{ApplyError, Hunk, ParseError};
