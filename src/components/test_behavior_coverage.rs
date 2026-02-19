fn extract_type_names(src: &str, marker: &str) -> Vec<String> {
    src.lines()
        .map(str::trim)
        .filter_map(|line| {
            line.find(marker).and_then(|start| {
                let rest = &line[start + marker.len()..];
                rest.find(">();").map(|end| rest[..end].trim().to_string())
            })
        })
        .collect()
}

fn assert_behavior_matrix_mentions_all(marker: &str, label: &str) {
    let contract_src = include_str!("test_contract_matrix.rs");
    let behavior_src = include_str!("test_behavior_matrix.rs");
    let names = extract_type_names(contract_src, marker);
    for name in names {
        assert!(
            behavior_src.contains(&name),
            "behavior matrix is missing {label} coverage mention for `{name}`",
        );
    }
}

#[test]
fn behavior_matrix_mentions_all_render_once_components() {
    assert_behavior_matrix_mentions_all("assert_render_once::<", "render_once");
}

#[test]
fn behavior_matrix_mentions_all_disableable_components() {
    assert_behavior_matrix_mentions_all("assert_disableable::<", "disableable");
}

#[test]
fn behavior_matrix_mentions_all_openable_components() {
    assert_behavior_matrix_mentions_all("assert_openable::<", "openable");
}

#[test]
fn behavior_matrix_mentions_all_field_like_components() {
    assert_behavior_matrix_mentions_all("assert_field_like::<", "field_like");
}

#[test]
fn behavior_matrix_mentions_all_style_contract_components() {
    assert_behavior_matrix_mentions_all("assert_varianted::<", "varianted");
    assert_behavior_matrix_mentions_all("assert_sized::<", "sized");
    assert_behavior_matrix_mentions_all("assert_radiused::<", "radiused");
}
