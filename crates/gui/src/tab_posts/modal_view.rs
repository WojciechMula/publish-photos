mod cursor;

use cursor::Cursor;

use crate::application::Message as MainMessage;
use crate::keyboard::KeyboardMapping;
use crate::tab_posts::ImageCache;
use crate::tab_posts::Message as TabMessage;
use crate::tab_posts::MessageQueue as TabMessageQueue;
use db::Database;
use db::PostId;
use egui::Align;
use egui::CentralPanel;
use egui::Context;
use egui::Image;
use egui::Key;
use egui::Layout;
use std::collections::VecDeque;

pub struct ModalView {
    post_id: PostId,
    cursor: Cursor,
    initialized: bool,

    pub queue: MessageQueue,
    pub keyboard_mapping: KeyboardMapping,
}

type MessageQueue = VecDeque<Message>;

#[derive(Clone)]
pub enum Message {
    Close,
    SelectFirst,
    SelectLast,
    SelectNext,
    SelectPrev,
    SelectPhoto1,
    SelectPhoto2,
    SelectPhoto3,
    SelectPhoto4,
    SelectPhoto5,
    SelectPhoto6,
    SelectPhoto7,
    SelectPhoto8,
    SelectPhoto9,
}

impl Message {
    pub const fn name(&self) -> &str {
        match self {
            Self::Close => "close window",
            Self::SelectFirst => "go to the first photo",
            Self::SelectLast => "go to the last photo",
            Self::SelectNext => "switch to the next photo",
            Self::SelectPrev => "switch to the previous photo",
            Self::SelectPhoto1 => "select photo #1",
            Self::SelectPhoto2 => "select photo #2",
            Self::SelectPhoto3 => "select photo #3",
            Self::SelectPhoto4 => "select photo #4",
            Self::SelectPhoto5 => "select photo #5",
            Self::SelectPhoto6 => "select photo #6",
            Self::SelectPhoto7 => "select photo #7",
            Self::SelectPhoto8 => "select photo #8",
            Self::SelectPhoto9 => "select photo #9",
        }
    }
}

impl ModalView {
    pub fn new(id: PostId, db: &Database) -> Self {
        let post = db.post(&id);

        Self {
            initialized: false,
            queue: MessageQueue::new(),
            post_id: id,
            cursor: Cursor::new(post.files.len()),
            keyboard_mapping: Self::create_mapping(post.files.len()),
        }
    }

    fn create_mapping(photos_count: usize) -> KeyboardMapping {
        fn msg(msg: Message) -> MainMessage {
            MainMessage::TabPosts(TabMessage::ModalView(msg))
        }

        let mut km = KeyboardMapping::default()
            .key(Key::Escape, msg(Message::Close))
            .key(Key::Space, msg(Message::Close))
            .key(Key::F, msg(Message::Close))
            .key(Key::V, msg(Message::Close));

        if photos_count > 1 {
            km = km
                .key(Key::ArrowUp, msg(Message::SelectPrev))
                .key(Key::ArrowLeft, msg(Message::SelectPrev))
                .key(Key::ArrowDown, msg(Message::SelectNext))
                .key(Key::ArrowRight, msg(Message::SelectNext))
                .key(Key::Home, msg(Message::SelectFirst))
                .key(Key::End, msg(Message::SelectLast))
                .key(Key::Num1, msg(Message::SelectPhoto1))
                .key(Key::Num2, msg(Message::SelectPhoto2));
        }

        if photos_count >= 3 {
            km = km.key(Key::Num3, msg(Message::SelectPhoto3));
        }
        if photos_count >= 4 {
            km = km.key(Key::Num4, msg(Message::SelectPhoto4));
        }
        if photos_count >= 5 {
            km = km.key(Key::Num5, msg(Message::SelectPhoto5));
        }
        if photos_count >= 6 {
            km = km.key(Key::Num6, msg(Message::SelectPhoto6));
        }
        if photos_count >= 7 {
            km = km.key(Key::Num7, msg(Message::SelectPhoto7));
        }
        if photos_count >= 8 {
            km = km.key(Key::Num8, msg(Message::SelectPhoto8));
        }
        if photos_count >= 9 {
            km = km.key(Key::Num9, msg(Message::SelectPhoto9));
        }

        km
    }

    fn handle_message(&mut self, msg: Message, tab_queue: &mut TabMessageQueue) {
        match msg {
            Message::Close => {
                tab_queue.push_back(TabMessage::CloseModal);
            }
            Message::SelectFirst => {
                self.cursor.first();
            }
            Message::SelectLast => {
                self.cursor.last();
            }
            Message::SelectPrev => {
                self.cursor.prev();
            }
            Message::SelectNext => {
                self.cursor.next();
            }
            Message::SelectPhoto1 => {
                self.cursor.set_current(0);
            }
            Message::SelectPhoto2 => {
                self.cursor.set_current(1);
            }
            Message::SelectPhoto3 => {
                self.cursor.set_current(2);
            }
            Message::SelectPhoto4 => {
                self.cursor.set_current(3);
            }
            Message::SelectPhoto5 => {
                self.cursor.set_current(4);
            }
            Message::SelectPhoto6 => {
                self.cursor.set_current(5);
            }
            Message::SelectPhoto7 => {
                self.cursor.set_current(6);
            }
            Message::SelectPhoto8 => {
                self.cursor.set_current(7);
            }
            Message::SelectPhoto9 => {
                self.cursor.set_current(8);
            }
        }
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        image_cache: &mut ImageCache,
        db: &Database,
        tab_queue: &mut TabMessageQueue,
    ) {
        let Some(current) = self.cursor.current() else {
            return;
        };

        let post = db.post(&self.post_id);

        if !self.initialized {
            for item in &post.files {
                image_cache.request(item.uri.clone());
            }
            self.initialized = true;
        }

        while let Some(msg) = self.queue.pop_front() {
            self.handle_message(msg, tab_queue);
        }

        let n = post.files.len();

        CentralPanel::default().show(ctx, |ui| {
            if n == 1 {
                ui.centered_and_justified(|ui| {
                    ui.add(
                        Image::from_uri(post.files[current].uri.clone())
                            .maintain_aspect_ratio(true)
                            .fit_to_exact_size(ui.available_size())
                            .show_loading_spinner(false),
                    );
                });
            } else {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.label(format!("{} of {n}", current + 1));
                    ui.add(
                        Image::from_uri(post.files[current].uri.clone())
                            .maintain_aspect_ratio(true)
                            .fit_to_exact_size(ui.available_size())
                            .show_loading_spinner(false),
                    );
                });
            }
        });
    }

    pub fn try_close(&mut self) {
        self.queue.push_back(Message::Close);
    }
}
