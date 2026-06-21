use crate::error::{PatchError, PatchResult};
use crate::manifest::LengthPolicy;

pub struct TextTarget<'a> {
    pub key: &'a str,
    pub text: &'a str,
}

pub fn validate_text_length(target: TextTarget<'_>, policy: LengthPolicy) -> PatchResult<()> {
    let max_chars = match policy {
        LengthPolicy::StrictName => 16,
        LengthPolicy::ExpandedText => 512,
    };

    let actual = target.text.chars().count();
    if actual > max_chars {
        return Err(PatchError::Validation(format!(
            "text length exceeds {} for {}: {} > {}",
            policy.as_str(),
            target.key,
            actual,
            max_chars
        )));
    }

    Ok(())
}
