use egui::epaint;
use egui::Color32;
use egui::Response;
use egui::Sense;
use egui::Stroke;
use egui::StrokeKind;
use egui::Ui;
use egui::Vec2;
use egui::Widget;

pub struct ColorBox {
    pub width: usize,
    pub color: Color32,
}

impl ColorBox {
    pub fn new(color: Color32, width: usize) -> Self {
        Self { color, width }
    }
}

impl Widget for ColorBox {
    fn ui(self, ui: &mut Ui) -> Response {
        let height = ui.spacing().interact_size.y;
        let width = self.width as f32 * height;

        let size = Vec2::new(width, height);
        let (rect, response) = ui.allocate_exact_size(size, Sense::empty());

        if ui.is_rect_visible(response.rect) {
            let rounding = 0.0;
            let stroke_width = 0.5;
            let border_color = Color32::BLACK;

            ui.painter()
                .add(epaint::RectShape::filled(rect, rounding, self.color));

            ui.painter().add(epaint::RectShape::stroke(
                rect,
                rounding,
                Stroke {
                    width: stroke_width,
                    color: border_color,
                },
                StrokeKind::Inside,
            ));
        }

        response
    }
}
