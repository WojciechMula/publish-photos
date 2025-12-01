use egui::Color32;
use egui::FontId;
use egui::Response;
use egui::Sense;
use egui::Ui;
use egui::Vec2;
use egui::Widget;
use egui::WidgetInfo;
use egui::WidgetType;

pub struct Label {
    text: String,
    pub padding: f32,
    pub rounding: f32,
    pub color: Color32,
    pub background: Color32,
}

impl Label {
    pub fn new(text: String) -> Self {
        Self {
            text,
            padding: 0.0,
            rounding: 0.0,
            color: Color32::TRANSPARENT,
            background: Color32::TRANSPARENT,
        }
    }
}

impl Widget for Label {
    fn ui(self, ui: &mut Ui) -> Response {
        let galley = ui
            .painter()
            .layout_no_wrap(self.text.clone(), FontId::default(), self.color);
        let text_size = galley.size();
        let text_size_with_padding = Vec2::new(
            text_size.x + 2.0 * self.padding,
            text_size.y + 2.0 * self.padding,
        );

        let (rect, mut response) = ui.allocate_exact_size(text_size_with_padding, Sense::empty());
        response.intrinsic_size = Some(galley.intrinsic_size());
        response
            .widget_info(|| WidgetInfo::labeled(WidgetType::Label, ui.is_enabled(), galley.text()));

        if ui.is_rect_visible(response.rect) {
            ui.painter()
                .rect_filled(rect, self.rounding, self.background);

            let pos = rect.translate(Vec2::splat(self.padding)).left_top();
            ui.painter().galley(pos, galley, self.color);
        }

        response
    }
}
