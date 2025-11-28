use egui::Color32;
use egui::Vec2;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Style {
    pub modified: Color32,
    pub hovered_frame: Color32,
    pub selected_post: Color32,
    pub published_post: Color32,

    pub tag_active_bg: Color32,
    pub tag_active_fg: Color32,
    pub tag_hovered_bg: Color32,
    pub tag_hovered_fg: Color32,
    pub tag_highlight_fg: Color32,

    pub image: ImageStyle,
    pub button: ButtonStyle,

    pub copied_mark: Color32,
    pub error: Color32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ImageStyle {
    pub radius: f32,
    pub preview_width: f32,
    pub thumbnail_width: f32,
    pub overlay: OverlayStyle,
    pub inactive: Color32,
}

impl PartialEq for ImageStyle {
    fn eq(&self, other: &Self) -> bool {
        self.radius == other.radius
            && self.preview_width == other.preview_width
            && self.thumbnail_width == other.thumbnail_width
            && self.overlay == other.overlay
            && self.inactive == other.inactive
    }
}

impl Eq for ImageStyle {}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ButtonStyle {
    pub save: Color32,
    pub remove: Color32,
    pub discard: Color32,
    pub publish: Color32,
    pub save_database: Color32,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OverlayStyle {
    pub margin: Vec2,
    pub fg: Color32,
    pub bg: Color32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            modified: crate::colors::GHOST_WHITE,
            tag_active_bg: crate::colors::LIGHT_SKY_BLUE,
            tag_active_fg: Color32::BLACK,
            tag_hovered_bg: crate::colors::LIGHT_STEEL_BLUE,
            tag_hovered_fg: Color32::BLACK,
            tag_highlight_fg: crate::colors::RED1,
            hovered_frame: crate::colors::DARK_SLATE_GRAY,
            selected_post: crate::colors::DARK_SLATE_BLUE,
            published_post: crate::colors::GRAY30,
            image: ImageStyle::default(),
            button: ButtonStyle::default(),
            copied_mark: Color32::GREEN,
            error: Color32::RED,
        }
    }
}

impl Default for ImageStyle {
    fn default() -> Self {
        Self {
            radius: 5.0,
            preview_width: 400.0,
            thumbnail_width: 200.0,
            overlay: OverlayStyle::default(),
            inactive: Color32::from_gray(128),
        }
    }
}

impl Default for OverlayStyle {
    fn default() -> Self {
        Self {
            margin: Vec2::splat(6.0),
            fg: Color32::BLACK,
            bg: Color32::WHITE,
        }
    }
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            publish: Color32::DARK_GREEN,
            save: Color32::DARK_GREEN,
            remove: Color32::RED,
            discard: Color32::RED,
            save_database: Color32::RED,
        }
    }
}
