use crate::error::{PatchError, PatchResult};

pub fn stable_text_key(resource: &str, path: &str) -> PatchResult<String> {
    let prefix = resource_prefix(resource)?;
    let mut parts = path.split('/').collect::<Vec<_>>();

    if parts.len() < 3 {
        return Err(PatchError::Validation(format!(
            "invalid text path for key: {path}"
        )));
    }

    let field = parts.pop().expect("field exists when len is checked");
    validate_text_field(field)?;

    let id = parts
        .last()
        .ok_or_else(|| PatchError::Validation(format!("missing text id in path: {path}")))?;

    Ok(format!("{prefix}.{id}.{field}"))
}

fn resource_prefix(resource: &str) -> PatchResult<&'static str> {
    match resource {
        "Data/String/Eqp.img" => Ok("string.eqp"),
        "Data/String/Consume.img" => Ok("string.consume"),
        "Data/String/Etc.img" => Ok("string.etc"),
        "Data/String/Ins.img" => Ok("string.ins"),
        "Data/String/Cash.img" => Ok("string.cash"),
        "Data/String/Pet.img" => Ok("string.pet"),
        "Data/String/Skill.img" => Ok("string.skill"),
        "Data/String/Map.img" => Ok("string.map"),
        "Data/Quest/QuestInfo.img" => Ok("quest.info"),
        _ => Err(PatchError::Validation(format!(
            "unsupported text resource: {resource}"
        ))),
    }
}

fn validate_text_field(field: &str) -> PatchResult<()> {
    match field {
        "name" | "desc" | "h" | "mapName" | "streetName" => Ok(()),
        _ => Err(PatchError::Validation(format!(
            "unsupported text field: {field}"
        ))),
    }
}
