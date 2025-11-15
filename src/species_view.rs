use crate::db::Database;
use crate::db::Species;
use crate::db::SpeciesId;
use crate::gui::add_image;
use crate::gui::frame;
use crate::gui::icon_en;
use crate::gui::icon_pl;
use crate::gui::overlay_label;
use crate::image_cache::ImageCache;
use crate::style::Style;
use crate::ImageCounter;
use egui::Align;
use egui::Label;
use egui::Layout;
use egui::RichText;
use egui::Ui;
use egui::UiBuilder;
use egui::Vec2;

use egui_material_icons::icons::ICON_ARROW_BACK_2;
use egui_material_icons::icons::ICON_PLAY_ARROW;

#[derive(Default)]
pub struct SpeciesList {
    ids: Option<Vec<SpeciesId>>,
    view: Vec<SpeciesId>,
    phrase: String,
    pub sort_order: SortOrder,
    pub image_width: f32,
    pub hovered: Option<SpeciesId>,
}

#[derive(PartialEq, Eq, Default, Clone, Copy)]
pub enum SortOrder {
    #[default]
    DateAddedAsc,
    DateAddedDesc,
    LatinAsc,
    LatinDesc,
    PolishAsc,
    PolishDesc,
    EnglishAsc,
    EnglishDesc,
}

impl SpeciesList {
    pub fn with_custom_list(mut self, ids: Vec<SpeciesId>) -> Self {
        self.ids = Some(ids);
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.image_width = width;
        self
    }

    pub fn refresh_view(&mut self, db: &Database) {
        self.view.clear();
        if let Some(ids) = &self.ids {
            for id in ids {
                let species = db.species_by_id(id).unwrap();
                if self.match_phrase(species) {
                    self.view.push(*id);
                }
            }
        } else {
            for species in db.species.iter() {
                if self.match_phrase(species) {
                    self.view.push(species.id);
                }
            }
        }

        self.sort(db);
    }

    pub fn is_empty(&self) -> bool {
        self.view.is_empty()
    }

    fn match_phrase(&self, species: &Species) -> bool {
        species.search_parts.matches(&self.phrase)
    }

    pub fn set_filter(&mut self, phrase: String, db: &Database) {
        if phrase == self.phrase {
            return;
        }

        self.phrase = phrase.trim().to_lowercase();
        self.refresh_view(db);
    }

    pub fn set_sort_order(&mut self, sort_order: SortOrder, db: &Database) {
        if self.sort_order == sort_order {
            return;
        }

        self.sort_order = sort_order;
        self.sort(db);
    }

    fn sort(&mut self, db: &Database) {
        match self.sort_order {
            SortOrder::DateAddedAsc | SortOrder::DateAddedDesc => {
                self.view.sort();
            }
            SortOrder::LatinAsc | SortOrder::LatinDesc => {
                self.view
                    .sort_by_key(|id| db.species_by_id(id).as_ref().unwrap().latin.as_str());
            }
            SortOrder::PolishAsc | SortOrder::PolishDesc => {
                self.view.sort_by_key(|id| {
                    let tmp = db.species_by_id(id);
                    let species = tmp.as_ref().unwrap();
                    if species.pl.is_empty() {
                        species.latin.as_str()
                    } else {
                        &species.pl
                    }
                });
            }
            SortOrder::EnglishAsc | SortOrder::EnglishDesc => {
                self.view.sort_by_key(|id| {
                    let tmp = db.species_by_id(id);
                    let species = tmp.as_ref().unwrap();
                    if species.en.is_empty() {
                        species.latin.as_str()
                    } else {
                        &species.en
                    }
                });
            }
        }

        if matches!(
            self.sort_order,
            SortOrder::DateAddedDesc
                | SortOrder::LatinDesc
                | SortOrder::PolishDesc
                | SortOrder::EnglishDesc
        ) {
            self.view.reverse();
        }
    }

    pub fn render(
        &self,
        ui: &mut Ui,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &Database,
    ) -> SpeciesListResponse {
        let mut result = SpeciesListResponse::default();

        let mut hovered: Option<SpeciesId> = None;
        for id in self.view.iter() {
            let species = &db.species[id.0];

            let fill = if Some(id) == self.hovered.as_ref() {
                Some(style.hovered_frame)
            } else {
                None
            };

            let resp = frame(ui, fill, |ui| {
                let action = block(ui, image_cache, style, db, species, self.image_width);
                if let Some(action) = action {
                    result.species_view_action = Some((action, *id));
                }
            });

            if resp.contains_pointer() {
                hovered = Some(*id);
            }
            if resp.clicked() {
                result.clicked = Some(*id);
            }
            if resp.double_clicked() {
                result.double_clicked = Some(*id);
            }
            ui.separator();
        }

        if hovered != self.hovered {
            result.hovered = Some(hovered);
        }

        result
    }
}

