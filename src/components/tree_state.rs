use std::collections::BTreeSet;

use super::control;

#[derive(Clone, Debug)]
pub struct TreeVisibleNode {
    pub value: String,
    pub parent: Option<String>,
    pub disabled: bool,
    pub has_children: bool,
    pub first_child: Option<String>,
}

#[derive(Default)]
pub struct TreeKeyResult {
    pub next_selected: Option<String>,
    pub next_expanded: Option<Vec<String>>,
}

pub fn resolve_selected(
    id: &str,
    selected_controlled: bool,
    controlled_value: Option<String>,
    default_value: Option<String>,
) -> Option<String> {
    control::optional_text_state(
        id,
        "value",
        selected_controlled.then_some(controlled_value),
        default_value,
    )
}

pub fn resolve_expanded(
    id: &str,
    expanded_controlled: bool,
    controlled_values: Vec<String>,
    default_values: Vec<String>,
) -> Vec<String> {
    control::list_state(
        id,
        "expanded",
        expanded_controlled.then_some(controlled_values),
        default_values,
    )
}

pub fn toggled_values(mut current: Vec<String>, value: &str) -> Vec<String> {
    if let Some(index) = current.iter().position(|item| item == value) {
        current.remove(index);
    } else {
        current.push(value.to_string());
    }
    current
}

pub fn apply_selected(id: &str, selected_controlled: bool, selected: Option<String>) -> bool {
    if selected_controlled {
        return false;
    }
    control::set_optional_text_state(id, "value", selected);
    true
}

pub fn apply_expanded(id: &str, expanded_controlled: bool, expanded: Vec<String>) -> bool {
    if expanded_controlled {
        return false;
    }
    control::set_list_state(id, "expanded", expanded);
    true
}

pub fn key_transition(
    key: &str,
    current_selected: Option<&str>,
    visible_nodes: &[TreeVisibleNode],
    current_expanded: &[String],
) -> TreeKeyResult {
    if visible_nodes.is_empty() {
        return TreeKeyResult::default();
    }

    let enabled_values = visible_nodes
        .iter()
        .filter(|node| !node.disabled)
        .map(|node| node.value.as_str())
        .collect::<Vec<_>>();
    if enabled_values.is_empty() {
        return TreeKeyResult::default();
    }

    let current_index = current_selected
        .and_then(|selected| enabled_values.iter().position(|value| *value == selected));
    let expanded_set = current_expanded
        .iter()
        .map(|value| value.as_str())
        .collect::<BTreeSet<_>>();
    let mut result = TreeKeyResult::default();

    match key {
        "up" => {
            if let Some(index) = current_index {
                if index > 0 {
                    result.next_selected = Some(enabled_values[index - 1].to_string());
                } else {
                    result.next_selected = Some(enabled_values[0].to_string());
                }
            } else {
                result.next_selected = Some(enabled_values[0].to_string());
            }
        }
        "down" => {
            if let Some(index) = current_index {
                let next_index = (index + 1).min(enabled_values.len().saturating_sub(1));
                result.next_selected = Some(enabled_values[next_index].to_string());
            } else {
                result.next_selected = Some(enabled_values[0].to_string());
            }
        }
        "home" => {
            result.next_selected = Some(enabled_values[0].to_string());
        }
        "end" => {
            if let Some(last) = enabled_values.last() {
                result.next_selected = Some((*last).to_string());
            }
        }
        "right" => {
            if let Some(selected_value) = current_selected
                && let Some(node) = visible_nodes
                    .iter()
                    .find(|node| node.value == selected_value)
            {
                let is_expanded = expanded_set.contains(node.value.as_str());
                if node.has_children && !is_expanded {
                    let mut next = current_expanded.to_vec();
                    next.push(node.value.clone());
                    next.sort();
                    next.dedup();
                    result.next_expanded = Some(next);
                } else if node.has_children
                    && is_expanded
                    && let Some(first_child) = node.first_child.as_ref()
                {
                    result.next_selected = Some(first_child.clone());
                }
            }
        }
        "left" => {
            if let Some(selected_value) = current_selected
                && let Some(node) = visible_nodes
                    .iter()
                    .find(|node| node.value == selected_value)
            {
                let is_expanded = expanded_set.contains(node.value.as_str());
                if node.has_children && is_expanded {
                    let mut next = current_expanded.to_vec();
                    next.retain(|value| value != node.value.as_str());
                    result.next_expanded = Some(next);
                } else if let Some(parent) = node.parent.as_ref() {
                    result.next_selected = Some(parent.clone());
                }
            }
        }
        "enter" | "space" => {
            if current_selected.is_none() {
                result.next_selected = Some(enabled_values[0].to_string());
            } else if key == "space"
                && let Some(selected_value) = current_selected
                && let Some(node) = visible_nodes
                    .iter()
                    .find(|node| node.value == selected_value)
                && node.has_children
            {
                result.next_expanded = Some(toggled_values(
                    current_expanded.to_vec(),
                    node.value.as_str(),
                ));
            }
        }
        _ => {}
    }

    result
}
