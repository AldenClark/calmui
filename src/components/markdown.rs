use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, LazyLock, Mutex};

use gpui::{
    Component, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    div,
};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::contracts::WithId;
use crate::id::stable_auto_id;
use crate::style::{Radius, Size};

use super::Stack;
use super::paper::Paper;
use super::text::{Text, TextTone};
use super::title::Title;
use super::utils::resolve_hsla;

#[derive(Clone)]
enum MarkdownBlock {
    Heading {
        level: u8,
        text: SharedString,
    },
    Paragraph(SharedString),
    Quote(SharedString),
    Code {
        lang: Option<SharedString>,
        code: SharedString,
    },
    BulletList(Vec<SharedString>),
    OrderedList(Vec<SharedString>),
    Rule,
}

#[derive(Clone)]
struct CachedMarkdown {
    source: String,
    blocks: Arc<Vec<MarkdownBlock>>,
}

static MARKDOWN_CACHE: LazyLock<Mutex<HashMap<u64, CachedMarkdown>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn hash_markdown(source: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}

fn level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

enum BlockBuilder {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph(String),
    Quote(String),
    Code {
        lang: Option<String>,
        text: String,
    },
    List {
        ordered: bool,
        items: Vec<String>,
        current_item: String,
    },
}

impl BlockBuilder {
    fn push_text(&mut self, value: &str) {
        match self {
            Self::Heading { text, .. }
            | Self::Paragraph(text)
            | Self::Quote(text)
            | Self::Code { text, .. } => text.push_str(value),
            Self::List { current_item, .. } => current_item.push_str(value),
        }
    }

    fn push_line_break(&mut self) {
        match self {
            Self::Heading { text, .. }
            | Self::Paragraph(text)
            | Self::Quote(text)
            | Self::Code { text, .. } => text.push('\n'),
            Self::List { current_item, .. } => current_item.push('\n'),
        }
    }

    fn start_item(&mut self) {
        if let Self::List { current_item, .. } = self {
            current_item.clear();
        }
    }

    fn end_item(&mut self) {
        if let Self::List {
            items,
            current_item,
            ..
        } = self
        {
            let text = current_item.trim().to_string();
            if !text.is_empty() {
                items.push(text);
            }
            current_item.clear();
        }
    }

    fn finish(self) -> Option<MarkdownBlock> {
        match self {
            Self::Heading { level, text } => {
                let text = text.trim().to_string();
                (!text.is_empty()).then_some(MarkdownBlock::Heading {
                    level,
                    text: text.into(),
                })
            }
            Self::Paragraph(text) => {
                let text = text.trim().to_string();
                (!text.is_empty()).then_some(MarkdownBlock::Paragraph(text.into()))
            }
            Self::Quote(text) => {
                let text = text.trim().to_string();
                (!text.is_empty()).then_some(MarkdownBlock::Quote(text.into()))
            }
            Self::Code { lang, text } => Some(MarkdownBlock::Code {
                lang: lang
                    .filter(|value| !value.trim().is_empty())
                    .map(Into::into),
                code: text.into(),
            }),
            Self::List {
                ordered,
                mut items,
                current_item,
            } => {
                let text = current_item.trim().to_string();
                if !text.is_empty() {
                    items.push(text);
                }
                if items.is_empty() {
                    None
                } else if ordered {
                    Some(MarkdownBlock::OrderedList(
                        items.into_iter().map(SharedString::from).collect(),
                    ))
                } else {
                    Some(MarkdownBlock::BulletList(
                        items.into_iter().map(SharedString::from).collect(),
                    ))
                }
            }
        }
    }
}

fn parse_blocks(source: &str) -> Vec<MarkdownBlock> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let mut blocks = Vec::new();
    let mut current: Option<BlockBuilder> = None;

    let flush = |current: &mut Option<BlockBuilder>, blocks: &mut Vec<MarkdownBlock>| {
        if let Some(builder) = current.take()
            && let Some(block) = builder.finish()
        {
            blocks.push(block);
        }
    };

    for event in Parser::new_ext(source, options) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    flush(&mut current, &mut blocks);
                    current = Some(BlockBuilder::Heading {
                        level: level_to_u8(level),
                        text: String::new(),
                    });
                }
                Tag::Paragraph => {
                    flush(&mut current, &mut blocks);
                    current = Some(BlockBuilder::Paragraph(String::new()));
                }
                Tag::BlockQuote(_) => {
                    flush(&mut current, &mut blocks);
                    current = Some(BlockBuilder::Quote(String::new()));
                }
                Tag::CodeBlock(kind) => {
                    flush(&mut current, &mut blocks);
                    let lang = match kind {
                        CodeBlockKind::Fenced(value) => Some(value.to_string()),
                        CodeBlockKind::Indented => None,
                    };
                    current = Some(BlockBuilder::Code {
                        lang,
                        text: String::new(),
                    });
                }
                Tag::List(start) => {
                    flush(&mut current, &mut blocks);
                    current = Some(BlockBuilder::List {
                        ordered: start.is_some(),
                        items: Vec::new(),
                        current_item: String::new(),
                    });
                }
                Tag::Item => {
                    if let Some(block) = current.as_mut() {
                        block.start_item();
                    }
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Heading(_)
                | TagEnd::Paragraph
                | TagEnd::BlockQuote(_)
                | TagEnd::CodeBlock => flush(&mut current, &mut blocks),
                TagEnd::Item => {
                    if let Some(block) = current.as_mut() {
                        block.end_item();
                    }
                }
                TagEnd::List(_) => flush(&mut current, &mut blocks),
                _ => {}
            },
            Event::Text(value)
            | Event::Code(value)
            | Event::InlineMath(value)
            | Event::DisplayMath(value) => {
                if let Some(block) = current.as_mut() {
                    block.push_text(value.as_ref());
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(block) = current.as_mut() {
                    block.push_line_break();
                }
            }
            Event::Rule => {
                flush(&mut current, &mut blocks);
                blocks.push(MarkdownBlock::Rule);
            }
            Event::TaskListMarker(checked) => {
                if let Some(block) = current.as_mut() {
                    if checked {
                        block.push_text("[x] ");
                    } else {
                        block.push_text("[ ] ");
                    }
                }
            }
            _ => {}
        }
    }

    flush(&mut current, &mut blocks);
    blocks
}

