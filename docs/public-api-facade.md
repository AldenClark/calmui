# Public API Facade (v3)

This document defines the new public-facing module layout for `calmui` as a reusable GitHub library.

## Goals

- keep backward compatibility with existing `calmui::components::*` call sites
- provide clearer grouped imports for new consumers
- prevent accidental leaking of internal module organization to users

## New Entry Points

1. `calmui::widgets`
- grouped re-exports by domain:
  - `widgets::form`
  - `widgets::layout`
  - `widgets::overlay`
  - `widgets::navigation`
  - `widgets::data`
  - `widgets::display`
  - `widgets::feedback`
- `widgets::*` also re-exports all grouped items at the top level for convenience.

2. `calmui::foundation`
- grouped foundational surface:
  - `foundation::contracts`
  - `foundation::style`
  - `foundation::theme`
  - `foundation::tokens`
  - `foundation::motion`
  - `foundation::icon`
  - `foundation::id`
  - `foundation::overlay`
  - `foundation::feedback`
  - `foundation::i18n` (feature-gated)

3. `calmui::prelude`
- single-import path for common app usage:
  - common component exports
  - style enums (`Size`, `Variant`, `Radius`, `FieldLayout`)
  - contracts (`Disableable`, `FieldLike`, etc.)
  - provider (`CalmProvider`)

## Compatibility Policy

- Existing `calmui::components::*` imports continue to work.
- Existing `calmui::{CalmProvider, I18nManager, Locale}` root exports continue to work.
- This pass is additive; no breaking rename/remove was introduced in public component symbols.

## Recommended Import Style (new projects)

```rust
use calmui::prelude::*;
use calmui::widgets::{form::*, layout::*, overlay::*};
use calmui::foundation::style::{Size, Variant};
```
