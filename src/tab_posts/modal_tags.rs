use crate::application::Message as MainMessage;
use crate::confirm::Confirm;
use crate::confirm::ConfirmOption;
use crate::db::Database;
use crate::db::PostId;
use crate::db::Selector;
use crate::db::TagList;
use crate::db::TranslatedTag;
use crate::db::TranslatedTagsView;
use crate::edit_details::EditDetails;
use crate::edit_tags::Action;
use crate::gui::add_image;
use crate::gui::button;
use crate::help;
use crate::keyboard::KeyboardMapping;
use crate::select_tags::SelectTags;
use crate::select_tags::SelectTagsAction;
use crate::select_tags::TranslatedTagGroup;
use crate::style::Style;
use crate::tab_posts::Message as TabMessage;
use crate::tab_posts::MessageQueue as TabMessageQueue;
use crate::widgets::tag_button;
use const_format::formatcp as fmt;
use egui::CentralPanel;
use egui::CollapsingHeader;
use egui::Context;
use egui::Key;
use egui::Label;
use egui::Layout;
use egui::ScrollArea;
use egui::SidePanel;
use egui::TopBottomPanel;
use egui::Ui;
use std::cell::LazyCell;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::collections::VecDeque;

use egui_material_icons::icons::ICON_ADD;
use egui_material_icons::icons::ICON_WARNING;

const ID_PREFIX: &str = "modal-post-tags";

pub struct ModalTags {
    id: PostId,
    select_tags: SelectTags,
    original: TagList,
    context_menu_opened: bool,
    tag_groups_opened: bool,
    frequent_tags_opened: bool,
    tag_groups_opened_flag: Option<bool>,
    frequent_tags_opened_flag: Option<bool>,

    pub queue: MessageQueue,
    pub keyboard_mapping: LazyCell<KeyboardMapping>,
}

type MessageQueue = VecDeque<Message>;

#[derive(Clone)]
pub enum Message {
    TagAction(SelectTagsAction),
    Undo,
    SoftClose,
    SaveAndExit,
    CancelAndExit,
    ToggleTagGroups,
    ToggleFrequentTags,
}

impl Message {
    pub const fn name(&self) -> &str {
        match self {
            Self::TagAction(_) => unreachable!(),
            Self::Undo => help::UNDO,
            Self::SoftClose => help::SOFT_CLOSE,
            Self::SaveAndExit => help::SAVE_AND_EXIT,
            Self::CancelAndExit => unreachable!(),
            Self::ToggleTagGroups => "show/hide list of tag groups",
            Self::ToggleFrequentTags => "show/hide list frequent tags",
        }
    }
}

impl From<Message> for TabMessage {
    fn from(val: Message) -> Self {
        Self::ModalTags(val)
    }
}

impl From<SelectTagsAction> for Message {
    fn from(val: SelectTagsAction) -> Self {
        Self::TagAction(val)
    }
}

impl From<Action> for Message {
    fn from(val: Action) -> Self {
        Self::TagAction(val.into())
    }
}

