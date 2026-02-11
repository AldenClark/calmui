use gpui::{Div, Styled, div};

pub fn h_stack() -> Div {
    div().flex().flex_row().items_center()
}

pub fn v_stack() -> Div {
    div().flex().flex_col()
}
