use std::collections::BTreeMap;

use crate::error::{PatchError, PatchResult};
use crate::resource::ResourceText;

#[derive(Debug, Clone)]
pub struct SyntheticResource {
    values: BTreeMap<String, String>,
}

impl SyntheticResource {
    pub fn from_str(input: &str) -> PatchResult<Self> {
        let values = serde_json::from_str(input).map_err(|error| {
            PatchError::Validation(format!("invalid synthetic resource: {}", error))
        })?;
        Ok(Self { values })
    }

    pub fn to_string_pretty(&self) -> PatchResult<String> {
        serde_json::to_string_pretty(&self.values).map_err(|error| {
            PatchError::Validation(format!("cannot write synthetic resource: {}", error))
        })
    }
}

impl ResourceText for SyntheticResource {
    fn set_text(&mut self, key: &str, value: &str) -> PatchResult<()> {
        let slot = self
            .values
            .get_mut(key)
            .ok_or_else(|| PatchError::Validation(format!("missing resource key: {}", key)))?;
        *slot = value.to_owned();
        Ok(())
    }
}
