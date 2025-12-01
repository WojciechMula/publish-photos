use crate::image_cache::ImageCache;
use crate::style::Style;
use crate::widgets::Label as CustomLabel;
use const_format::formatcp as fmt;
use db::FileMetadata;
use egui::Align;
use egui::Button;
use egui::Color32;
use egui::FontId;
use egui::Frame;
use egui::Image;
use egui::ImageSource;
use egui::Label;
use egui::Layout;
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

pub fn tight_frame<R>(
    ui: &mut Ui,
    fill: Option<Color32>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Response {
    ui.scope_builder(UiBuilder::new().sense(Sense::CLICK | Sense::HOVER), |ui| {
        let mut frame = Frame::new().inner_margin(4);
        if let Some(fill) = fill {
            frame.fill = fill
        }

        frame.show(ui, |ui| add_contents(ui))
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
    resp: &Response,
    location: OverlayLocation,
    margin: Vec2,
    contents: impl FnOnce(&mut Ui) -> Response,
) -> Response {
    let rect = resp.rect.shrink2(margin);

    let layout = match location {
        OverlayLocation::TopLeft => Layout::left_to_right(Align::Min),
        OverlayLocation::TopRight => Layout::right_to_left(Align::Min),
        OverlayLocation::BottomLeft => Layout::left_to_right(Align::Max),
        OverlayLocation::BottomRight => Layout::right_to_left(Align::Max),
    };

    let mut ui = ui.new_child(UiBuilder::new().max_rect(rect).layout(layout));

    contents(&mut ui)
}

// --------------------------------------------------

pub fn add_image(
    ui: &mut Ui,
    meta: &FileMetadata,
    image_cache: &mut ImageCache,
    width: f32,
    radius: f32,
) -> Response {
    let ratio = if let Some(image_size) = meta.image_size {
        let w = image_size.width as f32;
        let h = image_size.height as f32;
        if image_size.width > image_size.height {
            h / w
        } else {
            w / h
        }
    } else {
        1.0 / 3.0
    };

    let height = width * ratio;
    let size = Vec2::new(width, height);

    if image_cache.is_cached(&meta.uri) {
        ui.add_sized(
            size,
            Image::from_uri(meta.uri.clone())
                .maintain_aspect_ratio(true)
                .fit_to_exact_size(Vec2::new(width, height))
                .show_loading_spinner(false)
                .corner_radius(radius),
        )
    } else {
        let resp = ui.add_sized(size, Label::new(&meta.uri).truncate());
        if ui.is_rect_visible(resp.rect) {
            image_cache.request(meta.uri.clone());
        }

        resp
    }
}

pub fn add_image_with_tint(
    ui: &mut Ui,
    meta: &FileMetadata,
    image_cache: &mut ImageCache,
    size: f32,
    radius: f32,
    tint: Color32,
) -> Response {
    if image_cache.is_cached(&meta.uri) {
        ui.add(
            Image::from_uri(meta.uri.clone())
                .maintain_aspect_ratio(true)
                .fit_to_exact_size(Vec2::splat(size))
                .show_loading_spinner(false)
                .tint(tint)
                .corner_radius(radius),
        )
    } else {
        let resp = ui.add_sized(Vec2::splat(size), Label::new(&meta.uri));
        if ui.is_rect_visible(resp.rect) {
            image_cache.request(meta.uri.clone());
        }

        resp
    }
}

// --------------------------------------------------

const PL_ICON: ImageSource = egui::include_image!("../../../assets/pl.png");
const EN_ICON: ImageSource = egui::include_image!("../../../assets/en.png");
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

pub fn text_size(s: &str, ui: &mut Ui) -> Vec2 {
    let galley = ui
        .painter()
        .layout_no_wrap(s.to_owned(), FontId::default(), Color32::TRANSPARENT);

    galley.size()
}

pub fn max_size(strings: &[&str], ui: &mut Ui) -> f32 {
    let mut max_size = 0.0;
    for s in strings {
        let size = text_size(s, ui).x;
        if size > max_size {
            max_size = size;
        }
    }

    max_size
}

// --------------------------------------------------

pub fn overlay_label(label: String, style: &Style) -> impl Widget {
    use crate::widgets::Label;

    let mut widget = Label::new(label);
    widget.padding = 3.0;
    widget.rounding = 5.0;
    widget.color = style.image.overlay.fg;
    widget.background = style.image.overlay.bg;

    widget
}

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
