use egui::Context;
use egui::Id;
use serde::Deserialize;
use serde::Serialize;

/// Actions emitted by `HistoryInput::show` describing user intents for this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryInputAction {
    None,
    /// The user modified the input string.
    TextChanged(String),
    /// The user explicitly deleted or canceled the autocompleted tail.
    CancelAutocomplete(String),
    /// The user submitted a non-empty string.
    Submit(String),
    /// The user requested navigation through history.
    NavigateHistory(HistoryDirection),
    /// The user requested autocompletion using a match from history.
    Autocomplete {
        current_len: usize,
        matched_text: String,
    },
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryDirection {
    Up,
    Down,
}

#[derive(Default, Serialize, Deserialize)]
pub struct HistoryInput {
    pub current_text: String,
    pub history: Vec<String>,
    history_index: Option<usize>,
    draft_text: String,
    hint_text: String,
}

impl HistoryInput {
    pub fn with_hint(mut self, hint_text: &str) -> Self {
        self.hint_text = hint_text.to_string();

        self
    }

    /// Phase 1: Pure UI rendering. Reads state and returns an action without mutating `self`.
    pub fn show(&self, ui: &mut egui::Ui, id: Id) -> HistoryInputAction {
        let mut text = self.current_text.clone();
        let text_edit = egui::TextEdit::singleline(&mut text)
            .id(id)
            .hint_text(&self.hint_text);
        let output = text_edit.show(ui);
        let response = output.response;

        // 1. Handle submission (Enter key)
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let trimmed = self.current_text.trim();
            if !trimmed.is_empty() {
                return HistoryInputAction::Submit(trimmed.to_string());
            }
        }

        // 2. Handle history navigation keypresses
        if response.has_focus() {
            let (up, down) = ui.input(|i| {
                (
                    i.key_pressed(egui::Key::ArrowUp),
                    i.key_pressed(egui::Key::ArrowDown),
                )
            });

            if up {
                return HistoryInputAction::NavigateHistory(HistoryDirection::Up);
            }
            if down {
                return HistoryInputAction::NavigateHistory(HistoryDirection::Down);
            }
        }

        // 3. Handle Autocomplete and Deletion / Backspace
        if response.has_focus() && self.history_index.is_none() {
            // Check if user pressed Backspace, Delete, or performed a Cut operation
            let is_deletion = ui.input(|i| {
                i.key_pressed(egui::Key::Backspace)
                    || i.key_pressed(egui::Key::Delete)
                    || i.events.iter().any(|e| matches!(e, egui::Event::Cut))
            });

            // If the user presses Backspace/Delete while autocompleted tail is active,
            // cancel autocomplete and revert to what the user had actually typed (`text` emitted by TextEdit).
            if is_deletion {
                // `text` holds the string after TextEdit processed the keypress
                return HistoryInputAction::CancelAutocomplete(text);
            }

            // Trigger autocomplete only if text changed via normal typing
            if response.changed() {
                if let Some(action) = self.check_autocomplete() {
                    return action;
                }
            }
        }

        // 4. Handle standard text changes
        if response.changed() {
            return HistoryInputAction::TextChanged(text);
        }

        HistoryInputAction::None
    }

    /// Phase 2: State mutation. Updates internal state using only `egui::Context` and the widget `Id`.
    pub fn update(&mut self, ctx: &egui::Context, id: Id, action: HistoryInputAction) {
        match action {
            HistoryInputAction::None => {}

            HistoryInputAction::TextChanged(new_text)
            | HistoryInputAction::CancelAutocomplete(new_text) => {
                self.current_text = new_text;
            }

            HistoryInputAction::Submit(trimmed) => {
                if self.history.last().map(|s| s.as_str()) != Some(&trimmed) {
                    self.history.push(trimmed);
                }
                self.current_text.clear();
                self.draft_text.clear();
                self.history_index = None;

                ctx.memory_mut(|m| m.request_focus(id));
            }

            HistoryInputAction::NavigateHistory(direction) => {
                ctx.input_mut(|i| match direction {
                    HistoryDirection::Up => {
                        i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)
                    }
                    HistoryDirection::Down => {
                        i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
                    }
                });

                match direction {
                    HistoryDirection::Up => self.navigate_up(),
                    HistoryDirection::Down => self.navigate_down(),
                }
            }

            HistoryInputAction::Autocomplete {
                current_len,
                matched_text,
            } => {
                self.current_text = matched_text;

                if let Some(mut state) = egui::TextEdit::load_state(ctx, id) {
                    let primary_cursor = egui::text::CCursor::new(current_len);
                    let secondary_cursor = egui::text::CCursor::new(self.current_text.len());

                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::two(
                            primary_cursor,
                            secondary_cursor,
                        )));

                    state.store(ctx, id);
                }
            }

            HistoryInputAction::Clear => {
                self.current_text.clear();
                self.history_index = None;
            }
        }
    }

    fn check_autocomplete(&self) -> Option<HistoryInputAction> {
        if self.current_text.is_empty() {
            return None;
        }

        let current_len = self.current_text.len();
        let match_found = self
            .history
            .iter()
            .rev()
            .find(|h| h.starts_with(&self.current_text) && h.len() > current_len)
            .cloned()?;

        Some(HistoryInputAction::Autocomplete {
            current_len,
            matched_text: match_found,
        })
    }

    fn navigate_up(&mut self) {
        if let Some(idx) = self.history_index {
            if idx > 0 {
                self.history_index = Some(idx - 1);
                self.current_text = self.history[idx - 1].clone();
            }
        } else if !self.history.is_empty() {
            self.draft_text = self.current_text.clone();
            let last_idx = self.history.len() - 1;
            self.history_index = Some(last_idx);
            self.current_text = self.history[last_idx].clone();
        }
    }

    fn navigate_down(&mut self) {
        if let Some(idx) = self.history_index {
            if idx + 1 < self.history.len() {
                self.history_index = Some(idx + 1);
                self.current_text = self.history[idx + 1].clone();
            } else {
                self.history_index = None;
                self.current_text = self.draft_text.clone();
            }
        }
    }

    pub fn persist(&self, ctx: &Context, id: Id) {
        ctx.data_mut(|data| data.insert_persisted(id, self.history.clone()));
    }

    pub fn restore(&mut self, ctx: &Context, id: Id) {
        self.history = ctx.data_mut(|data| data.get_persisted(id).unwrap_or_default());
    }
}

impl HistoryInputAction {
    pub const fn is_some(&self) -> bool {
        !matches!(self, Self::None)
    }
}
