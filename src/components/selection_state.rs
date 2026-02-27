use super::control;

pub fn resolve_optional_text(
    id: &str,
    slot: &str,
    controlled: bool,
    controlled_value: Option<String>,
    default_value: Option<String>,
) -> Option<String> {
    control::optional_text_state(
        id,
        slot,
        controlled.then_some(controlled_value),
        default_value,
    )
}

pub fn apply_optional_text(id: &str, slot: &str, controlled: bool, next: Option<String>) -> bool {
    if controlled {
        return false;
    }
    control::set_optional_text_state(id, slot, next);
    true
}

pub fn resolve_list(
    id: &str,
    slot: &str,
    controlled: bool,
    controlled_values: Vec<String>,
    default_values: Vec<String>,
) -> Vec<String> {
    control::list_state(
        id,
        slot,
        controlled.then_some(controlled_values),
        default_values,
    )
}

pub fn apply_list(id: &str, slot: &str, controlled: bool, next: Vec<String>) -> bool {
    if controlled {
        return false;
    }
    control::set_list_state(id, slot, next);
    true
}

pub fn resolve_usize(
    id: &str,
    slot: &str,
    controlled: bool,
    controlled_value: usize,
    default_value: usize,
) -> usize {
    control::usize_state(
        id,
        slot,
        controlled.then_some(controlled_value),
        default_value,
    )
}

pub fn apply_usize(id: &str, slot: &str, controlled: bool, next: usize) -> bool {
    if controlled {
        return false;
    }
    control::set_usize_state(id, slot, next);
    true
}

pub fn resolve_optional_usize(
    id: &str,
    slot: &str,
    controlled_value: Option<Option<usize>>,
    default_value: Option<usize>,
) -> Option<usize> {
    control::optional_usize_state(id, slot, controlled_value, default_value)
}

pub fn apply_optional_usize(id: &str, slot: &str, controlled: bool, next: Option<usize>) -> bool {
    if controlled {
        return false;
    }
    control::set_optional_usize_state(id, slot, next);
    true
}
