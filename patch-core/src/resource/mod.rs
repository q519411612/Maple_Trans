use crate::error::PatchResult;

pub mod export;
pub mod key;
pub mod probe;
pub mod synthetic;
pub mod wz_img;

pub trait ResourceText {
    fn set_text(&mut self, key: &str, value: &str) -> PatchResult<()>;
}
