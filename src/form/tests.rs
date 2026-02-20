use super::*;
use futures::executor::block_on;
use gpui::SharedString;
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use crate::components::TextInput;

#[derive(Clone, Debug, Eq, PartialEq)]
struct TestError(&'static str);

impl ValidationError for TestError {
    fn message(&self) -> SharedString {
        self.0.into()
    }
}

#[allow(dead_code)]
#[derive(Clone, calmui_form_derive::FormModel)]
struct ProfileForm {
    email: SharedString,
    password: SharedString,
    confirm_password: SharedString,
    enabled: bool,
    amount: Decimal,
    tags: Vec<SharedString>,
}

fn base_form() -> ProfileForm {
    ProfileForm {
        email: "user@example.com".into(),
        password: "pass".into(),
        confirm_password: "pass".into(),
        enabled: false,
        amount: Decimal::from_i128_with_scale(1200, 2),
        tags: vec!["a".into()],
    }
}

#[derive(Clone)]
struct PerfForm {
    values: BTreeMap<&'static str, SharedString>,
}

impl FormModel for PerfForm {
    type Fields = ();

    fn fields() -> Self::Fields {}
}

#[derive(Clone, Copy)]
struct MapLens {
    key: &'static str,
}

impl FieldLens<PerfForm> for MapLens {
    type Value = SharedString;

    fn key(self) -> FieldKey {
        FieldKey::new(self.key)
    }

    fn get<'a>(self, model: &'a PerfForm) -> &'a Self::Value {
        model
            .values
            .get(self.key)
            .expect("perf key must exist in model values")
    }

    fn set(self, model: &mut PerfForm, value: Self::Value) {
        model.values.insert(self.key, value);
    }
}

struct TimedValidator {
    delay_ms: u64,
    fail: bool,
}

impl AsyncFieldValidator<ProfileForm, ProfileFormEmailLens, TestError> for TimedValidator {
    type Fut<'a> = BoxedValidationFuture<'a, TestError>;

    fn validate<'a>(&'a self, _model: &'a ProfileForm, _value: &'a SharedString) -> Self::Fut<'a> {
        Box::pin(async move {
            thread::sleep(Duration::from_millis(self.delay_ms));
            if self.fail {
                Err(TestError("async error"))
            } else {
                Ok(())
            }
        })
    }
}

struct ContainsValidator {
    needle: &'static str,
}

impl AsyncFieldValidator<ProfileForm, ProfileFormEmailLens, TestError> for ContainsValidator {
    type Fut<'a> = BoxedValidationFuture<'a, TestError>;

    fn validate<'a>(&'a self, _model: &'a ProfileForm, value: &'a SharedString) -> Self::Fut<'a> {
        let value = value.clone();
        let needle = self.needle;
        Box::pin(async move {
            if value.as_ref().contains(needle) {
                Err(TestError("email invalid"))
            } else {
                Ok(())
            }
        })
    }
}

struct RequiredValidator;

impl AsyncFieldValidator<ProfileForm, ProfileFormEmailLens, TestError> for RequiredValidator {
    type Fut<'a> = BoxedValidationFuture<'a, TestError>;

    fn validate<'a>(&'a self, _model: &'a ProfileForm, value: &'a SharedString) -> Self::Fut<'a> {
        let value = value.clone();
        Box::pin(async move {
            if value.is_empty() {
                Err(TestError("required"))
            } else {
                Ok(())
            }
        })
    }
}

#[test]
fn field_lens_updates_model_and_dirty_state() {
    let controller =
        FormController::<ProfileForm, TestError>::new(base_form(), FormOptions::default());
    let fields = ProfileForm::fields();

    controller
        .set(fields.email(), "changed@example.com".into())
        .expect("set must succeed");
    let snapshot = controller.snapshot().expect("snapshot must succeed");
    assert!(snapshot.is_dirty);
    assert_eq!(snapshot.model.email, "changed@example.com");

    let email_meta = snapshot
        .field_meta
        .get(&fields.email().key())
        .expect("email meta should exist");
    assert!(email_meta.dirty);
}

