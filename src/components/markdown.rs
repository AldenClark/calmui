use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::{Arc, LazyLock, Mutex};

use gpui::{
    AnyElement, ElementId, FontStyle, FontWeight, InteractiveElement, IntoElement, ParentElement,
    Refineable, RenderOnce, SharedString, StatefulInteractiveElement, StrikethroughStyle, Styled,
    StyledText, TextRun, UnderlineStyle, Window, div, font, img, px,
};
use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use super::Stack;
use super::utils::resolve_hsla;
use crate::id::ComponentId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkdownLinkClick {
    pub href: SharedString,
    pub title: Option<SharedString>,
    pub text: SharedString,
    pub is_external: bool,
}

type LinkClickHandler = Rc<dyn Fn(&MarkdownLinkClick, &mut Window, &mut gpui::App)>;

#[derive(Clone, Debug)]
enum MarkdownBlock {
    Heading {
        level: u8,
        inlines: Vec<InlineNode>,
    },
    Paragraph(Vec<InlineNode>),
    Quote(Vec<MarkdownBlock>),
    Code {
        lang: Option<SharedString>,
        code: SharedString,
    },
    List {
        ordered: bool,
        start: u64,
        items: Vec<ListItem>,
    },
    Table {
        alignments: Vec<Alignment>,
        header: Vec<TableCell>,
        rows: Vec<Vec<TableCell>>,
    },
    Image {
        src: SharedString,
        title: Option<SharedString>,
        alt: SharedString,
    },
    Rule,
}

#[derive(Clone, Debug, Default)]
struct ListItem {
    checked: Option<bool>,
    blocks: Vec<MarkdownBlock>,
}

#[derive(Clone, Debug, Default)]
struct TableCell {
    inlines: Vec<InlineNode>,
}

#[derive(Clone, Debug)]
enum InlineNode {
    Text(SharedString),
    SoftBreak,
    HardBreak,
    Code(SharedString),
    Math {
        display: bool,
        content: SharedString,
    },
    Emphasis(Vec<InlineNode>),
    Strong(Vec<InlineNode>),
    Strikethrough(Vec<InlineNode>),
    Mark(Vec<InlineNode>),
    Kbd(Vec<InlineNode>),
    Link {
        href: SharedString,
        title: Option<SharedString>,
        children: Vec<InlineNode>,
    },
    Image {
        src: SharedString,
        title: Option<SharedString>,
        alt: Vec<InlineNode>,
    },
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

#[derive(Default)]
struct TableAcc {
    alignments: Vec<Alignment>,
    header: Vec<TableCell>,
    rows: Vec<Vec<TableCell>>,
}

#[derive(Default)]
struct TableHeadAcc {
    rows: Vec<Vec<TableCell>>,
}

#[derive(Default)]
struct TableRowAcc {
    cells: Vec<TableCell>,
}

enum ParseContainer {
    Root {
        blocks: Vec<MarkdownBlock>,
    },
    Heading {
        level: u8,
        inlines: Vec<InlineNode>,
    },
    Paragraph {
        inlines: Vec<InlineNode>,
    },
    Quote {
        blocks: Vec<MarkdownBlock>,
    },
    Code {
        lang: Option<SharedString>,
        text: String,
    },
    List {
        ordered: bool,
        start: u64,
        items: Vec<ListItem>,
    },
    Item {
        checked: Option<bool>,
        blocks: Vec<MarkdownBlock>,
    },
    Table(TableAcc),
    TableHead(TableHeadAcc),
    TableRow(TableRowAcc),
    TableCell(TableCell),
    Emphasis {
        inlines: Vec<InlineNode>,
    },
    Strong {
        inlines: Vec<InlineNode>,
    },
    Strikethrough {
        inlines: Vec<InlineNode>,
    },
    Mark {
        inlines: Vec<InlineNode>,
    },
    Kbd {
        inlines: Vec<InlineNode>,
    },
    Link {
        href: SharedString,
        title: Option<SharedString>,
        inlines: Vec<InlineNode>,
    },
    Image {
        src: SharedString,
        title: Option<SharedString>,
        alt: Vec<InlineNode>,
    },
}

fn append_text(inlines: &mut Vec<InlineNode>, text: &str) {
    if text.is_empty() {
        return;
    }

    if let Some(InlineNode::Text(current)) = inlines.last_mut() {
        let mut joined = current.to_string();
        joined.push_str(text);
        *current = joined.into();
        return;
    }

    inlines.push(InlineNode::Text(text.to_string().into()));
}

fn push_inline(container: &mut ParseContainer, inline: InlineNode) -> bool {
    match container {
        ParseContainer::Heading { inlines, .. }
        | ParseContainer::Paragraph { inlines }
        | ParseContainer::Emphasis { inlines }
        | ParseContainer::Strong { inlines }
        | ParseContainer::Strikethrough { inlines }
        | ParseContainer::Mark { inlines }
        | ParseContainer::Kbd { inlines }
        | ParseContainer::Link { inlines, .. }
        | ParseContainer::Image { alt: inlines, .. } => {
            inlines.push(inline);
            true
        }
        ParseContainer::TableCell(TableCell { inlines }) => {
            inlines.push(inline);
            true
        }
        _ => false,
    }
}

fn push_text(stack: &mut [ParseContainer], text: &str) {
    for container in stack.iter_mut().rev() {
        match container {
            ParseContainer::Heading { inlines, .. }
            | ParseContainer::Paragraph { inlines }
            | ParseContainer::Emphasis { inlines }
            | ParseContainer::Strong { inlines }
            | ParseContainer::Strikethrough { inlines }
            | ParseContainer::Mark { inlines }
            | ParseContainer::Kbd { inlines }
            | ParseContainer::Link { inlines, .. }
            | ParseContainer::Image { alt: inlines, .. } => {
                append_text(inlines, text);
                return;
            }
            ParseContainer::TableCell(TableCell { inlines }) => {
                append_text(inlines, text);
                return;
            }
            _ => {}
        }
    }
}

fn push_block(stack: &mut [ParseContainer], block: MarkdownBlock) {
    for container in stack.iter_mut().rev() {
        match container {
            ParseContainer::Root { blocks }
            | ParseContainer::Quote { blocks }
            | ParseContainer::Item { blocks, .. } => {
                blocks.push(block);
                return;
            }
            _ => {}
        }
    }
}

fn trim_inline_whitespace(inlines: &[InlineNode]) -> Vec<InlineNode> {
    let mut out = Vec::new();
    for inline in inlines {
        match inline {
            InlineNode::Text(value) if value.trim().is_empty() => {}
            _ => out.push(inline.clone()),
        }
    }
    out
}

fn flatten_inline_text(inlines: &[InlineNode]) -> String {
    fn walk(buffer: &mut String, nodes: &[InlineNode]) {
        for node in nodes {
            match node {
                InlineNode::Text(value)
                | InlineNode::Code(value)
                | InlineNode::Math { content: value, .. } => buffer.push_str(value),
                InlineNode::SoftBreak | InlineNode::HardBreak => buffer.push(' '),
                InlineNode::Emphasis(children)
                | InlineNode::Strong(children)
                | InlineNode::Strikethrough(children)
                | InlineNode::Mark(children)
                | InlineNode::Kbd(children)
                | InlineNode::Link {
                    children,
                    href: _,
                    title: _,
                } => walk(buffer, children),
                InlineNode::Image { alt, .. } => walk(buffer, alt),
            }
        }
    }

    let mut text = String::new();
    walk(&mut text, inlines);
    text.trim().to_string()
}

fn finish_paragraph(inlines: Vec<InlineNode>) -> Option<MarkdownBlock> {
    let trimmed = trim_inline_whitespace(&inlines);
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.len() == 1 {
        if let InlineNode::Image { src, title, alt } = &trimmed[0] {
            let alt_text = flatten_inline_text(alt);
            return Some(MarkdownBlock::Image {
                src: src.clone(),
                title: title.clone(),
                alt: alt_text.into(),
            });
        }
    }

    Some(MarkdownBlock::Paragraph(trimmed))
}

fn parse_blocks(source: &str) -> Vec<MarkdownBlock> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let mut stack = vec![ParseContainer::Root { blocks: Vec::new() }];

