use crate::application::Message as MainMessage;
use crate::gui::add_image;
use crate::gui::button;
use crate::image_cache::ImageCache;
use crate::keyboard::KeyboardMapping;
use crate::style::Style;
use crate::tab_posts::ClipboardValue;
use crate::tab_posts::Message as TabMessage;
use crate::tab_posts::MessageQueue as TabMessageQueue;
use crate::widgets::checkmark;
use const_format::formatcp as fmt;
use db::edit_details::EditDetails;
use db::Database;
use db::Post;
use db::PostId;
use db::PublishedState;
use db::Species;
use db::TaxonomicRank;
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
    CopyText,
    CopyPath,
    Cancel,
}

impl Message {
    pub const fn name(&self) -> &str {
        match self {
            Self::Publish => "mark post as published and close window",
            Self::CopyText => "copy text to clipboard",
            Self::CopyPath => "copy path of photo",
            Self::Cancel => "cancel publishing",
        }
    }
}

impl ModalPublish {
    pub fn new(id: PostId, db: &Database) -> Self {
        let post = db.post(&id);

        let text = render_text(post, db);
        let full_path = post.file.full_path.display().to_string();

        let entries = vec![
            Entry {
                copied: false,
                label: "Copy text".to_owned(),
                text,
            },
            Entry {
                copied: false,
                label: full_path.clone(),
                text: full_path,
            },
        ];

        Self {
            id,
            entries,
            queue: MessageQueue::new(),
            keyboard_mapping: Self::create_mapping(),
        }
    }

    fn create_mapping() -> KeyboardMapping {
        fn msg(msg: Message) -> MainMessage {
            MainMessage::TabPosts(TabMessage::ModalPublish(msg))
        }

        KeyboardMapping::default()
            .key(Key::Escape, msg(Message::Cancel))
            .key(Key::Num1, msg(Message::CopyText))
            .key(Key::Num2, msg(Message::CopyPath))
            .ctrl(Key::S, msg(Message::Publish))
            .ctrl(Key::P, msg(Message::Publish))
    }

    fn handle_message(&mut self, msg: Message, tab_queue: &mut TabMessageQueue) {
        match msg {
            Message::Cancel => {
                tab_queue.push_back(TabMessage::CloseModal);
            }
            Message::CopyText => {
                if let Some(entry) = self.entries.get_mut(0) {
                    entry.copied = true;
                    tab_queue.push_back(ClipboardValue::generic(entry.text.clone()).into());
                }
            }
            Message::CopyPath => {
                if let Some(entry) = self.entries.get_mut(1) {
                    entry.copied = true;
                    tab_queue.push_back(ClipboardValue::generic(entry.text.clone()).into());
                }
            }
            Message::Publish => {
                tab_queue.push_back(
                    EditDetails::SetPublished(self.id, PublishedState::timestamp_now()).into(),
                );
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
                        add_image(
                            ui,
                            &post.file,
                            image_cache,
                            style.image.preview_width,
                            style.image.radius,
                        );
                    });
            });

        TopBottomPanel::bottom(fmt!("{ID_PREFIX}-buttons")).show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                let button = Button::new(fmt!("{ICON_PUBLISH} Mark as published"))
                    .fill(style.button.publish);

                if ui.add(button).clicked() {
                    self.queue.push_back(Message::Publish);
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
                                    tab_queue.push_back(
                                        ClipboardValue::generic(entry.text.clone()).into(),
                                    );
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

        let has_pl = !species.pl.is_empty();
        let has_en = !species.en.is_empty();

        f.writeln(match (has_pl, has_en) {
            (false, false) => latin.as_str().to_owned(),
            (true, false) => format!("{latin} ({PL_EMOJI} {})", format_pl_species(species)),
            (false, true) => format!("{latin} ({EN_EMOJI} {})", format_en_species(species)),
            (true, true) => format!(
                "{latin} ({PL_EMOJI} {} {EN_EMOJI} {})",
                format_pl_species(species),
                format_en_species(species)
            ),
        });
    }

    if post.species.is_some() && f.is_empty() {
        let latin = post.species.as_ref().unwrap();
        let species = db.species_by_latin(latin).unwrap();

        if !species.pl.is_empty() {
            f.writeln(format!(
                "{PL_EMOJI} {} ({})",
                format_pl_species(species),
                species.latin
            ));
            if !species.en.is_empty() {
                f.writeln(format!(
                    "{EN_EMOJI} {} ({})",
                    format_en_species(species),
                    species.latin
                ));
            }
        } else if !species.en.is_empty() {
            f.writeln(format!(
                "{EN_EMOJI} {} ({})",
                format_en_species(species),
                species.latin.as_str()
            ));
        } else {
            if species.taxonomic_rank == TaxonomicRank::Species {
                f.writeln(species.latin.as_str().to_string());
            } else {
                f.writeln(format!(
                    "{PL_EMOJI} {} {}",
                    species.taxonomic_rank.pl_name(),
                    species.latin.as_str()
                ));
                f.writeln(format!(
                    "{EN_EMOJI} {} {}",
                    species.taxonomic_rank.en_name(),
                    species.latin.as_str()
                ));
            }
        }
    }

    if !f.is_empty() {
        f.newline()
    }

    let mut first = true;
    for tag in post.tags.iter() {
        if !first {
            f.put_char(' ');
        }

        first = false;
        f.write(format!("#{tag}"));
    }

    f.buf
}

fn format_pl_species(species: &Species) -> String {
    if species.taxonomic_rank == TaxonomicRank::Species {
        species.pl.clone()
    } else {
        format!("{} {}", species.taxonomic_rank.pl_name(), species.pl)
    }
}

fn format_en_species(species: &Species) -> String {
    if species.taxonomic_rank == TaxonomicRank::Species {
        species.en.clone()
    } else {
        format!("{} {}", species.taxonomic_rank.en_name(), species.en)
    }
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

    fn put_char(&mut self, c: char) {
        self.buf.push(c);
    }

    fn newline(&mut self) {
        self.buf.push('\n');
    }

    fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}