#[derive(Default, Debug)]
pub struct SpeciesListResponse {
    pub hovered: Option<Option<SpeciesId>>,
    pub clicked: Option<SpeciesId>,
    pub double_clicked: Option<SpeciesId>,
    pub species_view_action: Option<(SpeciesViewAction, SpeciesId)>,
}

#[derive(Debug, Clone)]
pub enum SpeciesViewAction {
    SelectNext,
    SelectPrev,
}

pub fn image(
    ui: &mut Ui,
    image_cache: &mut ImageCache,
    style: &Style,
    _db: &Database,
    species: &Species,
    width: f32,
) -> Option<SpeciesViewAction> {
    let n = species.examples.len();
    if n == 0 {
        let placeholder = Vec2::new(width, 0.75 * width);
        ui.add_sized(placeholder, Label::new("no images"));
        return None;
    }

    let mut result: Option<SpeciesViewAction> = None;

    let uri = &species.examples[species.current_example];
    let resp = add_image(ui, uri.clone(), image_cache, width, style.image.radius);
    if n > 1 {
        let mut ui = ui.new_child(
            UiBuilder::new()
                .max_rect(resp.rect.shrink(5.0))
                .layout(Layout::right_to_left(Align::Max)),
        );

        ui.horizontal(|ui| {
            if ui.button(ICON_PLAY_ARROW).clicked() {
                result = Some(SpeciesViewAction::SelectNext);
            }
            if ui.button(ICON_ARROW_BACK_2).clicked() {
                result = Some(SpeciesViewAction::SelectPrev);
            }
            ui.add(overlay_label(ImageCounter(n).to_string(), style));
        });
    }

    result
}

fn block(
    ui: &mut Ui,
    image_cache: &mut ImageCache,
    style: &Style,
    db: &Database,
    species: &Species,
    width: f32,
) -> Option<SpeciesViewAction> {
    let mut result: Option<SpeciesViewAction> = None;

    ui.horizontal(|ui| {
        result = image(ui, image_cache, style, db, species, width);

        ui.vertical(|ui| {
            ui.label(RichText::new(&species.latin).italics().heading());
            if !species.pl.is_empty() || !species.wikipedia_pl.is_empty() {
                ui.horizontal(|ui| {
                    format_pl(ui, species);
                });
            }

            if !species.en.is_empty() || !species.wikipedia_en.is_empty() {
                ui.horizontal(|ui| {
                    format_en(ui, species);
                });
            }

            if let Some(category) = &species.category {
                ui.horizontal(|ui| {
                    ui.label("category:");

                    let color = ui.visuals().strong_text_color();
                    ui.colored_label(color, category);
                });
            }
        });
    });

    result
}

pub fn format_latin(ui: &mut Ui, species: &Species) {
    ui.label(RichText::new(&species.latin).italics());
}

pub fn format_pl(ui: &mut Ui, species: &Species) {
    icon_pl(ui);
    if species.wikipedia_pl.is_empty() {
        ui.label(&species.pl);
    } else {
        if species.pl.is_empty() {
            ui.hyperlink_to("Wikipedia", &species.wikipedia_pl);
        } else {
            ui.hyperlink_to(&species.pl, &species.wikipedia_pl);
        }
    }
}

pub fn format_en(ui: &mut Ui, species: &Species) {
    icon_en(ui);
    if species.wikipedia_en.is_empty() {
        ui.label(&species.en);
    } else {
        if species.pl.is_empty() {
            ui.hyperlink_to("Wikipedia", &species.wikipedia_pl);
        } else {
            ui.hyperlink_to(&species.en, &species.wikipedia_en);
        }
    }
}

pub fn singleline(ui: &mut Ui, species: &Species) {
    format_latin(ui, species);
    if !species.pl.is_empty() | !species.wikipedia_pl.is_empty() {
        format_pl(ui, species);
    }

    if !species.en.is_empty() | !species.wikipedia_en.is_empty() {
        format_en(ui, species);
    }
}
