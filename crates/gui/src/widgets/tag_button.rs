use crate::style::Style;
use egui::text::LayoutJob;
use egui::Color32;
use egui::Response;
use egui::Sense;
use egui::StrokeKind;
use egui::TextFormat;
use egui::Ui;
use egui::Vec2;

fn tag_button_ui(ui: &mut Ui, text: &str, needle: &str, style: &Style) -> Response {
    let mut job = LayoutJob::default();

    let leading_space = 0.0;
    let rounding = 3.0;

    let normal_style = TextFormat {
        color: style.tag_active_fg,
        ..TextFormat::default()
    };

    let highlight_style = TextFormat {
        color: style.tag_highlight_fg,
        ..TextFormat::default()
    };

    let mut s = text;
    while !needle.is_empty() && !s.is_empty() {
        let Some((prefix, rest)) = s.split_once(needle) else {
            break;
        };

        if !prefix.is_empty() {
            job.append(prefix, leading_space, normal_style.clone());
        }

        job.append(needle, leading_space, highlight_style.clone());
        s = rest;
    }

    if !s.is_empty() {
        job.append(s, leading_space, normal_style.clone());
    }

    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let size = galley.size();
    let padding = ui.style().spacing.button_padding;

    let size_padding = Vec2::new(size.x + 2.0 * padding.x, size.y + 2.0 * padding.y);
    let (rect, mut response) = ui.allocate_exact_size(size_padding, Sense::HOVER | Sense::CLICK);
    response.intrinsic_size = Some(galley.intrinsic_size());

    if ui.is_rect_visible(response.rect) {
        let pos = rect.translate(padding).left_top();
        let (bg_stroke, background) = if response.hovered() {
            (ui.visuals().widgets.hovered.bg_stroke, style.tag_hovered_bg)
        } else {
            (ui.visuals().widgets.active.bg_stroke, style.tag_active_bg)
        };

        ui.painter().rect_filled(rect, rounding, background);

        ui.painter()
            .rect_stroke(rect, rounding, bg_stroke, StrokeKind::Inside);
        ui.painter().galley(pos, galley, Color32::PLACEHOLDER);
    }

    response
}

pub fn tag_button<'a>(text: &'a str, needle: &'a str, style: &'a Style) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| tag_button_ui(ui, text, needle, style)
}