#[test]
fn validation_mode_controls_when_errors_appear() {
    let fields = ProfileForm::fields();
    let on_change = FormController::<ProfileForm, TestError>::new(
        base_form(),
        FormOptions {
            validate_mode: ValidationMode::OnChange,
            ..FormOptions::default()
        },
    );
    on_change
        .register_field_validator(
            fields.email(),
            |_model: &ProfileForm, value: &SharedString| {
                if value.is_empty() {
                    Err(TestError("required"))
                } else {
                    Ok(())
                }
            },
        )
        .expect("register validator");
    on_change
        .set(fields.email(), "".into())
        .expect("set should trigger validation");
    assert_eq!(
        on_change
            .snapshot()
            .expect("snapshot")
            .field_meta
            .get(&fields.email().key())
            .expect("field meta")
            .errors
            .len(),
        1
    );

    let on_submit = FormController::<ProfileForm, TestError>::new(
        base_form(),
        FormOptions {
            validate_mode: ValidationMode::OnSubmit,
            ..FormOptions::default()
        },
    );
    on_submit
        .register_field_validator(
            fields.email(),
            |_model: &ProfileForm, value: &SharedString| {
                if value.is_empty() {
                    Err(TestError("required"))
                } else {
                    Ok(())
                }
            },
        )
        .expect("register validator");
    on_submit
        .set(fields.email(), "".into())
        .expect("set should not trigger validation immediately");
    assert!(
        on_submit
            .snapshot()
            .expect("snapshot")
            .field_meta
            .get(&fields.email().key())
            .is_some_and(|meta| meta.errors.is_empty())
    );
    assert!(!on_submit.validate_form().expect("validate form"));
}

#[test]
fn dependencies_revalidate_linked_fields() {
    let fields = ProfileForm::fields();
    let controller = FormController::<ProfileForm, TestError>::new(
        base_form(),
        FormOptions {
            validate_mode: ValidationMode::OnChange,
            revalidate_mode: RevalidateMode::OnChange,
            ..FormOptions::default()
        },
    );
    controller
        .register_field_validator(
            fields.confirm_password(),
            |model: &ProfileForm, value: &SharedString| {
                if value != &model.password {
                    Err(TestError("password mismatch"))
                } else {
                    Ok(())
                }
            },
        )
        .expect("register validator");
    controller
        .register_dependency(fields.password(), fields.confirm_password())
        .expect("register dependency");

    controller
        .set(fields.password(), "new-pass".into())
        .expect("set source field");
    let confirm_errors = controller
        .snapshot()
        .expect("snapshot")
        .field_meta
        .get(&fields.confirm_password().key())
        .expect("confirm field meta")
        .errors
        .clone();
    assert_eq!(confirm_errors, vec![TestError("password mismatch")]);
}

#[test]
fn async_validation_ticket_keeps_latest_result() {
    let fields = ProfileForm::fields();
    let controller =
        FormController::<ProfileForm, TestError>::new(base_form(), FormOptions::default());
    let slow_controller = controller.clone();
    let fast_controller = controller.clone();
    let lens = fields.email();

    let slow = thread::spawn(move || {
        let validator = TimedValidator {
            delay_ms: 70,
            fail: true,
        };
        block_on(slow_controller.validate_field_async(lens, &validator)).expect("slow async");
    });
    thread::sleep(Duration::from_millis(10));
    let fast = thread::spawn(move || {
        let validator = TimedValidator {
            delay_ms: 5,
            fail: false,
        };
        block_on(fast_controller.validate_field_async(lens, &validator)).expect("fast async");
    });

    slow.join().expect("slow thread joins");
    fast.join().expect("fast thread joins");

    let snapshot = controller.snapshot().expect("snapshot");
    let email_meta = snapshot
        .field_meta
        .get(&fields.email().key())
        .expect("email meta");
    assert!(email_meta.errors.is_empty());
}