fn cached_blocks(source: &str) -> Arc<Vec<MarkdownBlock>> {
    let key = hash_markdown(source);
    if let Ok(cache) = MARKDOWN_CACHE.lock()
        && let Some(cached) = cache.get(&key)
        && cached.source == source
    {
        return cached.blocks.clone();
    }

    let parsed = Arc::new(parse_blocks(source));
    if let Ok(mut cache) = MARKDOWN_CACHE.lock() {
        if cache.len() > 256 {
            cache.clear();
        }
        cache.insert(
            key,
            CachedMarkdown {
                source: source.to_string(),
                blocks: parsed.clone(),
            },
        );
    }
    parsed
}

pub struct Markdown {
    id: String,
    source: SharedString,
    compact: bool,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
}

impl Markdown {
    #[track_caller]
    pub fn new(source: impl Into<SharedString>) -> Self {
        Self {
            id: stable_auto_id("markdown"),
            source: source.into(),
            compact: false,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
        }
    }

    pub fn compact(mut self, value: bool) -> Self {
        self.compact = value;
        self
    }
}

impl WithId for Markdown {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl RenderOnce for Markdown {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let blocks = cached_blocks(self.source.as_ref());
        let mut root = Stack::vertical().id(self.id.clone()).w_full();
        root = if self.compact {
            root.gap_1p5()
        } else {
            root.gap_2()
        };

        for (index, block) in blocks.iter().cloned().enumerate() {
            let element = match block {
                MarkdownBlock::Heading { level, text } => {
                    Title::new(text).order(level.clamp(1, 6)).into_any_element()
                }
                MarkdownBlock::Paragraph(text) => Text::new(text).into_any_element(),
                MarkdownBlock::Quote(text) => div()
                    .id(format!("{}-quote-{index}", self.id))
                    .w_full()
                    .pl_3()
                    .py_1()
                    .border_1()
                    .border_color(resolve_hsla(
                        &self.theme,
                        &self.theme.semantic.border_subtle,
                    ))
                    .bg(resolve_hsla(&self.theme, &self.theme.semantic.bg_surface))
                    .child(Text::new(text).tone(TextTone::Secondary))
                    .into_any_element(),
                MarkdownBlock::Code { lang, code } => {
                    let mut content = Stack::vertical().gap_1();
                    if let Some(lang) = lang {
                        content = content.child(
                            Text::new(lang)
                                .size(Size::Xs)
                                .tone(TextTone::Muted)
                                .truncate(true),
                        );
                    }
                    content = content.child(
                        div()
                            .text_sm()
                            .text_color(resolve_hsla(
                                &self.theme,
                                &self.theme.semantic.text_primary,
                            ))
                            .child(code),
                    );
                    Paper::new()
                        .padding(Size::Sm)
                        .radius(Radius::Sm)
                        .bordered(true)
                        .child(content)
                        .into_any_element()
                }
                MarkdownBlock::BulletList(items) => {
                    let mut list = Stack::vertical().gap_1();
                    for item in items {
                        list = list.child(
                            div()
                                .flex()
                                .flex_row()
                                .gap_2()
                                .items_start()
                                .child(Text::new("•").tone(TextTone::Secondary))
                                .child(div().flex_1().min_w_0().child(Text::new(item))),
                        );
                    }
                    list.into_any_element()
                }
                MarkdownBlock::OrderedList(items) => {
                    let mut list = Stack::vertical().gap_1();
                    for (order, item) in items.into_iter().enumerate() {
                        list = list.child(
                            div()
                                .flex()
                                .flex_row()
                                .gap_2()
                                .items_start()
                                .child(
                                    Text::new(format!("{}.", order + 1)).tone(TextTone::Secondary),
                                )
                                .child(div().flex_1().min_w_0().child(Text::new(item))),
                        );
                    }
                    list.into_any_element()
                }
                MarkdownBlock::Rule => div()
                    .id(format!("{}-rule-{index}", self.id))
                    .w_full()
                    .h(gpui::px(1.0))
                    .bg(resolve_hsla(
                        &self.theme,
                        &self.theme.semantic.border_subtle,
                    ))
                    .into_any_element(),
            };
            root = root.child(element);
        }

        root
    }
}

impl IntoElement for Markdown {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemeOverridable for Markdown {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl gpui::Styled for Markdown {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
