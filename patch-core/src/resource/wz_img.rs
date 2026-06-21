use std::cell::RefCell;
use std::path::Path;

use wz_reader::property::string::resolve_string_from_node;
use wz_reader::util::walk_node;
use wz_reader::{WzNode, WzNodeArc};

use crate::error::{PatchError, PatchResult};
use crate::resource::export::{RawTextNode, TargetResource};

pub fn read_img_text_nodes(
    game_dir: &Path,
    target: TargetResource,
) -> PatchResult<Vec<RawTextNode>> {
    let resource_path = game_dir.join(target.relative_path);
    if !resource_path.is_file() {
        return Err(PatchError::Validation(format!(
            "missing target resource: {}",
            resource_path.display()
        )));
    }

    let root: WzNodeArc = WzNode::from_img_file(&resource_path, None, None)
        .map_err(|error| {
            PatchError::Validation(format!(
                "cannot parse img resource {}: {}",
                target.relative_path, error
            ))
        })?
        .into();
    let nodes = RefCell::new(Vec::new());

    walk_node(&root, true, &|node: &WzNodeArc| {
        if let Ok(value) = resolve_string_from_node(node) {
            let path = node.read().unwrap().get_full_path();
            nodes.borrow_mut().push(RawTextNode {
                resource: target.relative_path.to_owned(),
                path,
                value,
            });
        }
    });

    Ok(nodes.into_inner())
}
