use std::fs;
use std::path::Path;

fn component_rs_files() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/components");
    let mut files = Vec::new();

    let Ok(entries) = fs::read_dir(&root) else {
        return files;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        files.push(path.to_string_lossy().to_string());
    }
    files.sort();
    files
}

#[test]
fn component_internal_id_binding_uses_id_ctx() {
    let forbidden_patterns = [
        ".with_id(self.id.slot(",
        ".with_id(self.id.slot_index(",
        ".with_id(self.tree_id.slot(",
        ".with_id(self.tree_id.slot_index(",
    ];

    let mut hits = Vec::new();
    for file in component_rs_files() {
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };

        for pattern in forbidden_patterns {
            if content.contains(pattern) {
                hits.push(format!("{file}: contains `{pattern}`"));
            }
        }
    }

    assert!(
        hits.is_empty(),
        "Found legacy internal id binding patterns:\\n{}",
        hits.join("\n")
    );
}
