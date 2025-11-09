use crate::db::Database;
use crate::db::TagList;
use crate::db::TranslatedTag;
use crate::db::TranslatedTagsView;
use crate::edit_tags::Action;
use crate::style::Style;
use crate::widgets::tag_button;
use const_format::formatcp as fmt;
use egui::Align;
use egui::Button;
use egui::Key;
use egui::Layout;
use egui::Popup;
use egui::PopupAnchor;
use egui::PopupCloseBehavior;
use egui::RichText;
use egui::Sense;
use egui::SetOpenCommand;
use egui::Ui;

use egui_material_icons::icons::ICON_ADD;
use egui_material_icons::icons::ICON_BACKSPACE;

#[derive(Default)]
pub struct SelectTags {
    new_tag: String,
    show_pl: bool,
    pub tags: TagList,
    pub available: Vec<TranslatedTagGroup>,
    filtered: Vec<TranslatedTagGroup>,
    autocompletion: Vec<TranslatedTag>,
    undo: Vec<Action>,
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn edit(tags: &TagList) -> Self {
        Self {
            tags: tags.clone(),
            ..Default::default()
        }
    }

    pub fn update(&mut self, action: SelectTagsAction, db: &Database) {
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
                //ctx.data_mut(|data| data.insert_persisted(self.show_pl_id, self.show_pl));
            }
        }
    }

    pub fn draw_controls(&self, ui: &mut Ui, style: &Style) -> Option<SelectTagsAction> {
        let mut result: Option<SelectTagsAction> = None;
        ui.horizontal(|ui| {
            ui.columns_const::<2, ()>(|[col1, col2]| {
                col1.horizontal(|ui| {
                    let mut tag = self.new_tag.clone();

                    let resp = ui.text_edit_singleline(&mut tag);
                    if resp.lost_focus() {
                        if ui.input(|input| input.key_pressed(Key::Enter)) {
                            result = Some(SelectTagsAction::AddNew);
                        }
                    } else if resp.changed() {
                        result = Some(SelectTagsAction::UpdateNewTag(tag.clone()));
                    }

                    let show_popup = !self.new_tag.is_empty() && !self.autocompletion.is_empty();

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

    fn draw_tag_groups(
        &self,
        ui: &mut Ui,
        style: &Style,
        groups: &[TranslatedTagGroup],
    ) -> Option<SelectTagsAction> {
        let mut result: Option<SelectTagsAction> = None;

        for group in groups {
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
