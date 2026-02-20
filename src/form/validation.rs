use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use futures_timer::Delay;
use gpui::SharedString;

use super::controller::{
    AsyncFieldValidatorEntry, AsyncFieldValidatorFn, FieldKey, FormController, FormResult,
    RevalidateMode, SyncFieldValidatorFn, SyncFormValidatorFn, ValidationMode, ValidationTicket,
    first_error_key, read_lock, write_lock,
};

pub trait ValidationError: Clone + Send + Sync + 'static {
    fn message(&self) -> SharedString;
}

pub trait FieldLens<T>: Copy + Send + Sync + 'static {
    type Value: Clone + PartialEq + Send + Sync + 'static;

    fn key(self) -> FieldKey;
    fn get<'a>(self, model: &'a T) -> &'a Self::Value;
    fn set(self, model: &mut T, value: Self::Value);
}

pub trait FormModel: Clone + Send + Sync + 'static {
    type Fields;

    fn fields() -> Self::Fields;
}

pub trait FieldValidator<T, L, E>: Send + Sync
where
    L: FieldLens<T>,
    E: ValidationError,
{
    fn validate(&self, model: &T, value: &L::Value) -> Result<(), E>;
}

impl<T, L, E, F> FieldValidator<T, L, E> for F
where
    L: FieldLens<T>,
    E: ValidationError,
    F: for<'a> Fn(&'a T, &'a L::Value) -> Result<(), E> + Send + Sync,
{
    fn validate(&self, model: &T, value: &L::Value) -> Result<(), E> {
        (self)(model, value)
    }
}

pub trait FormValidator<T, E>: Send + Sync
where
    E: ValidationError,
{
    fn validate(&self, model: &T) -> Vec<(FieldKey, E)>;
}

impl<T, E, F> FormValidator<T, E> for F
where
    E: ValidationError,
    F: Fn(&T) -> Vec<(FieldKey, E)> + Send + Sync,
{
    fn validate(&self, model: &T) -> Vec<(FieldKey, E)> {
        (self)(model)
    }
}

pub type BoxedValidationFuture<'a, E> = Pin<Box<dyn Future<Output = Result<(), E>> + Send + 'a>>;

pub trait AsyncFieldValidator<T, L, E>: Send + Sync
where
    L: FieldLens<T>,
    E: ValidationError,
{
    type Fut<'a>: Future<Output = Result<(), E>> + Send + 'a
    where
        Self: 'a,
        T: 'a,
        L::Value: 'a;

    fn validate<'a>(&'a self, model: &'a T, value: &'a L::Value) -> Self::Fut<'a>;
}

impl<T, L, E, F> AsyncFieldValidator<T, L, E> for F
where
    L: FieldLens<T>,
    E: ValidationError,
    F: for<'a> Fn(&'a T, &'a L::Value) -> BoxedValidationFuture<'a, E> + Send + Sync,
{
    type Fut<'a>
        = BoxedValidationFuture<'a, E>
    where
        Self: 'a,
        T: 'a,
        L::Value: 'a;

    fn validate<'a>(&'a self, model: &'a T, value: &'a L::Value) -> Self::Fut<'a> {
        (self)(model, value)
    }
}

