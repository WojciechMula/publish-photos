use crate::style::Style;
use crate::widgets::Label as CustomLabel;
use const_format::formatcp as fmt;
use egui::Button;
use egui::Color32;
use egui::Frame;
use egui::Image;
use egui::ImageSource;
use egui::Rect;
use egui::Response;
use egui::Sense;
use egui::Ui;
use egui::UiBuilder;
use egui::Vec2;
use egui::Widget;

use egui_material_icons::icons::ICON_CANCEL;
use egui_material_icons::icons::ICON_CHECK;
use egui_material_icons::icons::ICON_CONTENT_COPY;
use egui_material_icons::icons::ICON_EDIT;

// --------------------------------------------------

pub fn frame<R>(
    ui: &mut Ui,
    fill: Option<Color32>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Response {
    ui.scope_builder(UiBuilder::new().sense(Sense::CLICK | Sense::HOVER), |ui| {
        let mut frame = Frame::new().inner_margin(4);
        if let Some(fill) = fill {
            frame.fill = fill
        }

        frame.show(ui, |ui| {
            let result = add_contents(ui);
            ui.set_min_width(ui.available_width());

            result
        })
    })
    .response
}

// --------------------------------------------------

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum OverlayLocation {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub fn add_overlay(
    ui: &mut Ui,
    resp: Response,
    location: OverlayLocation,
    size: Vec2,
    margin: Vec2,
    widget: impl Widget,
) -> Response {
    let min_x = resp.rect.min.x + margin.x;
    let max_x = resp.rect.max.x - margin.x;

    let min_y = resp.rect.min.y + margin.y;
    let max_y = resp.rect.max.y - margin.y;

    let pos = match location {
        OverlayLocation::TopLeft => egui::pos2(min_x, min_y),
        OverlayLocation::TopRight => egui::pos2(max_x - size.x, min_y),
        OverlayLocation::BottomLeft => egui::pos2(min_x, min_y - size.y),
        OverlayLocation::BottomRight => egui::pos2(max_x - size.x, max_y - size.y),
    };

    let rect = Rect::from_min_size(pos, size);

    ui.place(rect, widget)
}

pub fn widget_size(ui: &mut Ui, widget: impl Widget) -> Vec2 {
    let resp = ui.add(widget);

    resp.rect.size()
}

// --------------------------------------------------

pub fn add_image(ui: &mut Ui, uri: String, size: f32, radius: f32) -> Response {
    ui.add(
        Image::from_uri(uri)
            .maintain_aspect_ratio(true)
            .fit_to_exact_size(Vec2::splat(size))
            .show_loading_spinner(false)
            .corner_radius(radius),
    )
}

pub fn add_image_with_tint(
    ui: &mut Ui,
    uri: String,
    size: f32,
    radius: f32,
    tint: Color32,
) -> Response {
    ui.add(
        Image::from_uri(uri)
            .maintain_aspect_ratio(true)
            .fit_to_exact_size(Vec2::splat(size))
            .show_loading_spinner(false)
            .tint(tint)
            .corner_radius(radius),
    )
}

// --------------------------------------------------

const PL_ICON: ImageSource = egui::include_image!("../assets/pl.png");
const EN_ICON: ImageSource = egui::include_image!("../assets/en.png");
const PL_ID: u8 = 0;
const EN_ID: u8 = 1;

fn icon_aux(ui: &mut Ui, id: u8) {
    let icon = Vec2::splat(ui.style().spacing.interact_size.y);
    let img = Image::new(match id {
        PL_ID => PL_ICON,
        EN_ID => EN_ICON,
        _ => unreachable!(),
    })
    .show_loading_spinner(false);

    ui.add_sized(icon, img);
}

pub fn icon_pl(ui: &mut Ui) {
    icon_aux(ui, PL_ID);
}

pub fn icon_en(ui: &mut Ui) {
    icon_aux(ui, EN_ID);
}

// --------------------------------------------------

pub fn tag(tag: &str, style: &Style) -> impl Widget {
    let mut widget = CustomLabel::new(tag.to_owned());
    widget.padding = 3.0;
    widget.rounding = 6.0;
    widget.color = style.tag_active_fg;
    widget.background = style.tag_active_bg;

    widget
}

// --------------------------------------------------

pub mod button {
    use super::*;

    pub fn save(ui: &mut Ui, enabled: bool, background: Option<Color32>) -> bool {
        let label = fmt!("{ICON_CHECK} Save");
        let button = if let Some(background) = background {
            Button::new(label).fill(background)
        } else {
            Button::new(label)
        };

        ui.add_enabled(enabled, button).clicked()
    }

    pub fn cancel(ui: &mut Ui) -> bool {
        let label = fmt!("{ICON_CANCEL} Cancel");

        ui.button(label).clicked()
    }

    pub fn close(ui: &mut Ui) -> bool {
        ui.button("Close").clicked()
    }

    pub fn copy(ui: &mut Ui, enabled: bool) -> bool {
        ui.add_enabled(enabled, Button::new(ICON_CONTENT_COPY))
            .clicked()
    }

    pub fn edit(ui: &mut Ui) -> bool {
        ui.button(ICON_EDIT).clicked()
    }
}
