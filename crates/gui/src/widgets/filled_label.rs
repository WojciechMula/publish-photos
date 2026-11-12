use egui::Color32;
use egui::FontId;
use egui::Response;
use egui::Sense;
use egui::Ui;
use egui::Vec2;
use egui::WidgetInfo;
use egui::WidgetType;

fn filled_label_ui(
    ui: &mut Ui,
    s: &str,
    fg: Color32,
    bg: Color32,
    padding: f32,
    rounding: f32,
) -> Response {
    let galley = ui
        .painter()
        .layout_no_wrap(s.to_string(), FontId::default(), fg);
    let text_size = galley.size();
    let text_size_with_padding =
        Vec2::new(text_size.x + 2.0 * padding, text_size.y + 2.0 * padding);

    let (rect, mut response) = ui.allocate_exact_size(text_size_with_padding, Sense::empty());
    response.intrinsic_size = Some(galley.intrinsic_size());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, ui.is_enabled(), s));

    if ui.is_rect_visible(response.rect) {
        ui.painter().rect_filled(rect, rounding, bg);

        let pos = rect.translate(Vec2::splat(padding)).left_top();
        ui.painter().galley(pos, galley, fg);
    }

    response
}

pub fn filled_label<'a>(
    text: &'a str,
    fg: Color32,
    bg: Color32,
    padding: f32,
    rounding: f32,
) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| filled_label_ui(ui, text, fg, bg, padding, rounding)
}
