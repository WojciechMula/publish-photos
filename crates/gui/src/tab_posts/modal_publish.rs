use crate::application::Message as MainMessage;
use crate::gui::add_image;
use crate::gui::button;
use crate::image_cache::ImageCache;
use crate::keyboard::KeyboardMapping;
use crate::style::Style;
use crate::tab_posts::Message as TabMessage;
use crate::tab_posts::MessageQueue as TabMessageQueue;
use crate::widgets::checkmark;
use const_format::formatcp as fmt;
use db::edit_details::EditDetails;
use db::Database;
use db::Post;
use db::PostId;
use egui::vec2;
use egui::Align;
use egui::Button;
use egui::CentralPanel;
use egui::Context;
use egui::Key;
use egui::Layout;
use egui::ScrollArea;
use egui::SidePanel;
use egui::TopBottomPanel;
use std::collections::VecDeque;

use egui_material_icons::icons::ICON_CONTENT_COPY;
use egui_material_icons::icons::ICON_PUBLISH;

const ID_PREFIX: &str = "publish-image";

pub struct ModalPublish {
    id: PostId,
    entries: Vec<Entry>,
    sm_available: bool,

    pub queue: MessageQueue,
    pub keyboard_mapping: KeyboardMapping,
}

type MessageQueue = VecDeque<Message>;

struct Entry {
    copied: bool,
    text: String,
    label: String,
}

#[derive(Clone)]
pub enum Message {
    Publish,
    Copy1,
    Copy2,
    Copy3,
    Copy4,
    Copy5,
    Copy6,
    Copy7,
    Copy8,
    Copy9,
    Cancel,
    PublishOnSocialMedia,
}

impl Message {
    pub const fn name(&self) -> &str {
        match self {
            Self::Publish => "mark post as published and close window",
            Self::PublishOnSocialMedia => unreachable!(),
            Self::Copy1 => "copy text to clipboard",
            Self::Copy2 => "copy path of 1st photo",
            Self::Copy3 => "copy path of 2nd photo",
            Self::Copy4 => "copy path of 3rd photo",
            Self::Copy5 => "copy path of 4th photo",
            Self::Copy6 => "copy path of 5th photo",
            Self::Copy7 => "copy path of 6th photo",
            Self::Copy8 => "copy path of 7th photo",
            Self::Copy9 => "copy path of 8th photo",
            Self::Cancel => "cancel publishing",
        }
    }
}

impl ModalPublish {
    pub fn new(sm_available: bool, id: PostId, db: &Database) -> Self {
        let post = db.post(&id);

        let text = render_text(post, db);
        let mut entries = vec![Entry {
            copied: false,
            label: "Copy text".to_owned(),
            text,
        }];
        for item in &post.files {
            let full_path = item.full_path.display().to_string();
            entries.push(Entry {
                label: full_path.clone(),
                text: full_path,
                copied: false,
            });
        }

        Self {
            id,
            entries,
            queue: MessageQueue::new(),
            keyboard_mapping: Self::create_mapping(),
            sm_available,
        }
    }

    fn create_mapping() -> KeyboardMapping {
        fn msg(msg: Message) -> MainMessage {
            MainMessage::TabPosts(TabMessage::ModalPublish(msg))
        }

        KeyboardMapping::default()
            .key(Key::Escape, msg(Message::Cancel))
            .key(Key::Num1, msg(Message::Copy1))
            .key(Key::Num2, msg(Message::Copy2))
            .key(Key::Num3, msg(Message::Copy3))
            .key(Key::Num4, msg(Message::Copy4))
            .key(Key::Num5, msg(Message::Copy5))
            .key(Key::Num6, msg(Message::Copy6))
            .key(Key::Num7, msg(Message::Copy7))
            .key(Key::Num8, msg(Message::Copy8))
            .key(Key::Num9, msg(Message::Copy9))
            .ctrl(Key::S, msg(Message::Publish))
            .ctrl(Key::P, msg(Message::Publish))
    }

