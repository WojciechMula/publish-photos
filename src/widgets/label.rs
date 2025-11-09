use egui::epaint;
use egui::Color32;
use egui::FontSelection;
use egui::Galley;
use egui::Rect;
use egui::Response;
use egui::Sense;
use egui::Ui;
use egui::Vec2;
use egui::Widget;
use egui::WidgetInfo;
use egui::WidgetText;
use egui::WidgetType;
use std::sync::Arc;

pub struct Label {
    text: WidgetText,
    pub padding: f32,
    pub rounding: f32,
    pub color: Color32,
    pub background: Color32,
}

impl Label {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            padding: 0.0,
            rounding: 0.0,
            color: Color32::TRANSPARENT,
            background: Color32::TRANSPARENT,
        }
    }
}

impl Label {
    pub fn layout_in_ui(&self, ui: &mut Ui) -> (Rect, Arc<Galley>, Response) {
        let layout_job = Arc::unwrap_or_clone(self.text.clone().into_layout_job(
            ui.style(),
            FontSelection::Default,
            ui.text_valign(),
        ));

        let galley = ui.fonts_mut(|fonts| fonts.layout_job(layout_job));
        let size = galley.size();
        let size_padding = Vec2::new(size.x + 2.0 * self.padding, size.y + 2.0 * self.padding);
        let (rect, mut response) = ui.allocate_exact_size(size_padding, Sense::empty());
        response.intrinsic_size = Some(galley.intrinsic_size());
        (rect, galley, response)
    }
}

impl Widget for Label {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, galley, response) = self.layout_in_ui(ui);
        response
            .widget_info(|| WidgetInfo::labeled(WidgetType::Label, ui.is_enabled(), galley.text()));

        if ui.is_rect_visible(response.rect) {
            ui.painter().add(epaint::RectShape::filled(
                rect,
                self.rounding,
                self.background,
            ));

            let pos = rect.translate(Vec2::splat(self.padding)).left_top();

            ui.painter()
                .add(epaint::TextShape::new(pos, galley, self.color));
        }

        response
    }
}
