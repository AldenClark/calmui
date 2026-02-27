use std::collections::BTreeSet;

fn collect_component_module_files() -> BTreeSet<String> {
    include_str!("../../src/components/mod.rs")
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("mod ") && line.ends_with(';'))
        .filter_map(|line| {
            let name = line
                .trim_start_matches("mod ")
                .trim_end_matches(';')
                .trim()
                .to_string();
            (!name.starts_with("test_") && name != "tests").then_some(format!("{name}.rs"))
        })
        .collect()
}

fn collect_files_from_test_source(src: &str) -> BTreeSet<String> {
    src.lines()
        .map(str::trim)
        .filter_map(|line| {
            let prefix = "file: \"";
            line.find(prefix).and_then(|start| {
                let rest = &line[start + prefix.len()..];
                rest.find('"').map(|end| rest[..end].to_string())
            })
        })
        .collect()
}

#[test]
fn depth_budget_and_flattening_cover_all_component_modules() {
    let component_files = collect_component_module_files();
    let budget_files = collect_files_from_test_source(include_str!("layout_depth_budget.rs"));
    let invariant_files = collect_files_from_test_source(include_str!("flattening_invariants.rs"));

    for file in &component_files {
        assert!(
            budget_files.contains(file),
            "layout_depth_budget.rs missing component file budget entry: {}",
            file
        );
        assert!(
            invariant_files.contains(file),
            "flattening_invariants.rs missing component file invariant entry: {}",
            file
        );
    }
}
