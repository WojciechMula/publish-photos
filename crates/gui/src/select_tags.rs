use crate::style::Style;
use crate::widgets::tag_button;
use crate::widgets::tag_button_draggable;
use const_format::formatcp as fmt;
use db::edit_tags::Action;
use db::Database;
use db::TagList;
use db::TranslatedTag;
use db::TranslatedTagsView;
use egui::text::CCursor;
use egui::text::CCursorRange;
use egui::Align;
use egui::Area;
use egui::Button;
use egui::Context;
use egui::Id;
use egui::Key;
use egui::Layout;
use egui::Order;
use egui::Popup;
use egui::PopupAnchor;
use egui::PopupCloseBehavior;
use egui::RichText;
use egui::Sense;
use egui::SetOpenCommand;
use egui::TextEdit;
use egui::Ui;
use egui::Vec2;

use egui_material_icons::icons::ICON_ADD;
use egui_material_icons::icons::ICON_BACKSPACE;

pub struct SelectTags {
    new_tag: String,
    show_pl: bool,
    pub tags: TagList,
    pub available: Vec<TranslatedTagGroup>,
    filtered: Vec<TranslatedTagGroup>,
    autocompletion: Vec<TranslatedTag>,
    undo: Vec<Action>,
    dragged_tag: Option<usize>,
    dragged_target: Option<usize>,
    drag_start_tags: Option<TagList>,
    pub text_edit_id: Id,
    show_pl_translations: Id,
    first_run: bool,
    pub show_popup: bool,
}

pub struct TranslatedTagGroup {
    pub name: String,
    pub tags: TranslatedTagsView,
}

#[derive(Clone)]
pub enum SelectTagsAction {
    Action(Action),
    Undo,
    UpdateNewTag(String),
    ClearNewTag,
    AddNew,
    ShowPolishTranslations(bool),
}

impl From<Action> for SelectTagsAction {
    fn from(val: Action) -> Self {
        Self::Action(val)
    }
}

impl SelectTags {
    pub fn new(id: Id) -> Self {
        let show_pl_translations = Id::new("show-pl-translations");

        Self {
            new_tag: String::new(),
            show_pl: false,
            tags: TagList::default(),
            available: Vec::new(),
            filtered: Vec::new(),
            autocompletion: Vec::new(),
            undo: Vec::new(),
            dragged_tag: None,
            dragged_target: None,
            drag_start_tags: None,
            text_edit_id: Id::new((id, "select-tag")),
            show_pl_translations,
            first_run: true,
            show_popup: true,
        }
    }

    pub fn edit(id: Id, tags: &TagList) -> Self {
        Self {
            tags: tags.clone(),
            ..Self::new(id)
        }
    }

    pub fn init(&mut self, ctx: &Context) {
        if !self.first_run {
            return;
        }

        self.show_pl = ctx.data_mut(|data| {
            data.get_persisted(self.show_pl_translations)
                .unwrap_or(self.show_pl)
        });

        self.first_run = false;
    }

    pub fn update(&mut self, ctx: &Context, action: SelectTagsAction, db: &Database) {
        match action {
            SelectTagsAction::Action(action) => {
                if let Some(action) = action.apply(&mut self.tags, db) {
                    self.undo.push(action);
                }
            }
            SelectTagsAction::Undo => {
                if let Some(action) = self.undo.pop() {
                    action.apply(&mut self.tags, db);
                }
            }
            SelectTagsAction::UpdateNewTag(string) => {
                self.new_tag = string.trim().to_string();
                self.update_filters();
            }
            SelectTagsAction::ClearNewTag => {
                self.new_tag.clear();
            }
            SelectTagsAction::AddNew => {
                let action = Action::FromString(self.new_tag.clone());
                if let Some(action) = action.apply(&mut self.tags, db) {
                    self.undo.push(action);
                    self.new_tag.clear();
                }
            }
            SelectTagsAction::ShowPolishTranslations(flag) => {
                self.show_pl = flag;
                self.update_filters();
                ctx.data_mut(|data| data.insert_persisted(self.show_pl_translations, flag));
            }
        }
    }

