#[derive(Clone, Debug)]
pub struct InputState {
    pub value: String,
    pub caret: usize,
    pub anchor: usize,
    pub selection: Option<(usize, usize)>,
}

impl InputState {
    pub fn new(
        value: impl Into<String>,
        caret: usize,
        anchor: usize,
        selection: Option<(usize, usize)>,
    ) -> Self {
        let value = value.into();
        let len = value.chars().count();
        let caret = caret.min(len);
        let anchor = anchor.min(len);
        let selection = Self::normalize_selection(selection, len);
        Self {
            value,
            caret,
            anchor,
            selection,
        }
    }

    pub fn len(&self) -> usize {
        self.value.chars().count()
    }

    pub fn selected_text(&self) -> String {
        let Some((start, end)) = self.selection else {
            return String::new();
        };
        self.value
            .chars()
            .skip(start)
            .take(end.saturating_sub(start))
            .collect()
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
        self.anchor = self.caret;
    }

    pub fn set_caret(&mut self, next_caret: usize, extend: bool) {
        let next_caret = next_caret.min(self.len());
        if extend {
            self.caret = next_caret;
            self.selection = Self::selection_from_anchor(self.anchor, self.caret);
        } else {
            self.caret = next_caret;
            self.clear_selection();
        }
    }

    pub fn move_left(&mut self, extend: bool) {
        if !extend {
            if let Some((start, _)) = self.selection {
                self.set_caret(start, false);
                return;
            }
            self.set_caret(self.caret.saturating_sub(1), false);
            return;
        }
        self.set_caret(self.caret.saturating_sub(1), true);
    }

    pub fn move_right(&mut self, extend: bool) {
        if !extend {
            if let Some((_, end)) = self.selection {
                self.set_caret(end, false);
                return;
            }
            self.set_caret((self.caret + 1).min(self.len()), false);
            return;
        }
        self.set_caret((self.caret + 1).min(self.len()), true);
    }

    pub fn move_to(&mut self, next_caret: usize, extend: bool) {
        self.set_caret(next_caret, extend);
    }

    pub fn delete_backward(&mut self) -> bool {
        if let Some((start, end)) = self.selection {
            self.replace_char_range(start, end, "");
            return true;
        }
        if self.caret == 0 {
            return false;
        }
        let delete_start = self.caret - 1;
        self.replace_char_range(delete_start, self.caret, "");
        true
    }

    pub fn delete_forward(&mut self) -> bool {
        if let Some((start, end)) = self.selection {
            self.replace_char_range(start, end, "");
            return true;
        }
        if self.caret >= self.len() {
            return false;
        }
        let delete_end = (self.caret + 1).min(self.len());
        self.replace_char_range(self.caret, delete_end, "");
        true
    }

    pub fn insert_text(&mut self, text: &str) -> bool {
        if text.is_empty() {
            return false;
        }
        if let Some((start, end)) = self.selection {
            self.replace_char_range(start, end, text);
        } else {
            self.replace_char_range(self.caret, self.caret, text);
        }
        true
    }

    pub fn replace_char_range(&mut self, start: usize, end: usize, insert: &str) {
        let len = self.len();
        let start = start.min(len);
        let end = end.min(len).max(start);
        let byte_start = Self::byte_index_at_char(&self.value, start);
        let byte_end = Self::byte_index_at_char(&self.value, end);
        self.value.replace_range(byte_start..byte_end, insert);
        self.caret = (start + insert.chars().count()).min(self.len());
        self.clear_selection();
    }

    pub fn clamp_to_max_length(&mut self, max_length: Option<usize>) -> bool {
        let Some(limit) = max_length else {
            return false;
        };
        if self.len() <= limit {
            return false;
        }
        self.value = self.value.chars().take(limit).collect::<String>();
        let len = self.len();
        self.caret = self.caret.min(len);
        self.anchor = self.anchor.min(len);
        self.selection = Self::normalize_selection(self.selection, len);
        true
    }

    pub fn set_selection_from_anchor(&mut self, anchor: usize, caret: usize) {
        self.anchor = anchor.min(self.len());
        self.caret = caret.min(self.len());
        self.selection = Self::selection_from_anchor(self.anchor, self.caret);
    }

    pub fn byte_index_at_char(value: &str, char_index: usize) -> usize {
        value
            .char_indices()
            .nth(char_index)
            .map(|(index, _)| index)
            .unwrap_or(value.len())
    }

    fn normalize_selection(
        selection: Option<(usize, usize)>,
        len: usize,
    ) -> Option<(usize, usize)> {
        let (start, end) = selection?;
        let start = start.min(len);
        let end = end.min(len);
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        (start < end).then_some((start, end))
    }

    fn selection_from_anchor(anchor: usize, caret: usize) -> Option<(usize, usize)> {
        if anchor == caret {
            None
        } else if anchor < caret {
            Some((anchor, caret))
        } else {
            Some((caret, anchor))
        }
    }
}
