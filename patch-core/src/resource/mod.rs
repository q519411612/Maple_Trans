use crate::error::PatchResult;

pub mod synthetic;
pub mod export;
pub mod key;

pub trait ResourceText {
    fn set_text(&mut self, key: &str, value: &str) -> PatchResult<()>;
}