    pub fn draw_controls(&self, ui: &mut Ui, style: &Style) -> Option<SelectTagsAction> {
        let mut result: Option<SelectTagsAction> = None;
        ui.horizontal(|ui| {
            ui.columns_const::<2, ()>(|[col1, col2]| {
                col1.horizontal(|ui| {
                    let mut tag = self.new_tag.clone();

                    let edit = TextEdit::singleline(&mut tag).id(self.text_edit_id);
                    let resp = ui.add(edit);
                    if resp.lost_focus() {
                        if ui.input(|input| input.key_pressed(Key::Enter)) {
                            result = Some(SelectTagsAction::AddNew);
                        }
                    } else if resp.changed() {
                        result = Some(SelectTagsAction::UpdateNewTag(tag.clone()));
                    }

                    let show_popup = self.show_popup
                        && !self.new_tag.is_empty()
                        && !self.autocompletion.is_empty();

                    Popup::from_response(&resp)
                        .open_memory(Some(SetOpenCommand::Bool(show_popup)))
                        .anchor(PopupAnchor::from(&resp))
                        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                        .show(|ui| {
                            ui.set_min_width(resp.rect.width());

                            for tag in self.autocompletion.iter().take(10) {
                                let enabled = true;
                                if ui
                                    .add_enabled(
                                        enabled,
                                        tag_button(tag.base(), &self.new_tag, style),
                                    )
                                    .clicked()
                                {
                                    let action = Action::AddTag(tag.clone());
                                    result = Some(action.into());
                                }
                            }
                        });

                    let button = Button::new(ICON_BACKSPACE);
                    if ui.add_enabled(!self.new_tag.is_empty(), button).clicked() {
                        result = Some(SelectTagsAction::ClearNewTag);
                    }

                    let button = Button::new(fmt!("{ICON_ADD} Add new"));
                    if ui.add_enabled(!tag.is_empty(), button).clicked() {
                        result = Some(SelectTagsAction::AddNew);
                    }
                });

                col2.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    let mut flag = self.show_pl;
                    if ui.checkbox(&mut flag, "polish translations").changed() {
                        result = Some(SelectTagsAction::ShowPolishTranslations(flag));
                    }
                });
            });
        });

        result
    }

    pub fn draw_tags(&self, ui: &mut Ui, style: &Style) -> Option<SelectTagsAction> {
        let groups = if self.new_tag.is_empty() {
            &self.available
        } else {
            &self.filtered
        };

        self.draw_tag_groups(ui, style, groups)
    }

    pub fn draw_selected_tags(&mut self, ui: &mut Ui, style: &Style) -> Option<SelectTagsAction> {
        let mut result: Option<SelectTagsAction> = None;
        let tags = self.tags.0.clone();
        let dragging = self.dragged_tag.is_some();
        let pointer_pos = ui.ctx().pointer_interact_pos();

        ui.horizontal_wrapped(|ui| {
            for (index, tag) in tags.iter().enumerate() {
                let resp = ui.add(tag_button_draggable(tag, "", style));
                let is_target = dragging
                    && self.dragged_tag != Some(index)
                    && pointer_pos.is_some_and(|pointer_pos| resp.rect.contains(pointer_pos));

                if resp.clicked() {
                    result = Some(Action::RemoveTag(tag.clone()).into());
                }

                if resp.drag_started() {
                    self.dragged_tag = Some(index);
                    self.dragged_target = None;
                    self.drag_start_tags = Some(self.tags.clone());
                }

                if dragging && self.dragged_tag != Some(index) && is_target {
                    self.dragged_target = Some(index);
                }

                if is_target {
                    ui.painter().rect_filled(
                        resp.rect,
                        3.0,
                        ui.visuals().selection.bg_fill.gamma_multiply(0.22),
                    );
                }
            }
        });

        if dragging {
            self.draw_drag_preview(ui, style);
        }

        if let Some(from) = self.dragged_tag {
            if !ui.input(|i| i.pointer.primary_down()) {
                if let Some(start_tags) = self.drag_start_tags.take() {
                    if let Some(to) = self.dragged_target {
                        let mut reordered = start_tags.clone();
                        if reordered.move_index(from, to).is_some() && reordered != self.tags {
                            result = Some(Action::AssignTags(reordered).into());
                        }
                    }
                }
                self.dragged_tag = None;
                self.dragged_target = None;
            }
        }

        result
    }

    fn draw_drag_preview(&self, ui: &mut Ui, style: &Style) {
        let Some(index) = self.dragged_tag else {
            return;
        };

        let Some(tag) = self.tags.0.get(index) else {
            return;
        };

        let Some(pointer_pos) = ui.ctx().pointer_interact_pos() else {
            return;
        };

        let pos = pointer_pos;
        Area::new(Id::new((self.text_edit_id, "drag-preview")))
            .order(Order::Foreground)
            .interactable(false)
            .fixed_pos(pos)
            .show(ui.ctx(), |ui| {
                ui.add(tag_button(tag, "", style));
            });
    }

    fn draw_tag_groups(
        &self,
        ui: &mut Ui,
        style: &Style,
        groups: &[TranslatedTagGroup],
    ) -> Option<SelectTagsAction> {
        let mut result: Option<SelectTagsAction> = None;

        for group in groups {
            let empty_list = !group.tags.iter().any(|tag| self.tag_matches_filter(tag));
            if empty_list {
                continue;
            }

            if !group.name.is_empty() {
                ui.horizontal(|ui| {
                    let enabled = !group.is_empty();
                    let mut text = RichText::new(&group.name).heading();
                    if !enabled {
                        let color = ui.style().visuals.weak_text_color();
                        text = text.color(color);
                    }

                    ui.label(text);

                    let button = Button::new(fmt!("{ICON_ADD} Add all"));
                    if ui.add_enabled(enabled, button).clicked() {
                        result = Some(Action::AddManyTags(group.tags.clone()).into());
                    }
                });
            }

            ui.horizontal_wrapped(|ui| {
                let mut needs_space = false;
                for tag in group.tags.iter() {
                    if !self.tag_matches_filter(tag) {
                        continue;
                    }

                    let base_tag = tag.base();
                    let enabled = !self.tags.contains(base_tag);

                    if needs_space {
                        ui.add_space(4.0);
                    }
                    needs_space = self.show_pl;

                    if ui
                        .add_enabled(enabled, tag_button(tag.base(), &self.new_tag, style))
                        .clicked()
                    {
                        result = Some(Action::AddTag(tag.clone()).into());
                    }

                    if self.show_pl {
                        if let TranslatedTag::Translation(trans) = &tag {
                            let button = Button::new(&trans.pl).sense(Sense::empty());
                            ui.add(button);
                        }
                    }
                }
            });
        }

        result
    }

    fn update_filters(&mut self) {
        if self.new_tag.is_empty() {
            return;
        }

        self.filtered.clear();

        let mut autocompletion = Vec::<TranslatedTag>::new();
        for group in &self.available {
            let mut filtered = TranslatedTagGroup::empty(&group.name);
            for tag in group.tags.iter().filter(|tag| self.tag_matches_filter(tag)) {
                autocompletion.push(tag.clone());
                filtered.tags.add(tag.clone());
            }

            self.filtered.push(filtered);
        }

        self.autocompletion = autocompletion;
    }

    fn tag_matches_filter(&self, tag: &TranslatedTag) -> bool {
        match tag {
            TranslatedTag::Translation(trans) => {
                if self.show_pl {
                    trans.pl.contains(&self.new_tag) | trans.en.contains(&self.new_tag)
                } else {
                    trans.en.contains(&self.new_tag)
                }
            }
            TranslatedTag::Untranslated(string) => string.contains(&self.new_tag),
        }
    }

    pub fn select_all(&mut self, ctx: &Context) {
        if let Some(mut state) = TextEdit::load_state(ctx, self.text_edit_id) {
            state.cursor.set_char_range(Some(CCursorRange::two(
                CCursor::new(0),
                CCursor::new(self.new_tag.len()),
            )));

            state.store(ctx, self.text_edit_id);
        }
    }
}

impl TranslatedTagGroup {
    pub fn empty(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            tags: TranslatedTagsView::default(),
        }
    }

    pub fn from_tags_view(name: &str, tags: TranslatedTagsView) -> Self {
        Self {
            name: name.to_owned(),
            tags,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }
}
