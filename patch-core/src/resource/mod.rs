use crate::error::PatchResult;

pub mod synthetic;

pub trait ResourceText {
    fn set_text(&mut self, key: &str, value: &str) -> PatchResult<()>;
}
