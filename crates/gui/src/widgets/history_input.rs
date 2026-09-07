use egui::Button;
use egui::Context;
use egui::Id;
use egui::Key;
use egui::Modifiers;
use egui::RectAlign;
use egui::ScrollArea;
use egui::TextEdit;
use egui::Ui;
use serde::Deserialize;
use serde::Serialize;
use std::collections::VecDeque;

/// Actions emitted by `HistoryInput::show` describing user intents for this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryInputAction {
    ArrowDown,
    ArrowUp,
    Delete,
    Home,
    End,
    Enter,
    Escape,
    Commit,
    Clear,
    ShowAll,
    Changed(String),
}

#[derive(Serialize, Deserialize)]
pub struct HistoryInput {
    pub id: Id,
    pub current: String,
    history: VecDeque<String>,
    filtered: Vec<String>,
    cursor: Option<usize>,
    hint_text: String,
}

impl HistoryInput {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            current: String::new(),
            history: VecDeque::new(),
            filtered: Vec::new(),
            cursor: None,
            hint_text: String::new(),
        }
    }

    pub fn with_hint(mut self, hint_text: &str) -> Self {
        self.hint_text = hint_text.to_string();
        self
    }

    pub fn show(&self, ui: &mut Ui) -> Option<HistoryInputAction> {
        let modifiers = Modifiers::NONE;
        let mut action: Option<HistoryInputAction> = None;
        let is_open = egui::Popup::is_id_open(ui.ctx(), self.id);
        if is_open {
            let ctx = ui.ctx();
            if ctx.input_mut(|i| i.consume_key(modifiers, Key::ArrowDown)) {
                action = Some(HistoryInputAction::ArrowDown);
            } else if ctx.input_mut(|i| i.consume_key(modifiers, Key::ArrowUp)) {
                action = Some(HistoryInputAction::ArrowUp);
            } else if ctx.input_mut(|i| i.consume_key(modifiers, Key::Home)) {
                action = Some(HistoryInputAction::Home);
            } else if ctx.input_mut(|i| i.consume_key(modifiers, Key::End)) {
                action = Some(HistoryInputAction::End);
            } else if ctx.input_mut(|i| i.consume_key(modifiers, Key::Enter)) {
                action = Some(HistoryInputAction::Enter);
            } else if ctx.input_mut(|i| i.consume_key(modifiers, Key::Escape)) {
                action = Some(HistoryInputAction::Escape);
            } else if ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::Delete)) {
                action = Some(HistoryInputAction::Delete);
            }
        } else {
            let ctx = ui.ctx();
            if self.current.is_empty() {
                if ctx.input_mut(|i| i.consume_key(modifiers, Key::ArrowDown))
                    || ctx.input_mut(|i| i.consume_key(modifiers, Key::ArrowUp))
                {
                    action = Some(HistoryInputAction::ShowAll);
                }
            }
        }

        let mut tmp = self.current.clone();
        let edit = TextEdit::singleline(&mut tmp);
        let edit_output = edit.show(ui);
        let r = edit_output.response;
        if r.changed() {
            action = Some(HistoryInputAction::Changed(tmp));
        }

        let open = r.has_focus() && !self.filtered.is_empty();
        if open {
            egui::Popup::open_id(ui.ctx(), self.id);
        }

        if r.lost_focus() {
            action = Some(HistoryInputAction::Commit);
        }

        egui::Popup::menu(&r)
            .align(RectAlign::BOTTOM_START)
            .open(open)
            .id(self.id)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
            .show(|ui| {
                ScrollArea::vertical()
                    .max_height(f32::INFINITY)
                    .show(ui, |ui| {
                        ui.set_min_width(r.rect.width());
                        for (idx, text) in self.filtered.iter().enumerate() {
                            let button = if self.cursor == Some(idx) {
                                Button::new(text).selected(true)
                            } else {
                                Button::new(text)
                            };

                            if ui.add(button).clicked() {
                                action = Some(HistoryInputAction::Changed(text.clone()));
                            }
                        }
                    });
            });

        action
    }

    fn update_filter(&mut self) {
        self.filtered.clear();
        for text in self.history.iter() {
            if self.current.is_empty() || !text.contains(&self.current) {
                continue;
            }

            self.filtered.push(text.clone());
        }

        self.cursor = if self.filtered.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    fn set_current(&mut self, new: String) -> bool {
        if new != self.current {
            self.current = new;
            self.update_filter();

            self.cursor = if self.filtered.is_empty() {
                None
            } else {
                Some(0)
            };

            true
        } else {
            false
        }
    }

    pub fn update(&mut self, key: HistoryInputAction) -> bool {
        match key {
            HistoryInputAction::Commit => {
                if !self.current.is_empty() {
                    if let Some(pos) = self.history.iter().position(|s| *s == self.current) {
                        self.history.remove(pos);
                    }

                    self.history.push_front(self.current.clone());
                }
                false
            }
            HistoryInputAction::Changed(new) => self.set_current(new),
            HistoryInputAction::Clear => self.set_current("".to_owned()),
            HistoryInputAction::ArrowUp => {
                if let Some(cursor) = &mut self.cursor {
                    if *cursor > 0 {
                        *cursor -= 1;
                    } else {
                        *cursor = self.filtered.len() - 1;
                    }
                }
                false
            }
            HistoryInputAction::ArrowDown => {
                if let Some(cursor) = &mut self.cursor {
                    *cursor = (*cursor + 1) % self.filtered.len();
                }
                false
            }
            HistoryInputAction::Home => {
                if let Some(cursor) = &mut self.cursor {
                    *cursor = 0;
                }
                false
            }
            HistoryInputAction::End => {
                if let Some(cursor) = &mut self.cursor {
                    *cursor = self.filtered.len() - 1;
                }
                false
            }
            HistoryInputAction::Delete => {
                if let Some(cursor) = &mut self.cursor {
                    self.filtered.remove(*cursor);
                    *cursor = self.filtered.len() - 1;
                }
                false
            }
            HistoryInputAction::Escape => false,
            HistoryInputAction::Enter => {
                if let Some(cursor) = &self.cursor {
                    self.set_current(self.filtered[*cursor].clone())
                } else {
                    false
                }
            }
            HistoryInputAction::ShowAll => {
                self.filtered = self.history.clone().into();
                self.cursor = if self.filtered.is_empty() {
                    None
                } else {
                    Some(0)
                };
                false
            }
        }
    }

    pub fn persist(&self, ctx: &Context) {
        ctx.data_mut(|data| data.insert_persisted(self.id, self.history.clone()));
    }

    pub fn restore(&mut self, ctx: &Context) {
        self.history = ctx.data_mut(|data| data.get_persisted(self.id).unwrap_or_default());
        self.update_filter();
    }
}