    for event in Parser::new_ext(source, options) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => stack.push(ParseContainer::Heading {
                    level: level_to_u8(level),
                    inlines: Vec::new(),
                }),
                Tag::Paragraph => stack.push(ParseContainer::Paragraph {
                    inlines: Vec::new(),
                }),
                Tag::BlockQuote(_) => stack.push(ParseContainer::Quote { blocks: Vec::new() }),
                Tag::CodeBlock(kind) => {
                    let lang = match kind {
                        CodeBlockKind::Fenced(value) => {
                            let raw = value.trim();
                            if raw.is_empty() {
                                None
                            } else {
                                Some(raw.to_string().into())
                            }
                        }
                        CodeBlockKind::Indented => None,
                    };
                    stack.push(ParseContainer::Code {
                        lang,
                        text: String::new(),
                    });
                }
                Tag::List(start) => stack.push(ParseContainer::List {
                    ordered: start.is_some(),
                    start: start.unwrap_or(1),
                    items: Vec::new(),
                }),
                Tag::Item => stack.push(ParseContainer::Item {
                    checked: None,
                    blocks: Vec::new(),
                }),
                Tag::Table(alignments) => stack.push(ParseContainer::Table(TableAcc {
                    alignments,
                    header: Vec::new(),
                    rows: Vec::new(),
                })),
                Tag::TableHead => stack.push(ParseContainer::TableHead(TableHeadAcc::default())),
                Tag::TableRow => stack.push(ParseContainer::TableRow(TableRowAcc::default())),
                Tag::TableCell => stack.push(ParseContainer::TableCell(TableCell::default())),
                Tag::Emphasis => stack.push(ParseContainer::Emphasis {
                    inlines: Vec::new(),
                }),
                Tag::Strong => stack.push(ParseContainer::Strong {
                    inlines: Vec::new(),
                }),
                Tag::Strikethrough => stack.push(ParseContainer::Strikethrough {
                    inlines: Vec::new(),
                }),
                Tag::Link {
                    dest_url, title, ..
                } => stack.push(ParseContainer::Link {
                    href: dest_url.to_string().into(),
                    title: (!title.trim().is_empty()).then_some(title.to_string().into()),
                    inlines: Vec::new(),
                }),
                Tag::Image {
                    dest_url, title, ..
                } => stack.push(ParseContainer::Image {
                    src: dest_url.to_string().into(),
                    title: (!title.trim().is_empty()).then_some(title.to_string().into()),
                    alt: Vec::new(),
                }),
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Heading(_) => {
                    if let Some(ParseContainer::Heading { level, inlines }) = stack.pop() {
                        let trimmed = trim_inline_whitespace(&inlines);
                        if !trimmed.is_empty() {
                            push_block(
                                &mut stack,
                                MarkdownBlock::Heading {
                                    level,
                                    inlines: trimmed,
                                },
                            );
                        }
                    }
                }
                TagEnd::Paragraph => {
                    if let Some(ParseContainer::Paragraph { inlines }) = stack.pop()
                        && let Some(block) = finish_paragraph(inlines)
                    {
                        push_block(&mut stack, block);
                    }
                }
                TagEnd::BlockQuote(_) => {
                    if let Some(ParseContainer::Quote { blocks }) = stack.pop()
                        && !blocks.is_empty()
                    {
                        push_block(&mut stack, MarkdownBlock::Quote(blocks));
                    }
                }
                TagEnd::CodeBlock => {
                    if let Some(ParseContainer::Code { lang, text }) = stack.pop() {
                        push_block(
                            &mut stack,
                            MarkdownBlock::Code {
                                lang,
                                code: text.into(),
                            },
                        );
                    }
                }
                TagEnd::Item => {
                    if let Some(ParseContainer::Item { checked, blocks }) = stack.pop() {
                        for container in stack.iter_mut().rev() {
                            if let ParseContainer::List { items, .. } = container {
                                items.push(ListItem { checked, blocks });
                                break;
                            }
                        }
                    }
                }
                TagEnd::List(_) => {
                    if let Some(ParseContainer::List {
                        ordered,
                        start,
                        items,
                    }) = stack.pop()
                        && !items.is_empty()
                    {
                        push_block(
                            &mut stack,
                            MarkdownBlock::List {
                                ordered,
                                start,
                                items,
                            },
                        );
                    }
                }
                TagEnd::TableCell => {
                    if let Some(ParseContainer::TableCell(cell)) = stack.pop() {
                        for container in stack.iter_mut().rev() {
                            if let ParseContainer::TableRow(row) = container {
                                row.cells.push(cell);
                                break;
                            }
                        }
                    }
                }
                TagEnd::TableRow => {
                    if let Some(ParseContainer::TableRow(row)) = stack.pop() {
                        for container in stack.iter_mut().rev() {
                            match container {
                                ParseContainer::TableHead(head) => {
                                    head.rows.push(row.cells);
                                    break;
                                }
                                ParseContainer::Table(table) => {
                                    table.rows.push(row.cells);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                TagEnd::TableHead => {
                    if let Some(ParseContainer::TableHead(head)) = stack.pop() {
                        for container in stack.iter_mut().rev() {
                            if let ParseContainer::Table(table) = container {
                                if let Some(first_row) = head.rows.into_iter().next() {
                                    table.header = first_row;
                                }
                                break;
                            }
                        }
                    }
                }
                TagEnd::Table => {
                    if let Some(ParseContainer::Table(table)) = stack.pop()
                        && (!table.header.is_empty() || !table.rows.is_empty())
                    {
                        push_block(
                            &mut stack,
                            MarkdownBlock::Table {
                                alignments: table.alignments,
                                header: table.header,
                                rows: table.rows,
                            },
                        );
                    }
                }
                TagEnd::Emphasis => {
                    if let Some(ParseContainer::Emphasis { inlines }) = stack.pop() {
                        let inline = InlineNode::Emphasis(inlines);
                        for container in stack.iter_mut().rev() {
                            if push_inline(container, inline.clone()) {
                                break;
                            }
                        }
                    }
                }
                TagEnd::Strong => {
                    if let Some(ParseContainer::Strong { inlines }) = stack.pop() {
                        let inline = InlineNode::Strong(inlines);
                        for container in stack.iter_mut().rev() {
                            if push_inline(container, inline.clone()) {
                                break;
                            }
                        }
                    }
                }
                TagEnd::Strikethrough => {
                    if let Some(ParseContainer::Strikethrough { inlines }) = stack.pop() {
                        let inline = InlineNode::Strikethrough(inlines);
                        for container in stack.iter_mut().rev() {
                            if push_inline(container, inline.clone()) {
                                break;
                            }
                        }
                    }
                }
                TagEnd::Link => {
                    if let Some(ParseContainer::Link {
                        href,
                        title,
                        inlines,
                    }) = stack.pop()
                    {
                        let inline = InlineNode::Link {
                            href,
                            title,
                            children: inlines,
                        };
                        for container in stack.iter_mut().rev() {
                            if push_inline(container, inline.clone()) {
                                break;
                            }
                        }
                    }
                }
                TagEnd::Image => {
                    if let Some(ParseContainer::Image { src, title, alt }) = stack.pop() {
                        let inline = InlineNode::Image { src, title, alt };
                        for container in stack.iter_mut().rev() {
                            if push_inline(container, inline.clone()) {
                                break;
                            }
                        }
                    }
                }
                _ => {}
            },
            Event::Text(value) => push_text(&mut stack, value.as_ref()),
            Event::Code(value) => {
                let inline = InlineNode::Code(value.to_string().into());
                for container in stack.iter_mut().rev() {
                    if push_inline(container, inline.clone()) {
                        break;
                    }
                }
            }
            Event::InlineMath(value) => {
                let inline = InlineNode::Math {
                    display: false,
                    content: value.to_string().into(),
                };
                for container in stack.iter_mut().rev() {
                    if push_inline(container, inline.clone()) {
                        break;
                    }
                }
            }
            Event::DisplayMath(value) => {
                let inline = InlineNode::Math {
                    display: true,
                    content: value.to_string().into(),
                };
                for container in stack.iter_mut().rev() {
                    if push_inline(container, inline.clone()) {
                        break;
                    }
                }
            }
            Event::SoftBreak => {
                let inline = InlineNode::SoftBreak;
                for container in stack.iter_mut().rev() {
                    if push_inline(container, inline.clone()) {
                        break;
                    }
                }
            }
            Event::HardBreak => {
                let inline = InlineNode::HardBreak;
                for container in stack.iter_mut().rev() {
                    if push_inline(container, inline.clone()) {
                        break;
                    }
                }
            }
            Event::Rule => push_block(&mut stack, MarkdownBlock::Rule),
            Event::TaskListMarker(checked) => {
                for container in stack.iter_mut().rev() {
                    if let ParseContainer::Item { checked: state, .. } = container {
                        *state = Some(checked);
                        break;
                    }
                }
            }
            Event::InlineHtml(value) => {
                let trimmed = value.trim();
                if trimmed.eq_ignore_ascii_case("<mark>") {
                    stack.push(ParseContainer::Mark {
                        inlines: Vec::new(),
                    });
                } else if trimmed.eq_ignore_ascii_case("</mark>") {
                    if let Some(ParseContainer::Mark { inlines }) = stack.pop() {
                        let inline = InlineNode::Mark(inlines);
                        for container in stack.iter_mut().rev() {
                            if push_inline(container, inline.clone()) {
                                break;
                            }
                        }
                    }
                } else if trimmed.eq_ignore_ascii_case("<kbd>") {
                    stack.push(ParseContainer::Kbd {
                        inlines: Vec::new(),
                    });
                } else if trimmed.eq_ignore_ascii_case("</kbd>") {
                    if let Some(ParseContainer::Kbd { inlines }) = stack.pop() {
                        let inline = InlineNode::Kbd(inlines);
                        for container in stack.iter_mut().rev() {
                            if push_inline(container, inline.clone()) {
                                break;
                            }
                        }
                    }
                } else if trimmed.eq_ignore_ascii_case("<br>")
                    || trimmed.eq_ignore_ascii_case("<br />")
                {
                    let inline = InlineNode::HardBreak;
                    for container in stack.iter_mut().rev() {
                        if push_inline(container, inline.clone()) {
                            break;
                        }
                    }
                } else {
                    push_text(&mut stack, value.as_ref());
                }
            }
            Event::Html(value) => {
                // Keep raw HTML inert by rendering it as plain text.
                push_text(&mut stack, value.as_ref());
            }
            Event::FootnoteReference(value) => {
                let text = format!("[^{value}]");
                push_text(&mut stack, &text);
            }
        }
    }

    while stack.len() > 1 {
        match stack.pop() {
            Some(ParseContainer::Paragraph { inlines }) => {
                if let Some(block) = finish_paragraph(inlines) {
                    push_block(&mut stack, block);
                }
            }
            Some(ParseContainer::Heading { level, inlines }) => {
                let trimmed = trim_inline_whitespace(&inlines);
                if !trimmed.is_empty() {
                    push_block(
                        &mut stack,
                        MarkdownBlock::Heading {
                            level,
                            inlines: trimmed,
                        },
                    );
                }
            }
            Some(ParseContainer::Quote { blocks }) => {
                if !blocks.is_empty() {
                    push_block(&mut stack, MarkdownBlock::Quote(blocks));
                }
            }
            Some(ParseContainer::Code { lang, text }) => push_block(
                &mut stack,
                MarkdownBlock::Code {
                    lang,
                    code: text.into(),
                },
            ),
            Some(ParseContainer::List {
                ordered,
                start,
                items,
            }) => {
                if !items.is_empty() {
                    push_block(
                        &mut stack,
                        MarkdownBlock::List {
                            ordered,
                            start,
                            items,
                        },
                    );
                }
            }
            Some(ParseContainer::Item { checked, blocks }) => {
                for container in stack.iter_mut().rev() {
                    if let ParseContainer::List { items, .. } = container {
                        items.push(ListItem { checked, blocks });
                        break;
                    }
                }
            }
            Some(ParseContainer::Mark { inlines }) => {
                let inline = InlineNode::Mark(inlines);
                for container in stack.iter_mut().rev() {
                    if push_inline(container, inline.clone()) {
                        break;
                    }
                }
            }
            Some(ParseContainer::Kbd { inlines }) => {
                let inline = InlineNode::Kbd(inlines);
                for container in stack.iter_mut().rev() {
                    if push_inline(container, inline.clone()) {
                        break;
                    }
                }
            }
            Some(ParseContainer::Link {
                href,
                title,
                inlines,
            }) => {
                let inline = InlineNode::Link {
                    href,
                    title,
                    children: inlines,
                };
                for container in stack.iter_mut().rev() {
                    if push_inline(container, inline.clone()) {
                        break;
                    }
                }
            }
            Some(ParseContainer::Image { src, title, alt }) => {
                let inline = InlineNode::Image { src, title, alt };
                for container in stack.iter_mut().rev() {
                    if push_inline(container, inline.clone()) {
                        break;
                    }
                }
            }
            Some(ParseContainer::Root { .. }) | None => {}
            _ => {}
        }
    }

    match stack.pop() {
        Some(ParseContainer::Root { blocks }) => blocks,
        _ => Vec::new(),
    }
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

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
struct InlineStyle {
    strong: bool,
    emphasis: bool,
    strike: bool,
    code: bool,
    mark: bool,
    kbd: bool,
    math: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct LinkMeta {
    href: SharedString,
    title: Option<SharedString>,
}

#[derive(Clone, Debug)]
struct InlineSegment {
    text: SharedString,
    style: InlineStyle,
    link: Option<LinkMeta>,
}

#[derive(Clone, Copy)]
struct InlinePalette {
    paragraph: gpui::Hsla,
    link: gpui::Hsla,
    strong: gpui::Hsla,
    emphasis: gpui::Hsla,
    del: gpui::Hsla,
    inline_code_fg: gpui::Hsla,
    inline_code_bg: gpui::Hsla,
    mark_fg: gpui::Hsla,
    mark_bg: gpui::Hsla,
    kbd_fg: gpui::Hsla,
    kbd_bg: gpui::Hsla,
    inline_code_border: gpui::Hsla,
    kbd_border: gpui::Hsla,
}

fn flatten_inlines(
    inlines: &[InlineNode],
    style: &InlineStyle,
    link: Option<LinkMeta>,
    output: &mut Vec<InlineSegment>,
) {
    for inline in inlines {
        match inline {
            InlineNode::Text(value) => {
                if value.is_empty() {
                    continue;
                }
                let should_merge = output
                    .last()
                    .map(|segment| segment.style == *style && segment.link == link)
                    .unwrap_or(false);
                if should_merge {
                    if let Some(last) = output.last_mut() {
                        let mut joined = last.text.to_string();
                        joined.push_str(value);
                        last.text = joined.into();
                    }
                } else {
                    output.push(InlineSegment {
                        text: value.clone(),
                        style: style.clone(),
                        link: link.clone(),
                    });
                }
            }
            InlineNode::SoftBreak => {
                flatten_inlines(
                    &[InlineNode::Text("\n".into())],
                    style,
                    link.clone(),
                    output,
                );
            }
            InlineNode::HardBreak => {
                flatten_inlines(
                    &[InlineNode::Text("\n".into())],
                    style,
                    link.clone(),
                    output,
                );
            }
            InlineNode::Code(value) => {
                let mut next = style.clone();
                next.code = true;
                flatten_inlines(
                    &[InlineNode::Text(value.clone())],
                    &next,
                    link.clone(),
                    output,
                );
            }
            InlineNode::Math { display, content } => {
                let mut next = style.clone();
                next.math = true;
                let math_text: SharedString = if *display {
                    format!("\n{content}\n").into()
                } else {
                    content.clone()
                };
                flatten_inlines(&[InlineNode::Text(math_text)], &next, link.clone(), output);
            }
            InlineNode::Emphasis(children) => {
                let mut next = style.clone();
                next.emphasis = true;
                flatten_inlines(children, &next, link.clone(), output);
            }
            InlineNode::Strong(children) => {
                let mut next = style.clone();
                next.strong = true;
                flatten_inlines(children, &next, link.clone(), output);
            }
            InlineNode::Strikethrough(children) => {
                let mut next = style.clone();
                next.strike = true;
                flatten_inlines(children, &next, link.clone(), output);
            }
            InlineNode::Mark(children) => {
                let mut next = style.clone();
                next.mark = true;
                flatten_inlines(children, &next, link.clone(), output);
            }
            InlineNode::Kbd(children) => {
                let mut next = style.clone();
                next.kbd = true;
                flatten_inlines(children, &next, link.clone(), output);
            }
            InlineNode::Link {
                href,
                title,
                children,
            } => {
                let next_link = Some(LinkMeta {
                    href: href.clone(),
                    title: title.clone(),
                });
                flatten_inlines(children, style, next_link, output);
            }
            InlineNode::Image { alt, src, .. } => {
                if alt.is_empty() {
                    flatten_inlines(
                        &[InlineNode::Text(src.clone())],
                        style,
                        link.clone(),
                        output,
                    );
                } else {
                    flatten_inlines(alt, style, link.clone(), output);
                }
            }
        }
    }
}

fn is_system_openable_url(url: &str) -> bool {
    let lower = url.trim().to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:")
}

fn is_external_url(url: &str) -> bool {
    let lower = url.trim().to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

fn interactive_text_from_inlines(
    id: ElementId,
    inlines: &[InlineNode],
    palette: InlinePalette,
    on_link_click: Option<LinkClickHandler>,
    open_links_with_system: bool,
) -> AnyElement {
    let mut segments = Vec::new();
    flatten_inlines(inlines, &InlineStyle::default(), None, &mut segments);

    if segments.is_empty() {
        return div().into_any_element();
    }

    let mut text = String::new();
    let mut runs = Vec::new();
    let mut clickable_ranges = Vec::new();
    let mut link_payloads = Vec::new();

    for segment in segments {
        if segment.text.is_empty() {
            continue;
        }

        let start = text.len();
        text.push_str(segment.text.as_ref());
        let len = segment.text.len();

        let mut run = TextRun {
            len,
            font: font(".SystemUIFont"),
            color: palette.paragraph,
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        if segment.style.strong {
            run.font.weight = FontWeight::SEMIBOLD;
            run.color = palette.strong;
        }
        if segment.style.emphasis {
            run.font.style = FontStyle::Italic;
            run.color = palette.emphasis;
        }
        if segment.style.strike {
            run.color = palette.del;
            run.strikethrough = Some(StrikethroughStyle {
                thickness: px(1.0),
                color: Some(palette.del),
            });
        }
        if segment.style.code || segment.style.math {
            run.font = font("SFMono-Regular");
            run.font.weight = FontWeight::MEDIUM;
            run.color = palette.inline_code_fg;
            run.background_color = Some(palette.inline_code_bg);
            run.underline = Some(UnderlineStyle {
                thickness: px(1.0),
                color: Some(palette.inline_code_border),
                wavy: false,
            });
        }
        if segment.style.mark {
            run.color = palette.mark_fg;
            run.background_color = Some(palette.mark_bg);
        }
        if segment.style.kbd {
            run.font = font("SFMono-Regular");
            run.font.weight = FontWeight::MEDIUM;
            run.color = palette.kbd_fg;
            run.background_color = Some(palette.kbd_bg);
            run.underline = Some(UnderlineStyle {
                thickness: px(1.0),
                color: Some(palette.kbd_border),
                wavy: false,
            });
        }

        if let Some(link) = &segment.link {
            run.color = palette.link;
            run.underline = Some(UnderlineStyle {
                thickness: px(1.0),
                color: Some(palette.link),
                wavy: false,
            });

            let end = start + len;
            clickable_ranges.push(start..end);
            link_payloads.push(MarkdownLinkClick {
                href: link.href.clone(),
                title: link.title.clone(),
                text: segment.text.clone(),
                is_external: is_external_url(link.href.as_ref()),
            });
        }

        runs.push(run);
    }

    if text.is_empty() {
        return div().into_any_element();
    }

    let styled = StyledText::new(text).with_runs(runs);
    if clickable_ranges.is_empty() {
        return styled.into_any_element();
    }

    let click_handler = on_link_click.clone();
    gpui::InteractiveText::new(id, styled)
        .on_click(clickable_ranges, move |index, window, cx| {
            if let Some(payload) = link_payloads.get(index).cloned() {
                if let Some(handler) = click_handler.as_ref() {
                    handler(&payload, window, cx);
                } else if open_links_with_system && is_system_openable_url(payload.href.as_ref()) {
                    cx.open_url(payload.href.as_ref());
                }
            }
        })
        .into_any_element()
}

#[derive(IntoElement)]
pub struct Markdown {
    id: ComponentId,
    source: SharedString,
    compact: bool,
    on_link_click: Option<LinkClickHandler>,
    open_links_with_system: bool,
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
            on_link_click: None,
            open_links_with_system: true,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
        }
    }

    pub fn compact(mut self, value: bool) -> Self {
        self.compact = value;
        self
    }

    pub fn on_link_click(
        mut self,
        handler: impl Fn(&MarkdownLinkClick, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_link_click = Some(Rc::new(handler));
        self
    }

    pub fn open_links_with_system(mut self, value: bool) -> Self {
        self.open_links_with_system = value;
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

        let paragraph_color = resolve_hsla(&self.theme, &tokens.paragraph);
        let paragraph_muted = resolve_hsla(&self.theme, &tokens.paragraph_muted);
        let heading_color = resolve_hsla(&self.theme, &tokens.heading);
        let heading2_border = resolve_hsla(&self.theme, &tokens.heading2_border);
        let quote_bg = resolve_hsla(&self.theme, &tokens.quote_bg);
        let quote_border = resolve_hsla(&self.theme, &tokens.quote_border);
        let quote_fg = resolve_hsla(&self.theme, &tokens.quote_fg);
        let code_bg = resolve_hsla(&self.theme, &tokens.code_bg);
        let code_border = resolve_hsla(&self.theme, &tokens.code_border);
        let code_fg = resolve_hsla(&self.theme, &tokens.code_fg);
        let code_lang_fg = resolve_hsla(&self.theme, &tokens.code_lang_fg);
        let link_color = resolve_hsla(&self.theme, &tokens.link);
        let link_hover = resolve_hsla(&self.theme, &tokens.link_hover);
        let strong_color = resolve_hsla(&self.theme, &tokens.strong);
        let em_color = resolve_hsla(&self.theme, &tokens.em);
        let del_color = resolve_hsla(&self.theme, &tokens.del);
        let inline_code_bg = resolve_hsla(&self.theme, &tokens.inline_code_bg);
        let inline_code_border = resolve_hsla(&self.theme, &tokens.inline_code_border);
        let inline_code_fg = resolve_hsla(&self.theme, &tokens.inline_code_fg);
        let kbd_bg = resolve_hsla(&self.theme, &tokens.kbd_bg);
        let kbd_border = resolve_hsla(&self.theme, &tokens.kbd_border);
        let kbd_fg = resolve_hsla(&self.theme, &tokens.kbd_fg);
        let mark_bg = resolve_hsla(&self.theme, &tokens.mark_bg);
        let mark_fg = resolve_hsla(&self.theme, &tokens.mark_fg);
        let list_marker = resolve_hsla(&self.theme, &tokens.list_marker);
        let rule_color = resolve_hsla(&self.theme, &tokens.rule);
        let table_border = resolve_hsla(&self.theme, &tokens.table_border);
        let table_header_bg = resolve_hsla(&self.theme, &tokens.table_header_bg);
        let table_header_fg = resolve_hsla(&self.theme, &tokens.table_header_fg);
        let table_row_alt_bg = resolve_hsla(&self.theme, &tokens.table_row_alt_bg);
        let table_cell_fg = resolve_hsla(&self.theme, &tokens.table_cell_fg);
        let task_border = resolve_hsla(&self.theme, &tokens.task_border);
        let task_bg = resolve_hsla(&self.theme, &tokens.task_bg);
        let task_checked_bg = resolve_hsla(&self.theme, &tokens.task_checked_bg);
        let task_checked_fg = resolve_hsla(&self.theme, &tokens.task_checked_fg);
        let details_bg = resolve_hsla(&self.theme, &tokens.details_bg);
        let details_border = resolve_hsla(&self.theme, &tokens.details_border);
        let details_summary_fg = resolve_hsla(&self.theme, &tokens.details_summary_fg);
        let details_body_fg = resolve_hsla(&self.theme, &tokens.details_body_fg);
        let image_border = resolve_hsla(&self.theme, &tokens.image_border);
        let image_bg = resolve_hsla(&self.theme, &tokens.image_bg);
        let image_caption_fg = resolve_hsla(&self.theme, &tokens.image_caption_fg);

        let mut root = Stack::vertical().id(self.id.clone()).w_full();
        root = if self.compact {
            root.gap(tokens.gap_compact)
        } else {
            root.gap(tokens.gap_regular)
        };

        let palette = InlinePalette {
            paragraph: paragraph_color,
            link: link_color,
            strong: strong_color,
            emphasis: em_color,
            del: del_color,
            inline_code_fg,
            inline_code_bg,
            mark_fg,
            mark_bg,
            kbd_fg,
            kbd_bg,
            inline_code_border,
            kbd_border,
        };

        fn heading_style(level: u8) -> (gpui::Pixels, gpui::Pixels, FontWeight) {
            match level.clamp(1, 6) {
                1 => (px(40.0), px(48.0), FontWeight::BOLD),
                2 => (px(32.0), px(40.0), FontWeight::SEMIBOLD),
                3 => (px(26.0), px(34.0), FontWeight::SEMIBOLD),
                4 => (px(22.0), px(30.0), FontWeight::MEDIUM),
                5 => (px(18.0), px(26.0), FontWeight::MEDIUM),
                _ => (px(16.0), px(24.0), FontWeight::MEDIUM),
            }
        }

        fn apply_table_alignment<T: Styled>(cell: T, alignment: Alignment) -> T {
            match alignment {
                Alignment::Center => cell.text_center(),
                Alignment::Right => cell.text_right(),
                Alignment::Left | Alignment::None => cell.text_left(),
            }
        }

        fn render_blocks(
            markdown_id: &ComponentId,
            blocks: &[MarkdownBlock],
            window: &mut Window,
            tokens: &crate::theme::MarkdownTokens,
            palette: InlinePalette,
            heading_color: gpui::Hsla,
            heading2_border: gpui::Hsla,
            quote_bg: gpui::Hsla,
            quote_border: gpui::Hsla,
            quote_fg: gpui::Hsla,
            code_bg: gpui::Hsla,
            code_border: gpui::Hsla,
            code_fg: gpui::Hsla,
            code_lang_fg: gpui::Hsla,
            list_marker: gpui::Hsla,
            rule_color: gpui::Hsla,
            table_border: gpui::Hsla,
            table_header_bg: gpui::Hsla,
            table_header_fg: gpui::Hsla,
            table_row_alt_bg: gpui::Hsla,
            table_cell_fg: gpui::Hsla,
            task_border: gpui::Hsla,
            task_bg: gpui::Hsla,
            task_checked_bg: gpui::Hsla,
            task_checked_fg: gpui::Hsla,
            details_bg: gpui::Hsla,
            details_border: gpui::Hsla,
            details_summary_fg: gpui::Hsla,
            details_body_fg: gpui::Hsla,
            image_border: gpui::Hsla,
            image_bg: gpui::Hsla,
            image_caption_fg: gpui::Hsla,
            on_link_click: Option<LinkClickHandler>,
            open_links_with_system: bool,
            depth: usize,
            inline_counter: &mut usize,
        ) -> gpui::Div {
            let mut stack = Stack::vertical().gap(tokens.gap_regular);

            for (index, block) in blocks.iter().enumerate() {
                let id_key = format!("{depth}-{index}-{}", *inline_counter);
                let element = match block {
                    MarkdownBlock::Heading { level, inlines } => {
                        let (size, line_height, weight) = heading_style(*level);
                        let mut node = div()
                            .id(markdown_id.slot_index("heading", id_key.clone()))
                            .w_full()
                            .text_size(size)
                            .line_height(line_height)
                            .font_weight(weight)
                            .text_color(heading_color)
                            .child(interactive_text_from_inlines(
                                markdown_id.slot_index("heading-inline", id_key.clone()),
                                inlines,
                                InlinePalette {
                                    paragraph: heading_color,
                                    ..palette
                                },
                                on_link_click.clone(),
                                open_links_with_system,
                            ));

                        if *level == 2 {
                            node = node
                                .pt(tokens.heading2_padding_top)
                                .border_t(super::utils::quantized_stroke_px(window, 1.0))
                                .border_color(heading2_border);
                        }

                        node.into_any_element()
                    }
                    MarkdownBlock::Paragraph(inlines) => div()
                        .id(markdown_id.slot_index("paragraph", id_key.clone()))
                        .w_full()
                        .text_size(tokens.paragraph_size)
                        .line_height(tokens.paragraph_line_height)
                        .text_color(palette.paragraph)
                        .child(interactive_text_from_inlines(
                            markdown_id.slot_index("paragraph-inline", id_key.clone()),
                            inlines,
                            palette,
                            on_link_click.clone(),
                            open_links_with_system,
                        ))
                        .into_any_element(),
                    MarkdownBlock::Quote(children) => div()
                        .id(markdown_id.slot_index("quote", id_key.clone()))
                        .w_full()
                        .px(tokens.quote_padding_x)
                        .py(tokens.quote_padding_y)
                        .rounded(tokens.quote_radius)
                        .border(super::utils::quantized_stroke_px(window, 1.0))
                        .border_color(quote_border)
                        .bg(quote_bg)
                        .child(
                            render_blocks(
                                markdown_id,
                                children,
                                window,
                                tokens,
                                InlinePalette {
                                    paragraph: quote_fg,
                                    ..palette
                                },
                                heading_color,
                                heading2_border,
                                quote_bg,
                                quote_border,
                                quote_fg,
                                code_bg,
                                code_border,
                                code_fg,
                                code_lang_fg,
                                list_marker,
                                rule_color,
                                table_border,
                                table_header_bg,
                                table_header_fg,
                                table_row_alt_bg,
                                table_cell_fg,
                                task_border,
                                task_bg,
                                task_checked_bg,
                                task_checked_fg,
                                details_bg,
                                details_border,
                                details_summary_fg,
                                details_body_fg,
                                image_border,
                                image_bg,
                                image_caption_fg,
                                on_link_click.clone(),
                                open_links_with_system,
                                depth + 1,
                                inline_counter,
                            )
                            .gap(tokens.quote_gap),
                        )
                        .into_any_element(),
                    MarkdownBlock::Code { lang, code } => {
                        let mut content = Stack::vertical().gap(tokens.code_gap);
                        if let Some(lang) = lang {
                            content = content.child(
                                div()
                                    .text_size(tokens.code_lang_size)
                                    .text_color(code_lang_fg)
                                    .truncate()
                                    .child(lang.clone()),
                            );
                        }
                        content = content.child(
                            div()
                                .text_size(tokens.code_size)
                                .line_height(tokens.code_line_height)
                                .text_color(code_fg)
                                .child(code.clone()),
                        );

                        div()
                            .id(markdown_id.slot_index("code", id_key.clone()))
                            .w_full()
                            .p(tokens.code_padding)
                            .rounded(tokens.code_radius)
                            .bg(code_bg)
                            .border(super::utils::quantized_stroke_px(window, 1.0))
                            .border_color(code_border)
                            .child(content)
                            .into_any_element()
                    }
                    MarkdownBlock::List {
                        ordered,
                        start,
                        items,
                    } => {
                        let mut list = Stack::vertical().gap(tokens.list_gap);
                        for (item_index, item) in items.iter().enumerate() {
                            let mut marker_text = if *ordered {
                                format!("{}.", start + item_index as u64)
                            } else {
                                "•".to_string()
                            };

                            let marker = if let Some(checked) = item.checked {
                                marker_text = if checked {
                                    "✔".to_string()
                                } else {
                                    "".to_string()
                                };
                                div()
                                    .min_w(tokens.list_indent)
                                    .h(tokens.list_line_height)
                                    .rounded(tokens.inline_code_radius)
                                    .bg(if checked { task_checked_bg } else { task_bg })
                                    .border(super::utils::quantized_stroke_px(window, 1.0))
                                    .border_color(if checked {
                                        task_checked_bg
                                    } else {
                                        task_border
                                    })
                                    .text_color(if checked { task_checked_fg } else { task_bg })
                                    .text_size(tokens.list_size)
                                    .line_height(tokens.list_line_height)
                                    .text_center()
                                    .child(marker_text)
                                    .into_any_element()
                            } else {
                                div()
                                    .min_w(tokens.list_indent)
                                    .text_size(tokens.list_size)
                                    .line_height(tokens.list_line_height)
                                    .text_color(list_marker)
                                    .child(marker_text)
                                    .into_any_element()
                            };

                            list = list.child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap(tokens.list_item_gap)
                                    .items_start()
                                    .child(marker)
                                    .child(
                                        render_blocks(
                                            markdown_id,
                                            &item.blocks,
                                            window,
                                            tokens,
                                            palette,
                                            heading_color,
                                            heading2_border,
                                            quote_bg,
                                            quote_border,
                                            quote_fg,
                                            code_bg,
                                            code_border,
                                            code_fg,
                                            code_lang_fg,
                                            list_marker,
                                            rule_color,
                                            table_border,
                                            table_header_bg,
                                            table_header_fg,
                                            table_row_alt_bg,
                                            table_cell_fg,
                                            task_border,
                                            task_bg,
                                            task_checked_bg,
                                            task_checked_fg,
                                            details_bg,
                                            details_border,
                                            details_summary_fg,
                                            details_body_fg,
                                            image_border,
                                            image_bg,
                                            image_caption_fg,
                                            on_link_click.clone(),
                                            open_links_with_system,
                                            depth + 1,
                                            inline_counter,
                                        )
                                        .gap(tokens.list_gap),
                                    ),
                            );
                        }
                        list.into_any_element()
                    }
                    MarkdownBlock::Table {
                        alignments,
                        header,
                        rows,
                    } => {
                        let mut table = Stack::vertical().gap(px(0.0));

                        if !header.is_empty() {
                            let mut header_row = Stack::horizontal().gap(px(0.0));
                            for (cell_index, cell) in header.iter().enumerate() {
                                let alignment = alignments
                                    .get(cell_index)
                                    .copied()
                                    .unwrap_or(Alignment::None);
                                let cell_node = apply_table_alignment(
                                    div()
                                        .id(markdown_id.slot_index(
                                            "table-header-cell",
                                            format!("{id_key}-{cell_index}"),
                                        ))
                                        .flex_1()
                                        .min_w(px(120.0))
                                        .px(tokens.table_cell_padding_x)
                                        .py(tokens.table_cell_padding_y)
                                        .bg(table_header_bg)
                                        .border(super::utils::quantized_stroke_px(window, 1.0))
                                        .border_color(table_border)
                                        .text_size(tokens.table_size)
                                        .text_color(table_header_fg)
                                        .font_weight(FontWeight::SEMIBOLD),
                                    alignment,
                                );
                                header_row = header_row.child(cell_node.child(
                                    interactive_text_from_inlines(
                                        markdown_id.slot_index(
                                            "table-header-inline",
                                            format!("{id_key}-{cell_index}"),
                                        ),
                                        &cell.inlines,
                                        InlinePalette {
                                            paragraph: table_header_fg,
                                            ..palette
                                        },
                                        on_link_click.clone(),
                                        open_links_with_system,
                                    ),
                                ));
                            }
                            table = table.child(header_row);
                        }

                        for (row_index, row) in rows.iter().enumerate() {
                            let mut row_node = Stack::horizontal().gap(px(0.0));
                            for (cell_index, cell) in row.iter().enumerate() {
                                let alignment = alignments
                                    .get(cell_index)
                                    .copied()
                                    .unwrap_or(Alignment::None);
                                let cell_node = apply_table_alignment(
                                    div()
                                        .id(markdown_id.slot_index(
                                            "table-cell",
                                            format!("{id_key}-{row_index}-{cell_index}"),
                                        ))
                                        .flex_1()
                                        .min_w(px(120.0))
                                        .px(tokens.table_cell_padding_x)
                                        .py(tokens.table_cell_padding_y)
                                        .bg(if row_index % 2 == 1 {
                                            table_row_alt_bg
                                        } else {
                                            gpui::transparent_black()
                                        })
                                        .border(super::utils::quantized_stroke_px(window, 1.0))
                                        .border_color(table_border)
                                        .text_size(tokens.table_size)
                                        .text_color(table_cell_fg),
                                    alignment,
                                );
                                row_node =
                                    row_node.child(cell_node.child(interactive_text_from_inlines(
                                        markdown_id.slot_index(
                                            "table-inline",
                                            format!("{id_key}-{row_index}-{cell_index}"),
                                        ),
                                        &cell.inlines,
                                        InlinePalette {
                                            paragraph: table_cell_fg,
                                            ..palette
                                        },
                                        on_link_click.clone(),
                                        open_links_with_system,
                                    )));
                            }
                            table = table.child(row_node);
                        }

                        div()
                            .id(markdown_id.slot_index("table", id_key.clone()))
                            .w_full()
                            .overflow_x_scroll()
                            .rounded(tokens.table_radius)
                            .border(super::utils::quantized_stroke_px(window, 1.0))
                            .border_color(table_border)
                            .child(table)
                            .into_any_element()
                    }
                    MarkdownBlock::Image { src, title, alt } => {
                        let caption = if !alt.trim().is_empty() {
                            alt.clone()
                        } else {
                            title.clone().unwrap_or_default()
                        };

                        let mut image_block = Stack::vertical().gap(tokens.image_gap).child(
                            div()
                                .w_full()
                                .p(tokens.image_padding)
                                .rounded(tokens.image_radius)
                                .bg(image_bg)
                                .border(super::utils::quantized_stroke_px(window, 1.0))
                                .border_color(image_border)
                                .child(
                                    img(src.clone())
                                        .w_full()
                                        .rounded(tokens.image_radius)
                                        .overflow_hidden(),
                                ),
                        );

                        if !caption.trim().is_empty() {
                            image_block = image_block.child(
                                div()
                                    .text_size(tokens.image_caption_size)
                                    .line_height(tokens.paragraph_line_height)
                                    .text_color(image_caption_fg)
                                    .child(caption),
                            );
                        }

                        image_block.into_any_element()
                    }
                    MarkdownBlock::Rule => div()
                        .id(markdown_id.slot_index("rule", id_key.clone()))
                        .w_full()
                        .h(super::utils::hairline_px(window))
                        .bg(rule_color)
                        .into_any_element(),
                };

                stack = stack.child(element);
            }

            stack
        }

        let mut inline_counter = 0usize;
        root = root.child(
            render_blocks(
                &self.id,
                blocks.as_slice(),
                window,
                tokens,
                palette,
                heading_color,
                heading2_border,
                quote_bg,
                quote_border,
                quote_fg,
                code_bg,
                code_border,
                code_fg,
                code_lang_fg,
                list_marker,
                rule_color,
                table_border,
                table_header_bg,
                table_header_fg,
                table_row_alt_bg,
                table_cell_fg,
                task_border,
                task_bg,
                task_checked_bg,
                task_checked_fg,
                details_bg,
                details_border,
                details_summary_fg,
                details_body_fg,
                image_border,
                image_bg,
                image_caption_fg,
                self.on_link_click.clone(),
                self.open_links_with_system,
                0,
                &mut inline_counter,
            )
            .gap(if self.compact {
                tokens.gap_compact
            } else {
                tokens.gap_regular
            }),
        );

        // Explicitly keep these values used so future style extension is straightforward.
        let _ = (
            link_hover,
            paragraph_muted,
            details_bg,
            details_border,
            details_summary_fg,
            details_body_fg,
        );

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
