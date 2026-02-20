use calmui::form::{FieldLens, FormModel};

#[derive(Clone, calmui::form::FormModel)]
struct DemoForm {
    email: String,
}

fn main() {
    let fields = DemoForm::fields();
    let lens = fields.email();
    let mut model = DemoForm {
        email: "a@calm.ui".to_string(),
    };
    lens.set(&mut model, "b@calm.ui".to_string());
    assert_eq!(lens.key().as_str(), "email");
    assert_eq!(lens.get(&model), "b@calm.ui");
}