#[test]
fn submit_state_transitions_are_enforced() {
    let fields = ProfileForm::fields();
    let controller =
        FormController::<ProfileForm, TestError>::new(base_form(), FormOptions::default());
    controller
        .register_field_validator(
            fields.email(),
            |_model: &ProfileForm, value: &SharedString| {
                if value.is_empty() {
                    Err(TestError("required"))
                } else {
                    Ok(())
                }
            },
        )
        .expect("register validator");

    let submit_count = Arc::new(AtomicUsize::new(0));

    controller
        .set(fields.email(), "".into())
        .expect("set invalid email");
    {
        let submit_count = submit_count.clone();
        controller
            .submit(move |_model| {
                submit_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .expect("submit should return Ok when validation fails");
    }
    assert_eq!(submit_count.load(Ordering::SeqCst), 0);
    assert_eq!(
        controller.snapshot().expect("snapshot").submit_state,
        SubmitState::Failed
    );

    controller
        .set(fields.email(), "valid@example.com".into())
        .expect("set valid email");
    {
        let submit_count = submit_count.clone();
        controller
            .submit(move |_model| {
                submit_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .expect("submit should succeed");
    }
    assert_eq!(submit_count.load(Ordering::SeqCst), 1);
    assert_eq!(
        controller.snapshot().expect("snapshot").submit_state,
        SubmitState::Succeeded
    );
}

#[test]
fn async_registered_validator_is_debounced_with_latest_ticket_wins() {
    let fields = ProfileForm::fields();
    let controller = FormController::<ProfileForm, TestError>::new(
        base_form(),
        FormOptions {
            validate_mode: ValidationMode::OnChange,
            ..FormOptions::default()
        },
    );
    controller
        .register_async_field_validator_with_debounce(
            fields.email(),
            30,
            ContainsValidator { needle: "bad" },
        )
        .expect("register async validator");

    let first = {
        let controller = controller.clone();
        let lens = fields.email();
        thread::spawn(move || {
            block_on(controller.set_async(lens, "bad@example.com".into())).expect("first set");
        })
    };
    thread::sleep(Duration::from_millis(5));
    let second = {
        let controller = controller.clone();
        let lens = fields.email();
        thread::spawn(move || {
            block_on(controller.set_async(lens, "good@example.com".into())).expect("second set");
        })
    };

    first.join().expect("first thread joins");
    second.join().expect("second thread joins");

    let snapshot = controller.snapshot().expect("snapshot");
    let meta = snapshot
        .field_meta
        .get(&fields.email().key())
        .expect("email meta");
    assert!(meta.errors.is_empty());
    assert_eq!(snapshot.model.email, "good@example.com");
}

#[test]
fn validate_form_async_runs_registered_async_validators() {
    let fields = ProfileForm::fields();
    let controller =
        FormController::<ProfileForm, TestError>::new(base_form(), FormOptions::default());
    controller
        .register_async_field_validator(fields.email(), RequiredValidator)
        .expect("register async validator");
    controller
        .set(fields.email(), "".into())
        .expect("set invalid value");

    let valid = block_on(controller.validate_form_async()).expect("validate async");
    assert!(!valid);
    let snapshot = controller.snapshot().expect("snapshot");
    assert_eq!(
        snapshot
            .field_meta
            .get(&fields.email().key())
            .expect("email meta")
            .errors,
        vec![TestError("required")]
    );
}

#[test]
fn draft_store_roundtrip_loads_and_clears() {
    let fields = ProfileForm::fields();
    let store = InMemoryDraftStore::new();
    let controller =
        FormController::<ProfileForm, TestError>::new(base_form(), FormOptions::default());

    controller
        .set(fields.email(), "draft@calm.ui".into())
        .expect("set email");
    controller.save_draft(&store).expect("save draft");

    controller.reset_to_initial().expect("reset form");
    assert_eq!(
        controller.snapshot().expect("snapshot").model.email,
        "user@example.com"
    );

    let loaded = controller.load_draft(&store).expect("load draft");
    assert!(loaded);
    let snapshot = controller.snapshot().expect("snapshot");
    assert_eq!(snapshot.model.email, "draft@calm.ui");
    assert!(snapshot.is_dirty);

    controller.clear_draft(&store).expect("clear draft");
    let loaded_again = controller.load_draft(&store).expect("load after clear");
    assert!(!loaded_again);
}

#[test]
fn reset_field_and_clear_errors_are_consistent() {
    let fields = ProfileForm::fields();
    let controller = FormController::<ProfileForm, TestError>::new(
        base_form(),
        FormOptions {
            validate_mode: ValidationMode::OnChange,
            ..FormOptions::default()
        },
    );

    controller
        .register_field_validator(
            fields.email(),
            |_model: &ProfileForm, value: &SharedString| {
                if value.is_empty() {
                    Err(TestError("required"))
                } else {
                    Ok(())
                }
            },
        )
        .expect("register validator");
    controller
        .set(fields.email(), "".into())
        .expect("set invalid value");
    controller
        .clear_field_errors(fields.email())
        .expect("clear field errors");
    assert!(
        controller
            .field_meta(fields.email())
            .expect("meta")
            .expect("meta exists")
            .errors
            .is_empty()
    );

    controller
        .set(fields.email(), "dirty@example.com".into())
        .expect("set dirty value");
    controller.reset_field(fields.email()).expect("reset field");
    let snapshot = controller.snapshot().expect("snapshot");
    assert_eq!(snapshot.model.email, "user@example.com");
    assert!(
        snapshot
            .field_meta
            .get(&fields.email().key())
            .is_some_and(|meta| !meta.dirty)
    );
}

#[test]
fn text_input_submit_binding_compiles() {
    let fields = ProfileForm::fields();
    let controller =
        FormController::<ProfileForm, TestError>::new(base_form(), FormOptions::default());
    let _ = controller
        .bind_text_input_submit(fields.email(), TextInput::new(), |form, window, cx| {
            let _ = form.submit_in(window, cx, |_model| Ok(()));
        })
        .expect("bind submit helper");
}

#[test]
fn single_field_update_keeps_other_field_meta_stable() {
    let fields = ProfileForm::fields();
    let controller =
        FormController::<ProfileForm, TestError>::new(base_form(), FormOptions::default());

    controller
        .set(fields.password(), "pass".into())
        .expect("seed password meta");
    controller
        .set(fields.email(), "only-email-changed@calm.ui".into())
        .expect("update email only");

    let snapshot = controller.snapshot().expect("snapshot");
    assert!(
        snapshot
            .field_meta
            .get(&fields.email().key())
            .is_some_and(|meta| meta.dirty)
    );
    assert!(
        snapshot
            .field_meta
            .get(&fields.password().key())
            .is_some_and(|meta| !meta.dirty)
    );
}

#[test]
fn error_visibility_requires_touch_or_submit() {
    let fields = ProfileForm::fields();
    let controller = FormController::<ProfileForm, TestError>::new(
        base_form(),
        FormOptions {
            validate_mode: ValidationMode::OnChange,
            ..FormOptions::default()
        },
    );
    controller
        .register_field_validator(
            fields.email(),
            |_model: &ProfileForm, value: &SharedString| {
                if value.is_empty() {
                    Err(TestError("required"))
                } else {
                    Ok(())
                }
            },
        )
        .expect("register validator");

    controller
        .set(fields.email(), "".into())
        .expect("set invalid");
    assert_eq!(
        controller
            .field_error_for_display(fields.email())
            .expect("display error"),
        None
    );

    controller.touch(fields.email()).expect("touch field");
    assert_eq!(
        controller
            .field_error_for_display(fields.email())
            .expect("display error"),
        Some(SharedString::from("required"))
    );
}

#[test]
fn required_and_description_registry_roundtrip() {
    let fields = ProfileForm::fields();
    let controller =
        FormController::<ProfileForm, TestError>::new(base_form(), FormOptions::default());

    controller
        .register_required_field(fields.email())
        .expect("register required");
    controller
        .register_field_description(fields.email(), "Enter a valid email")
        .expect("register description");

    assert!(controller.is_required(fields.email()).expect("is required"));
    assert_eq!(
        controller
            .field_description(fields.email())
            .expect("field description"),
        Some(SharedString::from("Enter a valid email"))
    );
}

#[test]
fn two_hundred_fields_update_invokes_single_validator_path() {
    let keys = (0..200)
        .map(|index| Box::leak(format!("field-{index}").into_boxed_str()) as &'static str)
        .collect::<Vec<_>>();

    let model = PerfForm {
        values: keys
            .iter()
            .map(|key| (*key, SharedString::from("")))
            .collect(),
    };

    let invoke_count = Arc::new(AtomicUsize::new(0));
    let controller = FormController::<PerfForm, TestError>::new(
        model,
        FormOptions {
            validate_mode: ValidationMode::OnChange,
            ..FormOptions::default()
        },
    );

    for key in &keys {
        let counter = invoke_count.clone();
        controller
            .register_field_validator(
                MapLens { key: *key },
                move |_model: &PerfForm, _value: &SharedString| {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                },
            )
            .expect("register validator");
    }

    let target = keys[137];
    controller
        .set(MapLens { key: target }, "changed".into())
        .expect("update single field");

    let snapshot = controller.snapshot().expect("snapshot");
    assert_eq!(invoke_count.load(Ordering::SeqCst), 1);
    assert_eq!(snapshot.field_meta.len(), 1);
    assert_eq!(
        snapshot
            .field_meta
            .get(&FieldKey::new(target))
            .expect("target meta")
            .errors
            .len(),
        0
    );
}

#[test]
fn derive_macro_generates_field_lenses() {
    let fields = ProfileForm::fields();
    assert_eq!(fields.email().key().as_str(), "email");
    assert_eq!(fields.confirm_password().key().as_str(), "confirm_password");
}
