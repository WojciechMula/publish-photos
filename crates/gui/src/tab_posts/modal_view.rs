use crate::application::Message as MainMessage;
use crate::keyboard::KeyboardMapping;
use crate::tab_posts::ImageCache;
use crate::tab_posts::Message as TabMessage;
use crate::tab_posts::MessageQueue as TabMessageQueue;
use db::Database;
use db::PostId;
use egui::CentralPanel;
use egui::Context;
use egui::Image;
use egui::Key;
use std::collections::VecDeque;

pub struct ModalView {
    post_id: PostId,
    initialized: bool,

    pub queue: MessageQueue,
    pub keyboard_mapping: KeyboardMapping,
}

type MessageQueue = VecDeque<Message>;

#[derive(Clone)]
pub enum Message {
    Close,
}

impl Message {
    pub const fn name(&self) -> &str {
        match self {
            Self::Close => "close window",
        }
    }
}

impl ModalView {
    pub fn new(id: PostId) -> Self {
        Self {
            initialized: false,
            queue: MessageQueue::new(),
            post_id: id,
            keyboard_mapping: Self::create_mapping(),
        }
    }

    fn create_mapping() -> KeyboardMapping {
        fn msg(msg: Message) -> MainMessage {
            MainMessage::TabPosts(TabMessage::ModalView(msg))
        }

        KeyboardMapping::default()
            .key(Key::Escape, msg(Message::Close))
            .key(Key::Space, msg(Message::Close))
            .key(Key::F, msg(Message::Close))
            .key(Key::V, msg(Message::Close))
    }

    fn handle_message(&mut self, msg: Message, tab_queue: &mut TabMessageQueue) {
        match msg {
            Message::Close => {
                tab_queue.push_back(TabMessage::CloseModal);
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
        let post = db.post(&self.post_id);

        if !self.initialized {
            image_cache.request(post.file.uri.clone(), ctx);
            self.initialized = true;
        }

        while let Some(msg) = self.queue.pop_front() {
            self.handle_message(msg, tab_queue);
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.add(
                    Image::from_uri(post.file.uri.clone())
                        .maintain_aspect_ratio(true)
                        .fit_to_exact_size(ui.available_size())
                        .show_loading_spinner(false),
                );
            });
        });
    }

    pub fn try_close(&mut self) {
        self.queue.push_back(Message::Close);
    }
}
