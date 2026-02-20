use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Duration;

use gpui::{SharedString, Window};

use super::validation::ValidationError;

static FORM_ID_ALLOCATOR: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FormId(pub u64);

impl FormId {
    pub fn next() -> Self {
        Self(FORM_ID_ALLOCATOR.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FieldKey(&'static str);

impl FieldKey {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl Display for FieldKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ValidationTicket(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubmitState {
    Idle,
    Validating,
    Submitting,
    Succeeded,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationMode {
    OnChange,
    OnBlur,
    OnSubmit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RevalidateMode {
    OnChange,
    OnBlur,
    OnSubmit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FormOptions {
    pub validate_mode: ValidationMode,
    pub revalidate_mode: RevalidateMode,
    pub validate_first_error_only: bool,
    pub focus_first_error_on_submit: bool,
}

impl Default for FormOptions {
    fn default() -> Self {
        Self {
            validate_mode: ValidationMode::OnSubmit,
            revalidate_mode: RevalidateMode::OnChange,
            validate_first_error_only: false,
            focus_first_error_on_submit: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldMeta<E> {
    pub dirty: bool,
    pub touched: bool,
    pub validating: bool,
    pub errors: Vec<E>,
}

impl<E> Default for FieldMeta<E> {
    fn default() -> Self {
        Self {
            dirty: false,
            touched: false,
            validating: false,
            errors: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FormSnapshot<T, E> {
    pub model: T,
    pub submit_state: SubmitState,
    pub submit_count: u32,
    pub is_dirty: bool,
    pub is_valid: bool,
    pub field_meta: BTreeMap<FieldKey, FieldMeta<E>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FormError {
    StatePoisoned(&'static str),
    InvalidStateTransition { from: SubmitState, to: SubmitState },
    AlreadySubmitting,
    DraftLoadFailed(String),
    DraftSaveFailed(String),
    DraftClearFailed(String),
}

impl Display for FormError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FormError::StatePoisoned(context) => {
                write!(f, "form state lock poisoned while {context}")
            }
            FormError::InvalidStateTransition { from, to } => {
                write!(f, "invalid submit state transition: {from:?} -> {to:?}")
            }
            FormError::AlreadySubmitting => f.write_str("form submit is already in progress"),
            FormError::DraftLoadFailed(error) => write!(f, "failed to load draft: {error}"),
            FormError::DraftSaveFailed(error) => write!(f, "failed to save draft: {error}"),
            FormError::DraftClearFailed(error) => write!(f, "failed to clear draft: {error}"),
        }
    }
}

impl std::error::Error for FormError {}

pub type FormResult<T> = Result<T, FormError>;

pub(super) type SyncFieldValidatorFn<T, E> = Arc<dyn Fn(&T) -> Result<(), E> + Send + Sync>;
pub(super) type SyncFormValidatorFn<T, E> = Arc<dyn Fn(&T) -> Vec<(FieldKey, E)> + Send + Sync>;
pub(super) type AsyncFieldValidatorFn<T, E> =
    Arc<dyn Fn(T) -> Pin<Box<dyn Future<Output = Result<(), E>> + Send + 'static>> + Send + Sync>;
pub(super) type FocusHandler = Arc<dyn Fn(&mut Window, &mut gpui::App) + Send + Sync>;

#[derive(Clone)]
pub(super) struct AsyncFieldValidatorEntry<T, E> {
    pub(super) debounce: Duration,
    pub(super) validator: AsyncFieldValidatorFn<T, E>,
}

pub(super) struct FormState<T, E> {
    pub(super) id: FormId,
    pub(super) initial_model: T,
    pub(super) model: T,
    pub(super) submit_state: SubmitState,
    pub(super) submit_count: u32,
    pub(super) dirty_fields: BTreeSet<FieldKey>,
    pub(super) field_meta: BTreeMap<FieldKey, FieldMeta<E>>,
    pub(super) tickets: BTreeMap<FieldKey, ValidationTicket>,
    pub(super) first_error: Option<FieldKey>,
}

impl<T, E> FormState<T, E> {
    pub(super) fn ensure_meta(&mut self, key: FieldKey) -> &mut FieldMeta<E> {
        self.field_meta.entry(key).or_default()
    }
}

#[derive(Clone)]
pub struct FormController<T, E>
where
    T: Clone + Send + Sync + 'static,
    E: ValidationError,
{
    pub(super) options: FormOptions,
    pub(super) state: Arc<RwLock<FormState<T, E>>>,
    pub(super) sync_field_validators:
        Arc<RwLock<BTreeMap<FieldKey, Vec<SyncFieldValidatorFn<T, E>>>>>,
    pub(super) async_field_validators:
        Arc<RwLock<BTreeMap<FieldKey, Vec<AsyncFieldValidatorEntry<T, E>>>>>,
    pub(super) form_validators: Arc<RwLock<Vec<SyncFormValidatorFn<T, E>>>>,
    pub(super) dependencies: Arc<RwLock<BTreeMap<FieldKey, BTreeSet<FieldKey>>>>,
    pub(super) focus_handlers: Arc<RwLock<BTreeMap<FieldKey, FocusHandler>>>,
    pub(super) required_fields: Arc<RwLock<BTreeSet<FieldKey>>>,
    pub(super) field_descriptions: Arc<RwLock<BTreeMap<FieldKey, SharedString>>>,
}

impl<T, E> FormController<T, E>
where
    T: Clone + Send + Sync + 'static,
    E: ValidationError,
{
    pub fn new(initial: T, options: FormOptions) -> Self {
        Self {
            options,
            state: Arc::new(RwLock::new(FormState {
                id: FormId::next(),
                initial_model: initial.clone(),
                model: initial,
                submit_state: SubmitState::Idle,
                submit_count: 0,
                dirty_fields: BTreeSet::new(),
                field_meta: BTreeMap::new(),
                tickets: BTreeMap::new(),
                first_error: None,
            })),
            sync_field_validators: Arc::new(RwLock::new(BTreeMap::new())),
            async_field_validators: Arc::new(RwLock::new(BTreeMap::new())),
            form_validators: Arc::new(RwLock::new(Vec::new())),
            dependencies: Arc::new(RwLock::new(BTreeMap::new())),
            focus_handlers: Arc::new(RwLock::new(BTreeMap::new())),
            required_fields: Arc::new(RwLock::new(BTreeSet::new())),
            field_descriptions: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub fn form_id(&self) -> FormResult<FormId> {
        Ok(read_lock(&self.state, "reading form id")?.id)
    }

    pub fn register_focus_handler<L>(
        &self,
        lens: L,
        handler: impl Fn(&mut Window, &mut gpui::App) + Send + Sync + 'static,
    ) -> FormResult<()>
    where
        L: super::validation::FieldLens<T>,
    {
        let mut handlers = write_lock(&self.focus_handlers, "registering focus handler")?;
        handlers.insert(lens.key(), Arc::new(handler));
        Ok(())
    }

    pub fn register_required_field<L>(&self, lens: L) -> FormResult<()>
    where
        L: super::validation::FieldLens<T>,
    {
        let mut required = write_lock(&self.required_fields, "registering required field")?;
        required.insert(lens.key());
        Ok(())
    }

    pub fn unregister_required_field<L>(&self, lens: L) -> FormResult<()>
    where
        L: super::validation::FieldLens<T>,
    {
        let mut required = write_lock(&self.required_fields, "unregistering required field")?;
        required.remove(&lens.key());
        Ok(())
    }

    pub fn register_field_description<L>(
        &self,
        lens: L,
        description: impl Into<SharedString>,
    ) -> FormResult<()>
    where
        L: super::validation::FieldLens<T>,
    {
        let mut descriptions =
            write_lock(&self.field_descriptions, "registering field description")?;
        descriptions.insert(lens.key(), description.into());
        Ok(())
    }

    pub fn clear_field_description<L>(&self, lens: L) -> FormResult<()>
    where
        L: super::validation::FieldLens<T>,
    {
        let mut descriptions = write_lock(&self.field_descriptions, "clearing field description")?;
        descriptions.remove(&lens.key());
        Ok(())
    }

    pub fn submit(&self, f: impl FnOnce(&T) -> FormResult<()> + 'static) -> FormResult<()> {
        {
            let mut state = write_lock(&self.state, "preparing submit")?;
            if state.submit_state == SubmitState::Submitting {
                return Err(FormError::AlreadySubmitting);
            }
            transition_submit_state(&mut state, SubmitState::Validating)?;
            state.submit_count = state.submit_count.saturating_add(1);
        }

        let is_valid = self.validate_form()?;
        if !is_valid {
            let mut state = write_lock(&self.state, "handling submit validation failure")?;
            transition_submit_state(&mut state, SubmitState::Failed)?;
            return Ok(());
        }

        let model = {
            let mut state = write_lock(&self.state, "moving submit state to submitting")?;
            transition_submit_state(&mut state, SubmitState::Submitting)?;
            state.model.clone()
        };
        let submit_result = f(&model);

        let mut state = write_lock(&self.state, "completing submit")?;
        if submit_result.is_ok() {
            transition_submit_state(&mut state, SubmitState::Succeeded)?;
        } else {
            transition_submit_state(&mut state, SubmitState::Failed)?;
        }
        submit_result
    }

    pub async fn submit_async<F, Fut>(&self, f: F) -> FormResult<()>
    where
        F: FnOnce(&T) -> Fut + 'static,
        Fut: Future<Output = FormResult<()>> + Send + 'static,
    {
        {
            let mut state = write_lock(&self.state, "preparing async submit")?;
            if state.submit_state == SubmitState::Submitting {
                return Err(FormError::AlreadySubmitting);
            }
            transition_submit_state(&mut state, SubmitState::Validating)?;
            state.submit_count = state.submit_count.saturating_add(1);
        }

        let is_valid = self.validate_form_async().await?;
        if !is_valid {
            let mut state = write_lock(&self.state, "handling async submit validation failure")?;
            transition_submit_state(&mut state, SubmitState::Failed)?;
            return Ok(());
        }

        let model = {
            let mut state = write_lock(&self.state, "moving async submit state to submitting")?;
            transition_submit_state(&mut state, SubmitState::Submitting)?;
            state.model.clone()
        };
        let submit_result = f(&model).await;

        let mut state = write_lock(&self.state, "completing async submit")?;
        if submit_result.is_ok() {
            transition_submit_state(&mut state, SubmitState::Succeeded)?;
        } else {
            transition_submit_state(&mut state, SubmitState::Failed)?;
        }
        submit_result
    }

    pub fn submit_in(
        &self,
        window: &mut Window,
        cx: &mut gpui::App,
        f: impl FnOnce(&T) -> FormResult<()> + 'static,
    ) -> FormResult<()> {
        let result = self.submit(f);
        if self.options.focus_first_error_on_submit {
            let _ = self.focus_first_error(window, cx)?;
        }
        result
    }

    pub async fn submit_async_in<F, Fut>(
        &self,
        window: &mut Window,
        cx: &mut gpui::App,
        f: F,
    ) -> FormResult<()>
    where
        F: FnOnce(&T) -> Fut + 'static,
        Fut: Future<Output = FormResult<()>> + Send + 'static,
    {
        let result = self.submit_async(f).await;
        if self.options.focus_first_error_on_submit {
            let _ = self.focus_first_error(window, cx)?;
        }
        result
    }

    pub fn focus_first_error(&self, window: &mut Window, cx: &mut gpui::App) -> FormResult<bool> {
        let first_error = read_lock(&self.state, "reading first error key")?.first_error;
        let Some(key) = first_error else {
            return Ok(false);
        };
        let handler = read_lock(&self.focus_handlers, "reading focus handlers")?
            .get(&key)
            .cloned();
        if let Some(handler) = handler {
            handler(window, cx);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn reset_to_initial(&self) -> FormResult<()> {
        let mut state = write_lock(&self.state, "resetting form")?;
        state.model = state.initial_model.clone();
        state.submit_state = SubmitState::Idle;
        state.dirty_fields.clear();
        state.tickets.clear();
        state.first_error = None;
        for meta in state.field_meta.values_mut() {
            meta.dirty = false;
            meta.touched = false;
            meta.validating = false;
            meta.errors.clear();
        }
        Ok(())
    }

    pub fn reset_field<L>(&self, lens: L) -> FormResult<()>
    where
        L: super::validation::FieldLens<T>,
    {
        let key = lens.key();
        let mut state = write_lock(&self.state, "resetting field")?;
        let initial_value = lens.get(&state.initial_model).clone();
        lens.set(&mut state.model, initial_value);
        state.dirty_fields.remove(&key);
        let meta = state.ensure_meta(key);
        meta.dirty = false;
        meta.touched = false;
        meta.validating = false;
        meta.errors.clear();
        state.first_error = first_error_key(&state.field_meta);
        Ok(())
    }

    pub fn clear_errors(&self) -> FormResult<()> {
        let mut state = write_lock(&self.state, "clearing all field errors")?;
        for meta in state.field_meta.values_mut() {
            meta.errors.clear();
            meta.validating = false;
        }
        state.first_error = None;
        Ok(())
    }

    pub fn clear_field_errors<L>(&self, lens: L) -> FormResult<()>
    where
        L: super::validation::FieldLens<T>,
    {
        let key = lens.key();
        let mut state = write_lock(&self.state, "clearing field errors")?;
        if let Some(meta) = state.field_meta.get_mut(&key) {
            meta.errors.clear();
            meta.validating = false;
        }
        state.first_error = first_error_key(&state.field_meta);
        Ok(())
    }

    pub fn snapshot(&self) -> FormResult<FormSnapshot<T, E>> {
        let state = read_lock(&self.state, "creating form snapshot")?;
        let is_valid = state.field_meta.values().all(|meta| meta.errors.is_empty());
        Ok(FormSnapshot {
            model: state.model.clone(),
            submit_state: state.submit_state,
            submit_count: state.submit_count,
            is_dirty: !state.dirty_fields.is_empty(),
            is_valid,
            field_meta: state.field_meta.clone(),
        })
    }

    pub fn field_meta<L>(&self, lens: L) -> FormResult<Option<FieldMeta<E>>>
    where
        L: super::validation::FieldLens<T>,
    {
        Ok(read_lock(&self.state, "reading field meta")?
            .field_meta
            .get(&lens.key())
            .cloned())
    }

    pub fn field_description<L>(&self, lens: L) -> FormResult<Option<SharedString>>
    where
        L: super::validation::FieldLens<T>,
    {
        Ok(
            read_lock(&self.field_descriptions, "reading field description")?
                .get(&lens.key())
                .cloned(),
        )
    }

    pub fn is_required<L>(&self, lens: L) -> FormResult<bool>
    where
        L: super::validation::FieldLens<T>,
    {
        Ok(read_lock(&self.required_fields, "reading required fields")?.contains(&lens.key()))
    }
}

pub(super) fn transition_submit_state<T, E>(
    state: &mut FormState<T, E>,
    next: SubmitState,
) -> FormResult<()> {
    let current = state.submit_state;
    if current == next {
        return Ok(());
    }

    let allowed = matches!(
        (current, next),
        (SubmitState::Idle, SubmitState::Validating)
            | (SubmitState::Validating, SubmitState::Submitting)
            | (SubmitState::Validating, SubmitState::Failed)
            | (SubmitState::Submitting, SubmitState::Succeeded)
            | (SubmitState::Submitting, SubmitState::Failed)
            | (SubmitState::Succeeded, SubmitState::Validating)
            | (SubmitState::Failed, SubmitState::Validating)
            | (_, SubmitState::Idle)
    );
    if !allowed {
        return Err(FormError::InvalidStateTransition {
            from: current,
            to: next,
        });
    }
    state.submit_state = next;
    Ok(())
}

pub(super) fn first_error_key<E>(
    field_meta: &BTreeMap<FieldKey, FieldMeta<E>>,
) -> Option<FieldKey> {
    field_meta
        .iter()
        .find_map(|(key, meta)| (!meta.errors.is_empty()).then_some(*key))
}

pub(super) fn read_lock<'a, T>(
    lock: &'a RwLock<T>,
    context: &'static str,
) -> FormResult<RwLockReadGuard<'a, T>> {
    lock.read().map_err(|_| FormError::StatePoisoned(context))
}

pub(super) fn write_lock<'a, T>(
    lock: &'a RwLock<T>,
    context: &'static str,
) -> FormResult<RwLockWriteGuard<'a, T>> {
    lock.write().map_err(|_| FormError::StatePoisoned(context))
}
