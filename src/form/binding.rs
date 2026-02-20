use std::str::FromStr;
use std::sync::Arc;

use gpui::{SharedString, Window};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use super::controller::{FieldKey, FormController, FormResult, read_lock};
use super::validation::{FieldLens, ValidationError};
use crate::components::{
    Checkbox, MultiSelect, NumberInput, PasswordInput, Select, Switch, TextInput, Textarea,
};
use crate::contracts::FieldLike;

impl<T, E> FormController<T, E>
where
    T: Clone + Send + Sync + 'static,
    E: ValidationError,
{
    pub fn field_error_for_display<L>(&self, lens: L) -> FormResult<Option<SharedString>>
    where
        L: FieldLens<T>,
    {
        self.display_error_message(lens.key())
    }

    pub fn bind_text_input<L>(&self, lens: L, input: TextInput) -> FormResult<TextInput>
    where
        L: FieldLens<T, Value = SharedString>,
    {
        let key = lens.key();
        let snapshot = self.snapshot()?;
        let value = lens.get(&snapshot.model).clone();
        let controller = self.clone();
        let bound = input
            .value(value)
            .on_change(move |next, _, _| drop(controller.set(lens, next)));
        self.apply_fieldlike_presentation(key, bound)
    }

    pub fn bind_password_input<L>(&self, lens: L, input: PasswordInput) -> FormResult<PasswordInput>
    where
        L: FieldLens<T, Value = SharedString>,
    {
        let key = lens.key();
        let snapshot = self.snapshot()?;
        let value = lens.get(&snapshot.model).clone();
        let controller = self.clone();
        let bound = input
            .value(value)
            .on_change(move |next, _, _| drop(controller.set(lens, next)));
        self.apply_fieldlike_presentation(key, bound)
    }

    pub fn bind_textarea<L>(&self, lens: L, textarea: Textarea) -> FormResult<Textarea>
    where
        L: FieldLens<T, Value = SharedString>,
    {
        let key = lens.key();
        let snapshot = self.snapshot()?;
        let value = lens.get(&snapshot.model).clone();
        let controller = self.clone();
        let bound = textarea
            .value(value)
            .on_change(move |next, _, _| drop(controller.set(lens, next)));
        self.apply_fieldlike_presentation(key, bound)
    }

    pub fn bind_number_input<L>(&self, lens: L, input: NumberInput) -> FormResult<NumberInput>
    where
        L: FieldLens<T, Value = Decimal>,
    {
        let key = lens.key();
        let snapshot = self.snapshot()?;
        let value = lens.get(&snapshot.model).to_f64().unwrap_or(0.0);
        let controller = self.clone();
        let bound = input.value(value).on_change(move |next, _, _| {
            if let Some(parsed) = decimal_from_f64(next) {
                drop(controller.set(lens, parsed));
            }
        });
        self.apply_fieldlike_presentation(key, bound)
    }

    pub fn bind_checkbox<L>(&self, lens: L, checkbox: Checkbox) -> FormResult<Checkbox>
    where
        L: FieldLens<T, Value = bool>,
    {
        let checked = *lens.get(&self.snapshot()?.model);
        let controller = self.clone();
        Ok(checkbox
            .checked(checked)
            .on_change(move |next, _, _| drop(controller.set(lens, next))))
    }

    pub fn bind_switch<L>(&self, lens: L, switch: Switch) -> FormResult<Switch>
    where
        L: FieldLens<T, Value = bool>,
    {
        let checked = *lens.get(&self.snapshot()?.model);
        let controller = self.clone();
        Ok(switch
            .checked(checked)
            .on_change(move |next, _, _| drop(controller.set(lens, next))))
    }

    pub fn bind_select<L>(&self, lens: L, select: Select) -> FormResult<Select>
    where
        L: FieldLens<T, Value = SharedString>,
    {
        let key = lens.key();
        let snapshot = self.snapshot()?;
        let value = lens.get(&snapshot.model).clone();
        let controller = self.clone();
        let bound = select
            .value(value)
            .on_change(move |next, _, _| drop(controller.set(lens, next)));
        self.apply_fieldlike_presentation(key, bound)
    }

    pub fn bind_multiselect<L>(&self, lens: L, multiselect: MultiSelect) -> FormResult<MultiSelect>
    where
        L: FieldLens<T, Value = Vec<SharedString>>,
    {
        let key = lens.key();
        let snapshot = self.snapshot()?;
        let values = lens.get(&snapshot.model).clone();
        let controller = self.clone();
        let bound = multiselect
            .values(values)
            .on_change(move |next, _, _| drop(controller.set(lens, next)));
        self.apply_fieldlike_presentation(key, bound)
    }

    pub fn bind_text_input_submit<L, F>(
        &self,
        lens: L,
        input: TextInput,
        on_submit: F,
    ) -> FormResult<TextInput>
    where
        L: FieldLens<T, Value = SharedString>,
        F: Fn(&Self, &mut Window, &mut gpui::App) + Send + Sync + 'static,
    {
        let submit_handler = Arc::new(on_submit);
        let controller = self.clone();
        let bound = self.bind_text_input(lens, input)?;
        Ok(bound.on_submit(move |_value, window, cx| submit_handler(&controller, window, cx)))
    }

    pub fn bind_password_input_submit<L, F>(
        &self,
        lens: L,
        input: PasswordInput,
        on_submit: F,
    ) -> FormResult<PasswordInput>
    where
        L: FieldLens<T, Value = SharedString>,
        F: Fn(&Self, &mut Window, &mut gpui::App) + Send + Sync + 'static,
    {
        let submit_handler = Arc::new(on_submit);
        let controller = self.clone();
        let bound = self.bind_password_input(lens, input)?;
        Ok(bound.on_submit(move |_value, window, cx| submit_handler(&controller, window, cx)))
    }

    fn apply_fieldlike_presentation<C>(&self, key: FieldKey, mut component: C) -> FormResult<C>
    where
        C: FieldLike,
    {
        if let Some(description) = read_lock(
            &self.field_descriptions,
            "reading field description for binding",
        )?
        .get(&key)
        .cloned()
        {
            component = component.description(description);
        }

        if read_lock(&self.required_fields, "reading required fields for binding")?.contains(&key) {
            component = component.required(true);
        }

        if let Some(error) = self.display_error_message(key)? {
            component = component.error(error);
        }

        Ok(component)
    }

    fn display_error_message(&self, key: FieldKey) -> FormResult<Option<SharedString>> {
        let state = read_lock(&self.state, "reading display error message")?;
        let Some(meta) = state.field_meta.get(&key) else {
            return Ok(None);
        };
        if !meta.touched && state.submit_count == 0 {
            return Ok(None);
        }
        Ok(meta.errors.first().map(ValidationError::message))
    }
}

fn decimal_from_f64(value: f64) -> Option<Decimal> {
    if !value.is_finite() {
        return None;
    }
    Decimal::from_str(&format!("{value:.18}")).ok()
}