impl<T, E> FormController<T, E>
where
    T: Clone + Send + Sync + 'static,
    E: ValidationError,
{
    pub fn register_field_validator<L, V>(&self, lens: L, validator: V) -> FormResult<()>
    where
        L: FieldLens<T>,
        V: FieldValidator<T, L, E> + 'static,
    {
        let key = lens.key();
        let validator = std::sync::Arc::new(validator);
        let wrapped: SyncFieldValidatorFn<T, E> =
            std::sync::Arc::new(move |model: &T| validator.validate(model, lens.get(model)));
        let mut validators =
            write_lock(&self.sync_field_validators, "registering field validator")?;
        validators.entry(key).or_default().push(wrapped);
        Ok(())
    }

    pub fn register_async_field_validator<L, V>(&self, lens: L, validator: V) -> FormResult<()>
    where
        L: FieldLens<T>,
        V: AsyncFieldValidator<T, L, E> + 'static,
    {
        self.register_async_field_validator_with_debounce(lens, 0, validator)
    }

    pub fn register_async_field_validator_with_debounce<L, V>(
        &self,
        lens: L,
        debounce_ms: u64,
        validator: V,
    ) -> FormResult<()>
    where
        L: FieldLens<T>,
        V: AsyncFieldValidator<T, L, E> + 'static,
    {
        let key = lens.key();
        let validator = std::sync::Arc::new(validator);
        let wrapped: AsyncFieldValidatorFn<T, E> = std::sync::Arc::new(move |model: T| {
            let value = lens.get(&model).clone();
            let validator = validator.clone();
            Box::pin(async move { validator.validate(&model, &value).await })
        });
        let entry = AsyncFieldValidatorEntry {
            debounce: Duration::from_millis(debounce_ms),
            validator: wrapped,
        };
        let mut validators = write_lock(
            &self.async_field_validators,
            "registering async field validator",
        )?;
        validators.entry(key).or_default().push(entry);
        Ok(())
    }

    pub fn register_form_validator<V>(&self, validator: V) -> FormResult<()>
    where
        V: FormValidator<T, E> + 'static,
    {
        let validator = std::sync::Arc::new(validator);
        let wrapped: SyncFormValidatorFn<T, E> =
            std::sync::Arc::new(move |model: &T| validator.validate(model));
        let mut validators = write_lock(&self.form_validators, "registering form validator")?;
        validators.push(wrapped);
        Ok(())
    }

    pub fn register_dependency<S, D>(&self, source: S, dependent: D) -> FormResult<()>
    where
        S: FieldLens<T>,
        D: FieldLens<T>,
    {
        let mut dependencies = write_lock(&self.dependencies, "registering dependency")?;
        dependencies
            .entry(source.key())
            .or_default()
            .insert(dependent.key());
        Ok(())
    }

    pub fn set<L>(&self, lens: L, value: L::Value) -> FormResult<()>
    where
        L: FieldLens<T>,
    {
        let key = lens.key();
        {
            let mut state = write_lock(&self.state, "writing form model")?;
            lens.set(&mut state.model, value);
            let is_dirty = lens.get(&state.model) != lens.get(&state.initial_model);
            if is_dirty {
                state.dirty_fields.insert(key);
            } else {
                state.dirty_fields.remove(&key);
            }
            state.ensure_meta(key).dirty = is_dirty;
        }

        if self.options.validate_mode == ValidationMode::OnChange {
            let _ = self.validate_field_by_key(key)?;
        }
        if self.options.revalidate_mode == RevalidateMode::OnChange {
            self.revalidate_dependents(key)?;
        }
        Ok(())
    }

    pub fn touch<L>(&self, lens: L) -> FormResult<()>
    where
        L: FieldLens<T>,
    {
        let key = lens.key();
        {
            let mut state = write_lock(&self.state, "touching field")?;
            state.ensure_meta(key).touched = true;
        }

        if self.options.validate_mode == ValidationMode::OnBlur {
            let _ = self.validate_field_by_key(key)?;
        }
        if self.options.revalidate_mode == RevalidateMode::OnBlur {
            self.revalidate_dependents(key)?;
        }
        Ok(())
    }

    pub async fn set_async<L>(&self, lens: L, value: L::Value) -> FormResult<()>
    where
        L: FieldLens<T>,
    {
        let key = lens.key();
        self.set(lens, value)?;
        if self.options.validate_mode == ValidationMode::OnChange {
            let _ = self.validate_field_async_registered_by_key(key).await?;
        }
        if self.options.revalidate_mode == RevalidateMode::OnChange {
            self.revalidate_dependents_async(key).await?;
        }
        Ok(())
    }

    pub async fn touch_async<L>(&self, lens: L) -> FormResult<()>
    where
        L: FieldLens<T>,
    {
        let key = lens.key();
        self.touch(lens)?;
        if self.options.validate_mode == ValidationMode::OnBlur {
            let _ = self.validate_field_async_registered_by_key(key).await?;
        }
        if self.options.revalidate_mode == RevalidateMode::OnBlur {
            self.revalidate_dependents_async(key).await?;
        }
        Ok(())
    }

    pub fn validate_field<L>(&self, lens: L) -> FormResult<()>
    where
        L: FieldLens<T>,
    {
        let _ = self.validate_field_by_key(lens.key())?;
        Ok(())
    }

    pub async fn validate_field_async<L, V>(
        &self,
        lens: L,
        validator: &V,
    ) -> FormResult<ValidationTicket>
    where
        L: FieldLens<T>,
        V: AsyncFieldValidator<T, L, E>,
    {
        let key = lens.key();
        let (ticket, model, value) = {
            let mut state = write_lock(&self.state, "starting async validation")?;
            let next = ValidationTicket(
                state
                    .tickets
                    .get(&key)
                    .copied()
                    .unwrap_or(ValidationTicket(0))
                    .0
                    + 1,
            );
            state.tickets.insert(key, next);
            state.ensure_meta(key).validating = true;
            (next, state.model.clone(), lens.get(&state.model).clone())
        };

        let result = validator.validate(&model, &value).await;
        self.finish_async_validation(key, ticket, result)?;
        Ok(ticket)
    }

    pub async fn validate_field_async_registered<L>(
        &self,
        lens: L,
    ) -> FormResult<Vec<ValidationTicket>>
    where
        L: FieldLens<T>,
    {
        self.validate_field_async_registered_by_key(lens.key())
            .await
    }

    pub fn validate_form(&self) -> FormResult<bool> {
        let model = {
            read_lock(&self.state, "reading model for form validation")?
                .model
                .clone()
        };
        let field_validators = read_lock(
            &self.sync_field_validators,
            "reading field validators for form validation",
        )?
        .clone();
        let form_validators = read_lock(
            &self.form_validators,
            "reading form validators for form validation",
        )?
        .clone();

        let mut field_errors = BTreeMap::<FieldKey, Vec<E>>::new();
        for (key, validators) in field_validators {
            let mut errors = Vec::new();
            for validator in validators {
                if let Err(error) = validator(&model) {
                    errors.push(error);
                    if self.options.validate_first_error_only {
                        break;
                    }
                }
            }
            field_errors.insert(key, errors);
        }

        for validator in form_validators {
            for (key, error) in validator(&model) {
                field_errors.entry(key).or_default().push(error);
            }
        }

        {
            let mut state = write_lock(&self.state, "applying form validation result")?;
            let mut keys = state
                .field_meta
                .keys()
                .copied()
                .collect::<BTreeSet<FieldKey>>();
            keys.extend(field_errors.keys().copied());
            for key in keys {
                let meta = state.ensure_meta(key);
                meta.validating = false;
                meta.errors = field_errors.remove(&key).unwrap_or_default();
            }
            state.first_error = first_error_key(&state.field_meta);
        }

        Ok(self.snapshot()?.is_valid)
    }

    pub async fn validate_form_async(&self) -> FormResult<bool> {
        let _ = self.validate_form()?;
        let keys = read_lock(
            &self.async_field_validators,
            "reading async validator keys for form validation",
        )?
        .keys()
        .copied()
        .collect::<Vec<_>>();

        for key in keys {
            let _ = self.validate_field_async_registered_by_key(key).await?;
        }

        Ok(self.snapshot()?.is_valid)
    }

    pub(super) fn validate_field_by_key(&self, key: FieldKey) -> FormResult<bool> {
        let model = {
            read_lock(&self.state, "reading model for field validation")?
                .model
                .clone()
        };
        let validators = {
            read_lock(
                &self.sync_field_validators,
                "reading field validators for key validation",
            )?
            .get(&key)
            .cloned()
            .unwrap_or_default()
        };

        let mut errors = Vec::new();
        for validator in validators {
            if let Err(error) = validator(&model) {
                errors.push(error);
                if self.options.validate_first_error_only {
                    break;
                }
            }
        }

        let mut state = write_lock(&self.state, "writing field validation result")?;
        let meta = state.ensure_meta(key);
        meta.validating = false;
        meta.errors = errors;
        state.first_error = first_error_key(&state.field_meta);
        Ok(state
            .field_meta
            .get(&key)
            .is_none_or(|m| m.errors.is_empty()))
    }

    pub(super) fn revalidate_dependents(&self, source: FieldKey) -> FormResult<()> {
        let dependents = read_lock(&self.dependencies, "reading field dependencies")?
            .get(&source)
            .cloned()
            .unwrap_or_default();
        for dependent in dependents {
            let _ = self.validate_field_by_key(dependent)?;
        }
        Ok(())
    }

    pub(super) async fn revalidate_dependents_async(&self, source: FieldKey) -> FormResult<()> {
        let dependents = read_lock(&self.dependencies, "reading async field dependencies")?
            .get(&source)
            .cloned()
            .unwrap_or_default();
        for dependent in dependents {
            let _ = self
                .validate_field_async_registered_by_key(dependent)
                .await?;
        }
        Ok(())
    }

    pub(super) async fn validate_field_async_registered_by_key(
        &self,
        key: FieldKey,
    ) -> FormResult<Vec<ValidationTicket>> {
        let model = {
            read_lock(&self.state, "reading model for registered async validation")?
                .model
                .clone()
        };
        let validators = {
            read_lock(
                &self.async_field_validators,
                "reading registered async validators",
            )?
            .get(&key)
            .cloned()
            .unwrap_or_default()
        };

        let mut tickets = Vec::with_capacity(validators.len());
        for entry in validators {
            let ticket = {
                let mut state = write_lock(&self.state, "starting registered async validation")?;
                let next = ValidationTicket(
                    state
                        .tickets
                        .get(&key)
                        .copied()
                        .unwrap_or(ValidationTicket(0))
                        .0
                        + 1,
                );
                state.tickets.insert(key, next);
                state.ensure_meta(key).validating = true;
                next
            };

            if !entry.debounce.is_zero() {
                Delay::new(entry.debounce).await;
                if !self.is_latest_ticket(key, ticket)? {
                    continue;
                }
            }

            let result = (entry.validator)(model.clone()).await;
            self.finish_async_validation(key, ticket, result)?;
            tickets.push(ticket);
        }
        Ok(tickets)
    }

    fn is_latest_ticket(&self, key: FieldKey, ticket: ValidationTicket) -> FormResult<bool> {
        Ok(read_lock(&self.state, "checking latest validation ticket")?
            .tickets
            .get(&key)
            .copied()
            == Some(ticket))
    }

    pub(super) fn known_field_keys(&self) -> FormResult<BTreeSet<FieldKey>> {
        let mut keys = BTreeSet::new();
        keys.extend(
            read_lock(&self.sync_field_validators, "reading sync validator keys")?
                .keys()
                .copied(),
        );
        keys.extend(
            read_lock(&self.async_field_validators, "reading async validator keys")?
                .keys()
                .copied(),
        );
        keys.extend(
            read_lock(&self.dependencies, "reading dependency keys")?
                .iter()
                .flat_map(|(key, values)| std::iter::once(*key).chain(values.iter().copied())),
        );
        keys.extend(
            read_lock(&self.focus_handlers, "reading focus handler keys")?
                .keys()
                .copied(),
        );
        keys.extend(
            read_lock(&self.required_fields, "reading required field keys")?
                .iter()
                .copied(),
        );
        keys.extend(
            read_lock(&self.field_descriptions, "reading description field keys")?
                .keys()
                .copied(),
        );
        keys.extend(
            read_lock(&self.state, "reading known keys from field metadata")?
                .field_meta
                .keys()
                .copied(),
        );
        Ok(keys)
    }

    fn finish_async_validation(
        &self,
        key: FieldKey,
        ticket: ValidationTicket,
        result: Result<(), E>,
    ) -> FormResult<()> {
        let mut state = write_lock(&self.state, "finishing async validation")?;
        if state.tickets.get(&key).copied() != Some(ticket) {
            return Ok(());
        }
        let meta = state.ensure_meta(key);
        meta.validating = false;
        meta.errors = match result {
            Ok(()) => Vec::new(),
            Err(error) => vec![error],
        };
        state.first_error = first_error_key(&state.field_meta);
        Ok(())
    }
}
