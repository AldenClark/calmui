# CalmUI Form v1（Rust 化）使用指南

## 目标

- 强类型模型：`FormController<TModel, TError>`
- 强类型字段访问：`FieldLens`
- 同步 / 异步校验、依赖联动、提交状态机
- 桌面端能力：首错聚焦、键盘提交绑定、草稿存储

## 最小示例

```rust
use calmui::prelude::*;
use rust_decimal::Decimal;

#[derive(Clone, FormModel)]
struct SettingsForm {
    email: SharedString,
    password: SharedString,
    confirm_password: SharedString,
    notifications_enabled: bool,
    budget: Decimal,
    tags: Vec<SharedString>,
}

#[derive(Clone)]
struct ValidationMsg(&'static str);

impl ValidationError for ValidationMsg {
    fn message(&self) -> SharedString {
        self.0.into()
    }
}

let form = FormController::<SettingsForm, ValidationMsg>::new(
    SettingsForm {
        email: "".into(),
        password: "".into(),
        confirm_password: "".into(),
        notifications_enabled: false,
        budget: Decimal::from_i128_with_scale(0, 0),
        tags: vec![],
    },
    FormOptions::default(),
);

let fields = SettingsForm::fields();

form.register_required_field(fields.email())?;
form.register_field_description(fields.email(), "请输入业务通知邮箱")?;

form.register_field_validator(fields.email(), |_model, value: &SharedString| {
    if value.is_empty() {
        Err(ValidationMsg("邮箱不能为空"))
    } else {
        Ok(())
    }
})?;

form.register_field_validator(fields.confirm_password(), |model: &SettingsForm, value: &SharedString| {
    if value != &model.password {
        Err(ValidationMsg("两次密码不一致"))
    } else {
        Ok(())
    }
})?;
form.register_dependency(fields.password(), fields.confirm_password())?;

form.register_async_field_validator_with_debounce(
    fields.email(),
    300,
    |_: &SettingsForm, value: &SharedString| {
        let value = value.clone();
        Box::pin(async move {
            if value.as_ref().ends_with("@example.com") {
                Err(ValidationMsg("该域名不允许"))
            } else {
                Ok(())
            }
        })
    },
)?;
```

## 绑定现有组件

```rust
let email_input = form.bind_text_input(fields.email(), TextInput::new())?;
let password_input = form.bind_password_input_submit(
    fields.password(),
    PasswordInput::new(),
    |form, window, cx| {
        let _ = form.submit_in(window, cx, |_model| Ok(()));
    },
)?;
let budget_input = form.bind_number_input(fields.budget(), NumberInput::new())?;
let switch_input = form.bind_switch(fields.notifications_enabled(), Switch::new())?;
let tags_input = form.bind_multiselect(fields.tags(), MultiSelect::new())?;
```

## 草稿能力

```rust
let store = InMemoryDraftStore::<SettingsForm>::new();
form.save_draft(&store)?;
let loaded = form.load_draft(&store)?;
if loaded {
    // 已恢复草稿
}
form.clear_draft(&store)?;
```

## 设计说明

- 错误显示规则：字段 `touched == true` 或 `submit_count > 0` 才对外显示。
- `set_async` / `touch_async` 会触发已注册异步校验。
- 防抖策略：每个字段 validator 可独立配置 `debounce_ms`。
- 竞态策略：`ValidationTicket` 保证旧请求返回不会覆盖新结果。
