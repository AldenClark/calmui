use std::collections::BTreeMap;
use std::convert::Infallible;
use std::sync::{Arc, RwLock};

use super::controller::{FormController, FormError, FormId, FormResult, read_lock, write_lock};
use super::validation::ValidationError;

pub trait FormDraftStore<T>: Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;

    fn save(&self, form_id: FormId, model: &T) -> Result<(), Self::Error>;
    fn load(&self, form_id: FormId) -> Result<Option<T>, Self::Error>;
    fn clear(&self, form_id: FormId) -> Result<(), Self::Error>;
}

#[derive(Clone)]
pub struct InMemoryDraftStore<T> {
    state: Arc<RwLock<BTreeMap<FormId, T>>>,
}

impl<T> InMemoryDraftStore<T> {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}

impl<T> Default for InMemoryDraftStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FormDraftStore<T> for InMemoryDraftStore<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = Infallible;

    fn save(&self, form_id: FormId, model: &T) -> Result<(), Self::Error> {
        let mut state = match self.state.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        state.insert(form_id, model.clone());
        Ok(())
    }

    fn load(&self, form_id: FormId) -> Result<Option<T>, Self::Error> {
        let state = match self.state.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let value = state.get(&form_id).cloned();
        Ok(value)
    }

    fn clear(&self, form_id: FormId) -> Result<(), Self::Error> {
        let mut state = match self.state.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        state.remove(&form_id);
        Ok(())
    }
}

impl<T, E> FormController<T, E>
where
    T: Clone + Send + Sync + 'static,
    E: ValidationError,
{
    pub fn save_draft<S>(&self, store: &S) -> FormResult<()>
    where
        S: FormDraftStore<T>,
    {
        let state = read_lock(&self.state, "saving draft")?;
        store
            .save(state.id, &state.model)
            .map_err(|error| FormError::DraftSaveFailed(error.to_string()))
    }

    pub fn load_draft<S>(&self, store: &S) -> FormResult<bool>
    where
        S: FormDraftStore<T>,
    {
        let form_id = self.form_id()?;
        let Some(draft) = store
            .load(form_id)
            .map_err(|error| FormError::DraftLoadFailed(error.to_string()))?
        else {
            return Ok(false);
        };

        let known_keys = self.known_field_keys()?;
        let mut state = write_lock(&self.state, "loading draft into form")?;
        state.model = draft;
        state.submit_state = super::controller::SubmitState::Idle;
        state.submit_count = 0;
        state.tickets.clear();
        state.first_error = None;
        state.dirty_fields = known_keys;
        for key in state.dirty_fields.clone() {
            let meta = state.ensure_meta(key);
            meta.dirty = true;
            meta.validating = false;
            meta.errors.clear();
        }
        Ok(true)
    }

    pub fn clear_draft<S>(&self, store: &S) -> FormResult<()>
    where
        S: FormDraftStore<T>,
    {
        let form_id = self.form_id()?;
        store
            .clear(form_id)
            .map_err(|error| FormError::DraftClearFailed(error.to_string()))
    }
}
