use super::*;
use crate::contracts::ComponentThemeOverridable;
use gpui::{IntoElement, ParentElement, div};

fn into_any(element: impl IntoElement) -> gpui::AnyElement {
    element.into_any_element()
}

#[test]
fn markdown_renders_rich_gfm_sample_without_panic() {
    let sample = r#"
# Title

A paragraph with **strong**, *emphasis*, ~~deleted~~, `inline code`, <mark>mark</mark>, <kbd>Cmd+K</kbd>, and a [link](https://example.com).

> Quote line 1
> Quote line 2

- [ ] task item
- [x] done item
- nested
  - child A
  - child B

1. ordered one
2. ordered two

| Name | Value |
| :--- | ---: |
| alpha | 1 |
| beta | 2 |

---

```rust
fn main() {
    println!("hello");
}
```

![alt text](https://example.com/image.png "caption")
"#;

    let _ = into_any(
        Markdown::new(sample)
            .open_links_with_system(false)
            .on_link_click(|_payload, _window, _cx| {}),
    );
}

#[test]
fn markdown_theme_override_compiles_with_extended_tokens() {
    let _ = into_any(Markdown::new("demo").theme(|theme| {
        theme.markdown(|markdown| {
            markdown
                .paragraph_size(gpui::px(16.0))
                .paragraph_line_height(gpui::px(28.0))
                .link(gpui::black())
                .inline_code_radius(gpui::px(6.0))
                .table_radius(gpui::px(10.0))
                .image_radius(gpui::px(10.0))
                .heading2_padding_top(gpui::px(10.0))
        })
    }));
}

#[test]
fn markdown_compact_and_id_are_still_available() {
    let _ = into_any(
        Markdown::new("text")
            .compact(true)
            .with_id("markdown-smoke"),
    );
    let _ = into_any(div().child("ok"));
}
