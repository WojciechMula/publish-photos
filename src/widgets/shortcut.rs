use egui::epaint;
use egui::Color32;
use egui::FontSelection;
use egui::Galley;
use egui::Key;
use egui::Modifiers;
use egui::Pos2;
use egui::Rect;
use egui::Response;
use egui::Sense;
use egui::Stroke;
use egui::StrokeKind;
use egui::Ui;
use egui::Vec2;
use egui::Widget;
use egui::WidgetInfo;
use egui::WidgetText;
use egui::WidgetType;
use std::sync::Arc;

use egui_material_icons::icons::ICON_ARROW_BACK;
use egui_material_icons::icons::ICON_ARROW_DOWNWARD;
use egui_material_icons::icons::ICON_ARROW_FORWARD;
use egui_material_icons::icons::ICON_ARROW_UPWARD;

pub struct Shortcut {
    modifiers: Modifiers,
    key: Key,
    pub padding: f32,
    pub rounding: f32,
    pub color: Color32,
    pub stroke_width: f32,
}

impl Shortcut {
    pub fn from_key(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers::NONE,
            padding: 3.0,
            rounding: 6.0,
            color: Color32::WHITE,
            stroke_width: 0.5,
        }
    }

    pub fn from_key_and_modifiers(key: Key, modifiers: Modifiers) -> Self {
        Self {
            modifiers,
            ..Self::from_key(key)
        }
    }

    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    fn parts(&self) -> Vec<WidgetText> {
        let mut result = Vec::<WidgetText>::with_capacity(5);

        if self.modifiers.command | self.modifiers.ctrl {
            result.push(WidgetText::Text("Ctrl".to_owned()));
        }

        if self.modifiers.alt {
            result.push(WidgetText::Text("Alt".to_owned()));
        }

        if self.modifiers.shift {
            result.push(WidgetText::Text("Shift".to_owned()));
        }

        let key_name = match self.key {
            Key::ArrowUp => ICON_ARROW_UPWARD,
            Key::ArrowDown => ICON_ARROW_DOWNWARD,
            Key::ArrowLeft => ICON_ARROW_BACK,
            Key::ArrowRight => ICON_ARROW_FORWARD,
            _ => self.key.name(),
        };

        result.push(WidgetText::Text(key_name.to_string()));

        result
    }

    fn text(&self) -> String {
        let mut result = String::new();

        if self.modifiers.command | self.modifiers.ctrl {
            result += "Ctrl";
        }

        if self.modifiers.alt {
            if !result.is_empty() {
                result += "-";
            }
            result += "Alt";
        }

        if self.modifiers.shift {
            if !result.is_empty() {
                result += "-";
            }
            result += "Shift";
        }

        if !result.is_empty() {
            result += "-";
        }
        result += self.key.name();

        result
    }

    fn render_text(wt: WidgetText, ui: &Ui) -> Arc<Galley> {
        let layout_job = Arc::unwrap_or_clone(wt.into_layout_job(
            ui.style(),
            FontSelection::Default,
            ui.text_valign(),
        ));

        ui.fonts_mut(|fonts| fonts.layout_job(layout_job))
    }
}

impl Widget for Shortcut {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut rendered = Vec::<Arc<Galley>>::new();
        for part in self.parts() {
            let galley = Self::render_text(part, ui);
            rendered.push(galley);
        }

        let separator = Self::render_text(WidgetText::Text("-".to_owned()), ui);

        let mut width = 0.0;
        let mut max_height = 0.0;
        for galley in &rendered {
            width += galley.size().x + 2.0 * self.padding;

            let height = galley.size().y + 2.0 * self.padding;
            if height > max_height {
                max_height = height;
            }
        }

        width += (separator.size().x + 2.0 * self.padding) * (rendered.len() - 1) as f32;

        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, max_height), Sense::empty());

        response
            .widget_info(|| WidgetInfo::labeled(WidgetType::Label, ui.is_enabled(), self.text()));

        if ui.is_rect_visible(response.rect) {
            let mut x = rect.left_top().x;
            let y = rect.left_top().y;
            let mut needs_separator = false;
            for galley in rendered {
                if needs_separator {
                    x += self.padding;
                    ui.painter().add(epaint::TextShape::new(
                        Pos2::new(x, y + self.padding),
                        separator.clone(),
                        self.color,
                    ));

                    x += separator.size().x + self.padding;
                }
                needs_separator = true;

                let size = galley.size();
                let rect = Rect::from_min_size(
                    Pos2::new(x, y),
                    Vec2::new(size.x + 2.0 * self.padding, size.y + 2.0 * self.padding),
                );

                ui.painter().add(epaint::RectShape::stroke(
                    rect,
                    self.rounding,
                    Stroke {
                        width: self.stroke_width,
                        color: self.color,
                    },
                    StrokeKind::Inside,
                ));

                ui.painter().add(epaint::TextShape::new(
                    Pos2::new(x + self.padding, y + self.padding),
                    galley,
                    self.color,
                ));

                x += rect.width();
            }
        }

        response
    }
}
