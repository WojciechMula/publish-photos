use eframe::emath::OrderedFloat;
use egui::Context;
use egui::SizeHint;
use std::collections::HashSet;

#[derive(Default)]
pub struct ImageCache {
    pub loaded: HashSet<String>,
    pub requested: HashSet<String>,
}

impl ImageCache {
    pub fn is_cached(&self, uri: &String) -> bool {
        self.loaded.contains(uri)
    }

    pub fn request(&mut self, uri: String) {
        if !self.loaded.contains(&uri) {
            self.requested.insert(uri);
        }
    }

    pub fn load_requested(&mut self, ctx: &Context) {
        for uri in self.requested.drain() {
            let _ = ctx.try_load_image(&uri, SizeHint::Scale(OrderedFloat(1.0)));
            self.loaded.insert(uri);
        }
    }
}
