mod binding;
mod controller;
mod draft;
mod validation;

#[cfg(test)]
mod tests;

pub use calmui_form_derive::FormModel;
pub use controller::{
    FieldKey, FieldMeta, FormController, FormError, FormId, FormOptions, FormResult, FormSnapshot,
    RevalidateMode, SubmitState, ValidationMode, ValidationTicket,
};
pub use draft::{FormDraftStore, InMemoryDraftStore};
pub use validation::{
    AsyncFieldValidator, BoxedValidationFuture, FieldLens, FieldValidator, FormModel,
    FormValidator, ValidationError,
};

#[doc(hidden)]
pub mod compat {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum CompatibilityStatus {
        Experimental,
    }
}
