use egui::Color32;
use egui::FontId;
use egui::Response;
use egui::Sense;
use egui::Ui;
use egui::Widget;

use egui_material_icons::icons::ICON_CHECK;

fn checkmark_ui(ui: &mut Ui, flag: bool, color: Color32) -> Response {
    let galley = ui
        .painter()
        .layout_no_wrap(ICON_CHECK.to_owned(), FontId::default(), color);
    let text_size = galley.size();
    let (rect, mut response) = ui.allocate_exact_size(text_size, Sense::empty());
    response.intrinsic_size = Some(galley.intrinsic_size());

    if flag && ui.is_rect_visible(response.rect) {
        ui.painter().galley(rect.min, galley, color);
    }

    response
}

pub fn checkmark(flag: bool, color: Color32) -> impl Widget + 'static {
    move |ui: &mut egui::Ui| checkmark_ui(ui, flag, color)
}
