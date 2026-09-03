use egui::text::LayoutJob;
use egui::Color32;
use egui::Response;
use egui::Sense;
use egui::StrokeKind;
use egui::TextFormat;
use egui::Ui;
use egui::Vec2;

fn label_button_ui(ui: &mut Ui, text: &str, fg: Color32, bg: Color32) -> Response {
    let mut job = LayoutJob::default();

    let leading_space = 0.0;
    let rounding = 3.0;

    let normal_style = TextFormat {
        color: fg,
        ..TextFormat::default()
    };

    job.append(text, leading_space, normal_style.clone());

    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let size = galley.size();
    let padding = ui.style().spacing.button_padding;

    let size_padding = Vec2::new(size.x + 2.0 * padding.x, size.y + 2.0 * padding.y);
    let sense = Sense::empty();

    let (rect, mut response) = ui.allocate_exact_size(size_padding, sense);
    response.intrinsic_size = Some(galley.intrinsic_size());

    if ui.is_rect_visible(response.rect) {
        let pos = rect.translate(padding).left_top();

        let mut bg_stroke = ui.visuals().widgets.active.bg_stroke;
        bg_stroke.color = bg;

        ui.painter().rect_filled(rect, rounding, bg);
        ui.painter()
            .rect_stroke(rect, rounding, bg_stroke, StrokeKind::Inside);
        ui.painter().galley(pos, galley, fg);
    }

    response
}

pub fn label_button<'a>(text: &'a str, bg: Color32, fg: Color32) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| label_button_ui(ui, text, bg, fg)
}
