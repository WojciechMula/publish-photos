use egui::Color32;
use egui::Response;
use egui::Sense;
use egui::Stroke;
use egui::StrokeKind;
use egui::Ui;
use egui::Vec2;
use egui::Widget;

fn color_box_ui(ui: &mut Ui, color: Color32, width: usize) -> Response {
    let height = ui.spacing().interact_size.y;
    let width = width as f32 * height;

    let size = Vec2::new(width, height);
    let (rect, response) = ui.allocate_exact_size(size, Sense::empty());

    if ui.is_rect_visible(response.rect) {
        let rounding = 0.0;
        let stroke_width = 0.5;
        let border_color = Color32::BLACK;

        ui.painter().rect_filled(rect, rounding, color);
        ui.painter().rect_stroke(
            rect,
            rounding,
            Stroke {
                width: stroke_width,
                color: border_color,
            },
            StrokeKind::Inside,
        );
    }

    response
}

pub fn color_box(color: Color32, width: usize) -> impl Widget + 'static {
    move |ui: &mut egui::Ui| color_box_ui(ui, color, width)
}
