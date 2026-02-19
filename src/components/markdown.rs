use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, LazyLock, Mutex};

use gpui::{
    InteractiveElement, IntoElement, ParentElement, Refineable, RenderOnce, SharedString, Styled,
    div,
};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use super::Stack;
use super::title::Title;
use super::utils::resolve_hsla;
use crate::id::ComponentId;

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

#[derive(IntoElement)]
pub struct Markdown {
    id: ComponentId,
    source: SharedString,
    compact: bool,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
}

impl Markdown {
    #[track_caller]
    pub fn new(source: impl Into<SharedString>) -> Self {
        Self {
            id: ComponentId::default(),
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

impl Markdown {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for Markdown {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = &self.theme.components.markdown;
        let blocks = cached_blocks(self.source.as_ref());
        let mut root = Stack::vertical().id(self.id.clone()).w_full();
        root = if self.compact {
            root.gap(tokens.gap_compact)
        } else {
            root.gap(tokens.gap_regular)
        };
        let paragraph_color = resolve_hsla(&self.theme, &tokens.paragraph);
        let quote_bg = resolve_hsla(&self.theme, &tokens.quote_bg);
        let quote_border = resolve_hsla(&self.theme, &tokens.quote_border);
        let quote_fg = resolve_hsla(&self.theme, &tokens.quote_fg);
        let code_bg = resolve_hsla(&self.theme, &tokens.code_bg);
        let code_border = resolve_hsla(&self.theme, &tokens.code_border);
        let code_fg = resolve_hsla(&self.theme, &tokens.code_fg);
        let code_lang_fg = resolve_hsla(&self.theme, &tokens.code_lang_fg);
        let list_marker = resolve_hsla(&self.theme, &tokens.list_marker);
        let rule_color = resolve_hsla(&self.theme, &tokens.rule);

        for (index, block) in blocks.iter().cloned().enumerate() {
            let element = match block {
                MarkdownBlock::Heading { level, text } => {
                    Title::new(text).order(level.clamp(1, 6)).into_any_element()
                }
                MarkdownBlock::Paragraph(text) => div()
                    .id(self.id.slot_index("paragraph", index.to_string()))
                    .w_full()
                    .text_size(tokens.paragraph_size)
                    .text_color(paragraph_color)
                    .child(text)
                    .into_any_element(),
                MarkdownBlock::Quote(text) => div()
                    .id(self.id.slot_index("quote", index.to_string()))
                    .w_full()
                    .px(tokens.quote_padding_x)
                    .py(tokens.quote_padding_y)
                    .text_size(tokens.quote_size)
                    .text_color(quote_fg)
                    .rounded(tokens.quote_radius)
                    .border(super::utils::quantized_stroke_px(window, 1.0))
                    .border_color(quote_border)
                    .bg(quote_bg)
                    .child(text)
                    .into_any_element(),
                MarkdownBlock::Code { lang, code } => {
                    let mut content = Stack::vertical().gap(tokens.code_gap);
                    if let Some(lang) = lang {
                        content = content.child(
                            div()
                                .text_size(tokens.code_lang_size)
                                .text_color(code_lang_fg)
                                .truncate()
                                .child(lang),
                        );
                    }
                    content = content.child(
                        div()
                            .text_size(tokens.code_size)
                            .text_color(code_fg)
                            .child(code),
                    );
                    div()
                        .id(self.id.slot_index("code", index.to_string()))
                        .w_full()
                        .p(tokens.code_padding)
                        .rounded(tokens.code_radius)
                        .bg(code_bg)
                        .border(super::utils::quantized_stroke_px(window, 1.0))
                        .border_color(code_border)
                        .child(content)
                        .into_any_element()
                }
                MarkdownBlock::BulletList(items) => {
                    let mut list = Stack::vertical().gap(tokens.list_gap);
                    for item in items {
                        list = list.child(
                            div()
                                .flex()
                                .flex_row()
                                .gap(tokens.list_item_gap)
                                .items_start()
                                .child(
                                    div()
                                        .text_size(tokens.list_size)
                                        .text_color(list_marker)
                                        .child("•"),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .text_size(tokens.list_size)
                                        .text_color(paragraph_color)
                                        .child(item),
                                ),
                        );
                    }
                    list.into_any_element()
                }
                MarkdownBlock::OrderedList(items) => {
                    let mut list = Stack::vertical().gap(tokens.list_gap);
                    for (order, item) in items.into_iter().enumerate() {
                        list = list.child(
                            div()
                                .flex()
                                .flex_row()
                                .gap(tokens.list_item_gap)
                                .items_start()
                                .child(
                                    div()
                                        .text_size(tokens.list_size)
                                        .text_color(list_marker)
                                        .child(format!("{}.", order + 1)),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .text_size(tokens.list_size)
                                        .text_color(paragraph_color)
                                        .child(item),
                                ),
                        );
                    }
                    list.into_any_element()
                }
                MarkdownBlock::Rule => div()
                    .id(self.id.slot_index("rule", index.to_string()))
                    .w_full()
                    .h(super::utils::hairline_px(window))
                    .bg(rule_color)
                    .into_any_element(),
            };
            root = root.child(element);
        }

        root.style().refine(&self.style);
        root
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