impl ModalTags {
    pub fn new(id: PostId, db: &Database) -> Self {
        let post = db.post(&id);
        let original = post.tags.clone();

        let mut select_tags = SelectTags::edit(&post.tags);

        let by_date = db.get_tags_view(&Selector::ByDate(post.date)).clone();
        let by_month = &db.get_tags_view(&Selector::ByMonth(post.date.month)).0;
        let by_month: BTreeSet<_> = by_month.difference(&by_date.0).cloned().collect();

        let all = &db.get_tags_view(&Selector::All).0;
        let all: BTreeSet<_> = all.difference(&by_date.0).cloned().collect();
        let all: BTreeSet<_> = all.difference(&by_month).cloned().collect();

        select_tags.available = vec![
            TranslatedTagGroup::from_tags_view("From this day", by_date),
            TranslatedTagGroup::from_tags_view("From this month", TranslatedTagsView(by_month)),
            TranslatedTagGroup::from_tags_view("All", TranslatedTagsView(all)),
        ];

        Self {
            id,
            original,
            select_tags,
            context_menu_opened: false,
            tag_groups_opened: true,
            frequent_tags_opened: true,
            tag_groups_opened_flag: Some(true),
            frequent_tags_opened_flag: Some(true),
            queue: MessageQueue::new(),
            keyboard_mapping: LazyCell::new(Self::create_mapping),
        }
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        style: &Style,
        db: &Database,
        tab_queue: &mut TabMessageQueue,
    ) {
        while let Some(message) = self.queue.pop_front() {
            self.handle_message(style, db, message, tab_queue);
        }

        //self.show_pl =
        //    ctx.data_mut(|data| data.get_persisted(self.show_pl_id).unwrap_or(self.show_pl));

        let mut queue = MessageQueue::new();

        self.draw(ctx, style, db, &mut queue);

        while let Some(msg) = queue.pop_front() {
            self.queue.push_back(msg);
        }

        self.context_menu_opened = ctx.is_popup_open();
        self.tag_groups_opened_flag = None;
        self.frequent_tags_opened_flag = None;
    }

    fn create_mapping() -> KeyboardMapping {
        fn msg(msg: Message) -> MainMessage {
            MainMessage::TabPosts(TabMessage::ModalTags(msg))
        }

        KeyboardMapping::default()
            .key(Key::Escape, msg(Message::SoftClose))
            .ctrl(Key::G, msg(Message::ToggleTagGroups))
            .ctrl(Key::F, msg(Message::ToggleFrequentTags))
            .ctrl(Key::Z, msg(Message::Undo))
            .ctrl(Key::S, msg(Message::SaveAndExit))
    }

    fn handle_message(
        &mut self,
        style: &Style,
        db: &Database,
        message: Message,
        tab_queue: &mut TabMessageQueue,
    ) {
        match message {
            Message::TagAction(action) => {
                self.select_tags.update(action, db);
            }
            Message::Undo => {
                self.queue.push_back(SelectTagsAction::Undo.into());
            }
            Message::SoftClose => {
                if !self.context_menu_opened {
                    if self.is_modified() {
                        let msg: TabMessage = Message::SaveAndExit.into();
                        let save = ConfirmOption::new("Save and exit")
                            .with_message(msg.into())
                            .with_color(style.button.save);

                        let msg: TabMessage = Message::CancelAndExit.into();
                        let abort = ConfirmOption::new(fmt!("{ICON_WARNING} Abandon changes"))
                            .with_message(msg.into())
                            .with_color(style.button.discard);

                        let cont = ConfirmOption::new("Continue").with_key(Key::Escape);

                        let confirm =
                            Confirm::new("The tags got changed.", vec![abort, save, cont]);

                        tab_queue.push_back(TabMessage::Confirm(confirm));
                    } else {
                        tab_queue.push_back(TabMessage::CloseModal);
                    }
                }
            }
            Message::CancelAndExit => {
                tab_queue.push_back(TabMessage::CloseModal);
            }
            Message::SaveAndExit => {
                tab_queue.push_back(TabMessage::CloseModal);
                if self.is_modified() {
                    let msg = EditDetails::SetTags(self.id, self.select_tags.tags.clone());
                    tab_queue.push_back(msg.into());
                }
            }
            Message::ToggleTagGroups => {
                self.tag_groups_opened = !self.tag_groups_opened;
                self.tag_groups_opened_flag = Some(self.tag_groups_opened);
            }
            Message::ToggleFrequentTags => {
                self.frequent_tags_opened = !self.frequent_tags_opened;
                self.frequent_tags_opened_flag = Some(self.frequent_tags_opened);
            }
        }
    }