    fn handle_message(&mut self, msg: Message, tab_queue: &mut TabMessageQueue) {
        match msg {
            Message::Cancel => {
                tab_queue.push_back(TabMessage::CloseModal);
            }
            Message::Copy1 => {
                if let Some(entry) = self.entries.get_mut(0) {
                    entry.copied = true;
                    tab_queue.push_back(TabMessage::Copy(entry.text.clone()));
                }
            }
            Message::Copy2 => {
                if let Some(entry) = self.entries.get_mut(1) {
                    entry.copied = true;
                    tab_queue.push_back(TabMessage::Copy(entry.text.clone()));
                }
            }
            Message::Copy3 => {
                if let Some(entry) = self.entries.get_mut(2) {
                    entry.copied = true;
                    tab_queue.push_back(TabMessage::Copy(entry.text.clone()));
                }
            }
            Message::Copy4 => {
                if let Some(entry) = self.entries.get_mut(3) {
                    entry.copied = true;
                    tab_queue.push_back(TabMessage::Copy(entry.text.clone()));
                }
            }
            Message::Copy5 => {
                if let Some(entry) = self.entries.get_mut(4) {
                    entry.copied = true;
                    tab_queue.push_back(TabMessage::Copy(entry.text.clone()));
                }
            }
            Message::Copy6 => {
                if let Some(entry) = self.entries.get_mut(5) {
                    entry.copied = true;
                    tab_queue.push_back(TabMessage::Copy(entry.text.clone()));
                }
            }
            Message::Copy7 => {
                if let Some(entry) = self.entries.get_mut(6) {
                    entry.copied = true;
                    tab_queue.push_back(TabMessage::Copy(entry.text.clone()));
                }
            }
            Message::Copy8 => {
                if let Some(entry) = self.entries.get_mut(7) {
                    entry.copied = true;
                    tab_queue.push_back(TabMessage::Copy(entry.text.clone()));
                }
            }
            Message::Copy9 => {
                if let Some(entry) = self.entries.get_mut(8) {
                    entry.copied = true;
                    tab_queue.push_back(TabMessage::Copy(entry.text.clone()));
                }
            }
            Message::Publish => {
                tab_queue.push_back(EditDetails::SetPublished(self.id).into());
                tab_queue.push_back(TabMessage::CloseModal);
            }
            Message::PublishOnSocialMedia => {
                tab_queue.push_back(TabMessage::StartPublishing(self.id));
                tab_queue.push_back(TabMessage::CloseModal);
            }
        }
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &mut Database,
        tab_queue: &mut TabMessageQueue,
    ) {
        while let Some(msg) = self.queue.pop_front() {
            self.handle_message(msg, tab_queue);
        }

        let post = db.post(&self.id);

        SidePanel::left(fmt!("{ID_PREFIX}-left"))
            .resizable(false)
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .id_salt(fmt!("{ID_PREFIX}-pictures-scroll"))
                    .show(ui, |ui| {
                        for meta in &post.files {
                            add_image(
                                ui,
                                meta,
                                image_cache,
                                style.image.preview_width,
                                style.image.radius,
                            );
                        }
                    });
            });

        TopBottomPanel::bottom(fmt!("{ID_PREFIX}-buttons")).show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                if self.sm_available {
                    let button = Button::new(fmt!("{ICON_PUBLISH} Publish on social media"))
                        .fill(style.button.publish);

                    if ui.add(button).clicked() {
                        self.queue.push_back(Message::PublishOnSocialMedia);
                    }
                } else {
                    let button = Button::new(fmt!("{ICON_PUBLISH} Mark as published"))
                        .fill(style.button.publish);

                    if ui.add(button).clicked() {
                        self.queue.push_back(Message::Publish);
                    }
                }

                if button::cancel(ui) {
                    self.queue.push_back(Message::Cancel);
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                let mut read_only: &str = &self.entries[0].text;
                ui.text_edit_multiline(&mut read_only);

                ui.separator();

                ScrollArea::vertical()
                    .id_salt(fmt!("{ID_PREFIX}-buttons-scroll"))
                    .show(ui, |ui| {
                        for entry in self.entries.iter_mut() {
                            ui.horizontal(|ui| {
                                ui.add(checkmark(entry.copied, style.copied_mark));

                                let button =
                                    Button::new(format!("{ICON_CONTENT_COPY} {}", entry.label))
                                        .min_size(vec2(ui.available_width(), 0.0));

                                if ui.add(button).clicked() {
                                    tab_queue.push_back(TabMessage::Copy(entry.text.clone()));
                                    entry.copied = true;
                                }
                            });
                        }
                    });
            });
        });
    }

    pub fn try_close(&mut self) {
        self.queue.push_back(Message::Cancel);
    }
}

// --------------------------------------------------

const PL_EMOJI: &str = "🇵🇱";
const EN_EMOJI: &str = "🇬🇧";

pub fn render_text(post: &Post, db: &Database) -> String {
    let mut f = Builder::default();

    if !post.pl.is_empty() {
        f.writeln(format!("{PL_EMOJI} {}", post.pl));
    }

    if !post.en.is_empty() {
        f.writeln(format!("{EN_EMOJI} {}", post.en));
    }

    if post.species.is_some() && !f.is_empty() {
        f.newline();

        let latin = post.species.as_ref().unwrap();
        let species = db.species_by_latin(latin).unwrap();
        let latin = latin.as_str();

        let pl = format_species(PL_EMOJI, &species.pl);
        let en = format_species(EN_EMOJI, &species.en);
        f.writeln(match (pl, en) {
            (None, None) => latin.to_owned(),
            (Some(pl), None) => format!("{latin} ({pl})"),
            (None, Some(en)) => format!("{latin} ({en})"),
            (Some(pl), Some(en)) => format!("{latin} ({pl} {en})"),
        });
    }

    if f.is_empty() && post.species.is_some() {
        let latin = post.species.as_ref().unwrap();
        let species = db.species_by_latin(latin).unwrap();

        if !species.pl.is_empty() {
            f.writeln(format!(
                "{PL_EMOJI} {} ({})",
                species.pl,
                &species.latin.as_str()
            ));
            if !species.en.is_empty() {
                f.writeln(format!("{EN_EMOJI} {}", species.en));
            }
        } else if !species.en.is_empty() {
            f.writeln(format!(
                "{EN_EMOJI} {} ({})",
                species.en,
                species.latin.as_str()
            ));
        } else {
            f.writeln(species.latin.as_str().to_string());
        }
    }

    if !f.is_empty() {
        f.newline()
    }

    for (id, tag) in post.tags.iter().enumerate() {
        if id > 0 {
            f.write(format!(" #{tag}"));
        } else {
            f.write(format!("#{tag}"));
        }
    }

    f.buf
}

fn format_species(emoji: &str, name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }

    Some(format!("{emoji} {name}"))
}

#[derive(Default)]
struct Builder {
    buf: String,
}

impl Builder {
    fn writeln(&mut self, s: String) {
        self.buf.push_str(&s);
        self.buf.push('\n');
    }

    fn write(&mut self, s: String) {
        self.buf.push_str(&s);
    }

    fn newline(&mut self) {
        self.buf.push('\n');
    }

    fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}