    fn draw(&self, ctx: &Context, style: &Style, db: &Database, queue: &mut MessageQueue) {
        SidePanel::left(fmt!("{ID_PREFIX}-left"))
            .resizable(false)
            .min_width(style.image.preview_width)
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .id_salt(fmt!("{ID_PREFIX}-pictures-scroll"))
                    .show(ui, |ui| {
                        let post = db.post(&self.id);
                        for uri in &post.uris {
                            add_image(
                                ui,
                                uri.clone(),
                                style.image.preview_width,
                                style.image.radius,
                            );
                        }
                    });
            });

        TopBottomPanel::bottom(fmt!("{ID_PREFIX}-bottom")).show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                if button::save(ui, self.is_modified(), Some(style.button.save)) {
                    queue.push_back(Message::SaveAndExit);
                }
                if button::cancel(ui) {
                    queue.push_back(Message::SoftClose);
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                self.view_selected_tags(ui, db, style, queue);

                self.draw_tag_groups(ui, db, queue);

                ui.separator();

                self.draw_frequent_tags(ui, style, queue);
            });
        });
    }

    fn view_selected_tags(
        &self,
        ui: &mut Ui,
        db: &Database,
        style: &Style,
        queue: &mut MessageQueue,
    ) {
        ui.horizontal_wrapped(|ui| {
            for tag in self.select_tags.tags.iter() {
                let resp = ui.add(tag_button(tag, "", style));

                let hints = mk_hints(tag, db, &self.select_tags.tags);
                if !hints.is_empty() {
                    resp.context_menu(|ui| {
                        for tag in hints {
                            if ui.add(tag_button(tag.base(), "", style)).clicked() {
                                let action = Action::AddTag(tag);
                                queue.push_back(action.into());
                            }
                        }
                    });
                }

                if resp.clicked() {
                    let action = Action::RemoveTag(tag.clone());
                    queue.push_back(action.into());
                }
            }
        });

        ui.separator();

        if let Some(action) = self.select_tags.draw_controls(ui, style) {
            queue.push_back(action.into());
        }
    }

    fn draw_tag_groups(&self, ui: &mut Ui, db: &Database, queue: &mut MessageQueue) {
        if db.tag_groups.is_empty() {
            return;
        }

        CollapsingHeader::new("Tag groups")
            .id_salt(fmt!("{ID_PREFIX}-tag-groups"))
            .default_open(true)
            .open(self.tag_groups_opened_flag)
            .show(ui, |ui| {
                for group in db.tag_groups.iter() {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.heading(&group.name);

                        if ui.button(fmt!("{ICON_ADD} Add all")).clicked() {
                            let action = Action::FromTagGroup(group.id);
                            queue.push_back(action.into());
                        }

                        let tags = Label::new(group.tags.as_str()).truncate().selectable(false);
                        ui.add(tags);
                    });
                }
            });
    }

    fn draw_frequent_tags(&self, ui: &mut Ui, style: &Style, queue: &mut MessageQueue) {
        CollapsingHeader::new("Frequent tags")
            .id_salt(fmt!("{ID_PREFIX}-frequent-tags"))
            .default_open(true)
            .open(self.frequent_tags_opened_flag)
            .show(ui, |ui| {
                if let Some(action) = self.select_tags.draw_tags(ui, style) {
                    queue.push_back(action.into());
                }
            });
    }

    fn is_modified(&self) -> bool {
        self.select_tags.tags != self.original
    }
}

fn mk_hints(tag: &String, db: &Database, existing: &TagList) -> Vec<TranslatedTag> {
    let Some(hints) = db.tag_hints.lookup(tag) else {
        return Vec::new();
    };

    let mut result = Vec::<TranslatedTag>::new();

    let mut seen = HashSet::<String>::new();
    for tag in existing.iter() {
        seen.insert(tag.clone());
    }

    for tag in hints.iter() {
        if seen.contains(tag) {
            continue;
        }

        let trans = db.tag_translations.as_tag(tag);
        match &trans {
            TranslatedTag::Translation(trans) => {
                seen.insert(trans.pl.clone());
                seen.insert(trans.en.clone());
            }
            TranslatedTag::Untranslated(tag) => {
                seen.insert(tag.clone());
            }
        }

        result.push(trans);
    }

    result
}
