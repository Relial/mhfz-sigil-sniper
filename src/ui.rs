use std::{
    cmp::Ordering,
    ops::RangeInclusive,
    sync::{LazyLock, Once},
    time::Duration,
};

use anyhow::Result;
use eframe::egui::{
    self, Align, Align2, Button, CollapsingHeader, Color32, ComboBox, DragValue, FontId, Frame,
    Grid, Layout, Margin, Panel, Rect, RichText, ScrollArea, Sense, Stroke, Ui, UiBuilder, Vec2,
};
use jiff::{
    RoundMode, Span, SpanCompare, SpanRound, Timestamp, ToSpan, Unit, civil::datetime, tz::TimeZone,
};

use crate::{
    dat::SigilData,
    sigil::{
        calc::{RecipeSupport, SigilCraft, SkillRoll},
        identify::{OffsetSigil, OffsetSigilRoll, ReferenceSigil, identify_seed_offset},
        recipe::{rarity_recipe_name, recipe_name},
        sigil_skills::SigilSkill,
        snipe::{AfterSnipeOffset, Requirements, SnipeResult, snipe},
    },
};

static UNSUPPORTED_RECIPE: LazyLock<Color32> =
    LazyLock::new(|| Color32::from_rgb(237, 106, 90).gamma_multiply(0.8));
static MOSTLY_SUPPORTED_RECIPE: LazyLock<Color32> =
    LazyLock::new(|| Color32::from_rgb(232, 170, 20).gamma_multiply(0.8));
static SUPPORTED_RECIPE: LazyLock<Color32> =
    LazyLock::new(|| Color32::from_rgb(178, 239, 155).gamma_multiply(0.8));
static ZAOE_RECIPE: LazyLock<Color32> =
    LazyLock::new(|| Color32::from_rgb(165, 216, 255).gamma_multiply(0.8));
static RARITY_RECIPE: LazyLock<Color32> = LazyLock::new(|| Color32::from_rgb(137, 99, 186));

const MIN_REPAINT: Duration = Duration::from_millis(50);

#[derive(Debug, Clone, Copy)]
struct EditingDateTime {
    year: i16,
    month: i8,
    day: i8,
    hour: i8,
    minute: i8,
    second: i8,
}

impl EditingDateTime {
    fn from_timestamp(timestamp: Timestamp, time_zone: TimeZone) -> Self {
        let zoned = timestamp.to_zoned(time_zone);
        Self {
            year: zoned.year(),
            month: zoned.month(),
            day: zoned.day(),
            hour: zoned.hour(),
            minute: zoned.minute(),
            second: zoned.second(),
        }
    }

    fn to_timestamp(self, time_zone: TimeZone) -> Option<Timestamp> {
        let timestamp = datetime(
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
            0,
        )
        .to_zoned(time_zone)
        .ok()?
        .timestamp();
        Some(timestamp)
    }
}

#[derive(Debug, Clone)]
struct OffsetFinderState {
    target: Option<Timestamp>,
    input_date_time: EditingDateTime,
    input_timestamp: i64,
    crafted_sigils: Vec<OffsetSigil>,
    offset_candidates: Option<usize>,
    found_seed_offset: Option<(u32, u8)>,
}

impl OffsetFinderState {
    fn new(initial_timestamp: Timestamp, time_zone: TimeZone) -> Self {
        let dt = EditingDateTime::from_timestamp(initial_timestamp, time_zone);
        let ts = initial_timestamp.as_second();
        Self {
            target: None,
            input_date_time: dt,
            input_timestamp: ts,
            crafted_sigils: vec![OffsetSigil::new()],
            offset_candidates: None,
            found_seed_offset: None,
        }
    }

    fn apply_timestamp(&mut self, timestamp: Timestamp, time_zone: TimeZone) {
        self.input_date_time = EditingDateTime::from_timestamp(timestamp, time_zone);
        self.input_timestamp = timestamp.as_second();
    }
}

#[derive(Debug, Clone, Copy)]
struct SearchSettings {
    search_hours: u8,
    search_minutes: u8,
    search_seconds: u8,
    search_depth: usize,
}

impl Default for SearchSettings {
    fn default() -> Self {
        Self {
            search_hours: 1,
            search_minutes: 0,
            search_seconds: 0,
            search_depth: 200,
        }
    }
}

#[derive(Debug)]
struct SnipeState {
    recipe_idx: usize,
    search_settings: SearchSettings,
    requirements: Vec<SkillRoll>,
    results: Vec<SnipeResult>,
    tracked: Option<Timestamp>,
}

impl SnipeState {
    fn new(recipe: &SigilCraft, search_settings: Option<SearchSettings>) -> Self {
        let initial_requirements = recipe
            .normal_rolls
            .first()
            .map(|craft_roll| vec![craft_roll.skill_roll.clone(); 3])
            .unwrap_or_default();
        Self {
            recipe_idx: recipe.recipe_idx,
            search_settings: search_settings.unwrap_or_default(),
            requirements: initial_requirements,
            results: Vec::new(),
            tracked: None,
        }
    }
}

#[derive(PartialEq)]
enum UiTab {
    RecipeChoice,
    OffsetFinder,
    Snipe,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Japanese,
}

pub struct UiState {
    tab: UiTab,
    language: Language,
    time: Timestamp,
    time_zone: TimeZone,
    selected_recipe_idx: Option<usize>,
    offset: u8,
    sigil_data: SigilData,
    offset_finder: OffsetFinderState,
    snipe: Option<SnipeState>,
}

impl eframe::App for UiState {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        ui.request_repaint_after(MIN_REPAINT);
        self.time = Timestamp::now();

        let style = ui.style();
        Panel::top("Top panel")
            .resizable(false)
            .exact_size(20.0)
            .frame(Frame::side_top_panel(style).inner_margin(Margin::ZERO))
            .show_inside(ui, |ui| {
                self.top_panel(ui);
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let (time_display_rect, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 35.0), Sense::empty());
            self.shared_display(ui, time_display_rect);
            ui.add_space(10.0);
            match self.tab {
                UiTab::RecipeChoice => self.recipe_choice(ui),
                UiTab::OffsetFinder => self.offset_finder(ui),
                UiTab::Snipe => self.snipe(ui),
                UiTab::Info => self.info(ui),
            }
        });
    }
}

impl UiState {
    pub fn new(sigil_data: SigilData) -> Self {
        let initial_timestamp = Timestamp::now();
        let time_zone = TimeZone::system();
        Self {
            tab: UiTab::Info,
            language: Language::English,
            time: initial_timestamp,
            time_zone: time_zone.clone(),
            selected_recipe_idx: None,
            offset: 0,
            sigil_data,
            offset_finder: OffsetFinderState::new(initial_timestamp, time_zone),
            snipe: None,
        }
    }

    fn top_panel(&mut self, ui: &mut Ui) {
        ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);
        ui.horizontal(|ui| {
            if ui
                .add(Button::selectable(self.tab == UiTab::Info, "Info").corner_radius(0))
                .clicked()
            {
                self.tab = UiTab::Info;
            }
            ui.separator();
            if ui
                .add(Button::selectable(self.tab == UiTab::RecipeChoice, "Recipe").corner_radius(0))
                .clicked()
            {
                self.tab = UiTab::RecipeChoice;
            }
            ui.separator();
            if ui
                .add(
                    Button::selectable(self.tab == UiTab::OffsetFinder, "Find offset")
                        .corner_radius(0),
                )
                .clicked()
            {
                self.tab = UiTab::OffsetFinder;
            }
            ui.separator();
            if ui
                .add(Button::selectable(self.tab == UiTab::Snipe, "Snipe").corner_radius(0))
                .clicked()
            {
                self.tab = UiTab::Snipe;
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(
                        Button::selectable(self.language == Language::English, "English")
                            .corner_radius(0),
                    )
                    .clicked()
                {
                    self.language = Language::English;
                }
                if ui
                    .add(
                        Button::selectable(self.language == Language::Japanese, "日本語")
                            .corner_radius(0),
                    )
                    .clicked()
                {
                    self.language = Language::Japanese;
                }
                ui.label("Sigil language:");
            });
        });
    }

    fn shared_display(&mut self, ui: &mut Ui, rect: Rect) {
        let painter = ui.painter();
        if let Some(snipe) = &mut self.snipe
            && let Some(tracked) = snipe.tracked
        {
            let diff = (tracked - self.time)
                .round(
                    SpanRound::new()
                        .largest(Unit::Hour)
                        .smallest(Unit::Second)
                        .mode(RoundMode::Ceil),
                )
                .unwrap();
            let fmt = format!("{:#}", diff);
            let color = span_color(&diff).unwrap_or_else(|_| ui.visuals().strong_text_color());
            painter.text(
                rect.right_center(),
                Align2::RIGHT_CENTER,
                fmt,
                FontId::proportional(20.0),
                color,
            );

            if let Ok(ordering) = diff.compare(SpanCompare::from(-30.seconds())) {
                if ordering == Ordering::Less {
                    snipe.tracked = None;
                }
            } else {
                snipe.tracked = None;
            }
        }
        let timestamp_seconds = self.time.as_second();
        let local = self.time.to_zoned(self.time_zone.clone());
        let local_fmt = local.strftime("%F %H:%M:%S");
        let color = ui.visuals().strong_text_color();
        painter.text(
            rect.center_top() + Vec2::new(0.0, 5.0),
            Align2::CENTER_CENTER,
            timestamp_seconds,
            FontId::proportional(20.0),
            color,
        );
        painter.text(
            rect.center_bottom() - Vec2::new(0.0, 5.0),
            Align2::CENTER_CENTER,
            local_fmt,
            FontId::proportional(20.0),
            color,
        );
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            let recipe = self
                .selected_recipe_idx
                .map(|recipe_idx| recipe_name(recipe_idx, self.language))
                .unwrap_or("None");
            ui.horizontal(|ui| {
                ui.strong("Offset:");
                ui.add(DragValue::new(&mut self.offset).speed(0.02));
            });
            ui.strong(format!("Recipe: {}", recipe));
        });
    }

    fn recipe_choice(&mut self, ui: &mut Ui) {
        ui.style_mut().interaction.selectable_labels = false;
        let paint_area_size = Vec2::new(18.0, 18.0);
        let circle_radius = 6.0;
        let info_font_size = 10.0;
        let text_color = ui.visuals().strong_text_color().gamma_multiply(0.8);
        let circle_stroke = Stroke::new(1.0, text_color);

        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(paint_area_size, Sense::HOVER);
            ui.painter().rect_filled(rect, 0, *SUPPORTED_RECIPE);
            ui.strong("100% predictable");
            let (rect, resp) = ui.allocate_exact_size(paint_area_size, Sense::HOVER);
            ui.painter()
                .circle_stroke(rect.center(), circle_radius, circle_stroke);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                "?",
                FontId::proportional(info_font_size),
                text_color,
            );
            resp.on_hover_text("These recipes will always give you the predicted result.")
        });
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(paint_area_size, Sense::HOVER);
            ui.painter().rect_filled(rect, 0, *MOSTLY_SUPPORTED_RECIPE);
            ui.strong("Mostly predictable");
            let (rect, resp) = ui.allocate_exact_size(paint_area_size, Sense::HOVER);
            ui.painter()
                .circle_stroke(rect.center(), circle_radius, circle_stroke);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                "?",
                FontId::proportional(info_font_size),
                text_color,
            );
            resp.on_hover_text(
                "These recipes can't be fully predicted due to being able to roll from the non-standard table.\nUsually these will still be correctly predicted.",
            )
        });
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(paint_area_size, Sense::HOVER);
            ui.painter().rect_filled(rect, 0, *UNSUPPORTED_RECIPE);
            ui.strong("Unpredictable");
            let (rect, resp) = ui.allocate_exact_size(paint_area_size, Sense::HOVER);
            ui.painter()
                .circle_stroke(rect.center(), circle_radius, circle_stroke);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                "?",
                FontId::proportional(info_font_size),
                text_color,
            );
            resp.on_hover_text(
                "These recipes use a random budget component that can't be predicted.\nPredictions are made with the assumption you roll the maximum budget.\nIf you roll max budget these become mostly predictable.",
            )
        });
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(paint_area_size, Sense::HOVER);
            ui.painter().rect_filled(rect, 0, *ZAOE_RECIPE);
            ui.strong("Zenith AoE");
            let (rect, resp) = ui.allocate_exact_size(paint_area_size, Sense::HOVER);
            ui.painter()
                .circle_stroke(rect.center(), circle_radius, circle_stroke);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                "?",
                FontId::proportional(info_font_size),
                text_color,
            );
            resp.on_hover_text(
                "This recipe's point ranges can be reliably predicted, but which skills you get cannot.\nAs such, the results predicted from this only apply if the slot rolls something with a point value.",
            )
        });
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(paint_area_size, Sense::HOVER);
            ui.painter().rect_filled(rect, 0, *RARITY_RECIPE);
            ui.strong("Rarity");
            let (rect, resp) = ui.allocate_exact_size(paint_area_size, Sense::HOVER);
            ui.painter()
                .circle_stroke(rect.center(), circle_radius, circle_stroke);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                "?",
                FontId::proportional(info_font_size),
                text_color,
            );
            resp.on_hover_text(
                "These recipes always roll the same result. They're used heavily in the sniping process to advance rng in a predictable way, but they can't be used for offset finding because the result never varies.",
            )
        });

        ui.separator();

        ScrollArea::vertical().show(ui, |ui| {
            ui.take_available_space();
            for recipe in self.sigil_data.recipes.iter().skip(1) {
                let color = match recipe.support {
                    RecipeSupport::Full => *SUPPORTED_RECIPE,
                    RecipeSupport::Mostly => *MOSTLY_SUPPORTED_RECIPE,
                    RecipeSupport::Low => *UNSUPPORTED_RECIPE,
                    RecipeSupport::ZAoe => *ZAOE_RECIPE,
                    RecipeSupport::Rarity => *RARITY_RECIPE,
                };

                let recipe_s = format!(
                    "{}. {}",
                    recipe.recipe_idx,
                    recipe_name(recipe.recipe_idx, self.language)
                );

                if ui
                    .add(Button::selectable(
                        self.selected_recipe_idx
                            .is_some_and(|idx| idx == recipe.recipe_idx),
                        RichText::new(recipe_s).color(color),
                    ))
                    .clicked()
                {
                    self.selected_recipe_idx = Some(recipe.recipe_idx);
                }
            }
        });
    }

    fn offset_finder(&mut self, ui: &mut Ui) {
        let Some(recipe_idx) = &self.selected_recipe_idx else {
            ui.label("Select a recipe first");
            return;
        };
        let recipe = &self.sigil_data.recipes[*recipe_idx];

        if ui.button("Reset").clicked() {
            self.offset_finder = OffsetFinderState::new(self.time, self.time_zone.clone());
        }
        let state = &mut self.offset_finder;
        if state.target.is_none() {
            state.apply_timestamp(self.time, self.time_zone.clone());
        }

        ui.label("Enter the time you opened the sigil recipe list:");
        Grid::new("Offset finder input").show(ui, |ui| {
            ui.label("Year");
            ui.label("Month");
            ui.label("Day");
            ui.label("Hour");
            ui.label("Minute");
            ui.label("Second");

            ui.end_row();

            let r0 = ui.add(DragValue::new(&mut state.input_date_time.year).speed(0.02));
            let r1 = ui.add(
                DragValue::new(&mut state.input_date_time.month)
                    .range(1..=12)
                    .speed(0.02),
            );
            let r2 = ui.add(
                DragValue::new(&mut state.input_date_time.day)
                    .range(1..=31)
                    .speed(0.02),
            );
            let r3 = ui.add(
                DragValue::new(&mut state.input_date_time.hour)
                    .range(0..=23)
                    .speed(0.02),
            );
            let r4 = ui.add(
                DragValue::new(&mut state.input_date_time.minute)
                    .range(0..=59)
                    .speed(0.02),
            );
            let r5 = ui.add(
                DragValue::new(&mut state.input_date_time.second)
                    .range(0..=59)
                    .speed(0.02),
            );

            for response in [r0, r1, r2, r3, r4, r5] {
                if state.target.is_none() && (response.gained_focus() || response.drag_started()) {
                    state.target = Some(self.time);
                }
                if response.lost_focus()
                    && let Some(converted) =
                        state.input_date_time.to_timestamp(self.time_zone.clone())
                {
                    state.target = Some(converted)
                }
                if response.changed()
                    && let Some(converted) =
                        state.input_date_time.to_timestamp(self.time_zone.clone())
                {
                    state.input_timestamp = converted.as_second();
                }
            }
        });
        ui.label("Unix timestamp");
        let resp = ui.add(DragValue::new(&mut state.input_timestamp).speed(0.02));
        if state.target.is_none() && (resp.gained_focus() || resp.drag_started()) {
            state.target = Some(self.time);
        }
        if resp.lost_focus()
            && let Ok(converted) = Timestamp::from_second(state.input_timestamp)
        {
            state.target = Some(converted)
        }
        if resp.changed()
            && let Ok(converted) = Timestamp::from_second(state.input_timestamp)
        {
            state.input_date_time =
                EditingDateTime::from_timestamp(converted, self.time_zone.clone());
        }

        let Some(target) = state.target else {
            return;
        };
        ui.label("Craft sigils 1 at a time");
        let possible_offsets = state
            .offset_candidates
            .map(|count| count.to_string())
            .unwrap_or("None".to_string());
        let color = if let Some(count) = state.offset_candidates {
            match count {
                0 => Color32::RED,
                1 => Color32::GREEN,
                _ => Color32::RED
                    .lerp_to_gamma(Color32::YELLOW, 1.0 - (count as f32 / u8::MAX as f32)),
            }
        } else {
            Color32::RED
        };
        ui.horizontal(|ui| {
            ui.label("Possible offsets: ");
            ui.colored_label(color, possible_offsets);
            if let Some((_, offset)) = state.found_seed_offset {
                ui.add_space(5.0);
                ui.label(format!("Found offset: {}", offset));
            }
        });
        let stroke = ui.visuals().window_stroke;
        let mut calc = false;
        let mut remove = None;
        let mut possible_skills: Vec<SigilSkill> = recipe
            .normal_rolls
            .iter()
            .map(|roll| roll.skill_roll.skill)
            .chain(recipe.not_blacklisted.iter().copied())
            .collect();
        possible_skills.sort();
        possible_skills.dedup();
        ui.push_id("Offset finder sigils", |ui| {
            for (sigil_i, crafted) in state.crafted_sigils.iter_mut().enumerate() {
                ui.horizontal_top(|ui| {
                    Frame::new()
                        .stroke(stroke)
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            Grid::new("Grid").show(ui, |ui| {
                                for (roll_i, roll_option) in crafted.rolls.iter_mut().enumerate() {
                                    if let Some(roll) = roll_option {
                                        let old = roll.skill;
                                        ComboBox::from_id_salt((sigil_i, roll_i))
                                            .width(150.0)
                                            .selected_text(roll.skill.name(self.language))
                                            .show_ui(ui, |ui| {
                                                for possible_skill in &possible_skills {
                                                    ui.selectable_value(
                                                        &mut roll.skill,
                                                        *possible_skill,
                                                        possible_skill.name(self.language),
                                                    );
                                                }
                                            });
                                        if roll.skill != old {
                                            calc = true;
                                        }
                                        let resp = ui.add(
                                            DragValue::new(&mut roll.points)
                                                .range(-10..=20)
                                                .speed(0.02),
                                        );
                                        if resp.lost_focus() || resp.drag_stopped() {
                                            calc = true;
                                        }
                                    } else {
                                        if ComboBox::from_id_salt((sigil_i, roll_i))
                                            .width(150.0)
                                            .show_ui(ui, |_| {})
                                            .response
                                            .clicked()
                                            && let Some(first) = possible_skills.first()
                                        {
                                            *roll_option = Some(OffsetSigilRoll::new(*first));
                                        }
                                    }
                                    ui.end_row();
                                }
                            });
                        });
                    if ui.add(Button::new("❌").frame(false)).clicked() {
                        remove = Some(sigil_i);
                    }
                });
            }
        });
        if let Some(remove) = remove {
            state.crafted_sigils.remove(remove);
            calc = true;
        }
        if calc {
            let references: Vec<ReferenceSigil> = state
                .crafted_sigils
                .iter()
                .map(|sigil| {
                    ReferenceSigil::from_crafted_sigil(sigil, recipe, &self.sigil_data.skill_ranges)
                })
                .collect();
            match identify_seed_offset(
                &references,
                target.as_second() as u32,
                recipe,
                &self.sigil_data.skill_ranges,
            ) {
                crate::sigil::identify::IdentifyResult::NotFound => {
                    state.offset_candidates = None;
                    state.found_seed_offset = None;
                }
                crate::sigil::identify::IdentifyResult::OneResult((seed, offset)) => {
                    state.offset_candidates = Some(1);
                    state.found_seed_offset = Some((seed, offset));
                    self.offset = offset;
                }
                crate::sigil::identify::IdentifyResult::MultipleResults(items) => {
                    state.offset_candidates = Some(items.len());
                    state.found_seed_offset = None;
                }
            }
        }
        if ui.button("Add").clicked() {
            state.crafted_sigils.push(OffsetSigil::new());
        }
    }

    fn snipe(&mut self, ui: &mut Ui) {
        let Some(recipe_idx) = &self.selected_recipe_idx else {
            ui.label("Select a recipe first");
            return;
        };
        let recipe = &self.sigil_data.recipes[*recipe_idx];
        let state = self.snipe.get_or_insert(SnipeState::new(recipe, None));
        if state.recipe_idx != *recipe_idx {
            *state = SnipeState::new(recipe, Some(state.search_settings));
        }

        ui.horizontal(|ui| {
            ui.label("Depth per seed: ");
            ui.add(DragValue::new(&mut state.search_settings.search_depth));
        });
        ui.horizontal(|ui| {
            ui.label("Search ahead: ");
            Grid::new("search_time").show(ui, |ui| {
                ui.label("Hours");
                ui.label("Minutes");
                ui.label("Seconds");

                ui.end_row();

                ui.add(DragValue::new(&mut state.search_settings.search_hours).speed(0.02));
                ui.add(
                    DragValue::new(&mut state.search_settings.search_minutes)
                        .speed(0.02)
                        .range(0..=59),
                );
                ui.add(
                    DragValue::new(&mut state.search_settings.search_seconds)
                        .speed(0.02)
                        .range(0..=59),
                );
            });
        });

        ui.label("The order you add requirements doesn't matter.");
        ui.label("Snipe target:");

        let mut possible_normal_rolls: Vec<SigilSkill> = recipe
            .normal_rolls
            .iter()
            .map(|roll| roll.skill_roll.skill)
            .collect();
        possible_normal_rolls.sort();
        possible_normal_rolls.dedup();

        let mut remove = None;
        ui.push_id("Snipe", |ui| {
            let stroke = ui.visuals().window_stroke;
            Frame::new()
                .stroke(stroke)
                .inner_margin(4.0)
                .show(ui, |ui| {
                    for (req_i, req) in state.requirements.iter_mut().enumerate() {
                        ui.horizontal_top(|ui| {
                            ComboBox::from_id_salt((req_i, 0))
                                .width(150.0)
                                .selected_text(req.skill.name(self.language))
                                .show_ui(ui, |ui| {
                                    let mut changed = false;
                                    for skill in &possible_normal_rolls {
                                        changed |= ui
                                            .selectable_value(
                                                &mut req.skill,
                                                *skill,
                                                skill.name(self.language),
                                            )
                                            .changed();
                                    }
                                    ui.separator();
                                    for skill in &recipe.not_blacklisted {
                                        changed |= ui
                                            .selectable_value(
                                                &mut req.skill,
                                                *skill,
                                                skill.name(self.language),
                                            )
                                            .changed();
                                    }
                                    if changed {
                                        req.points = req
                                            .skill
                                            .points_options(recipe, &self.sigil_data.skill_ranges)
                                            .first()
                                            .unwrap()
                                            .clone();
                                    }
                                });
                            ComboBox::from_id_salt((req_i, 1))
                                .width(100.0)
                                .selected_text(range_fmt(&req.points))
                                .show_ui(ui, |ui| {
                                    let options = req
                                        .skill
                                        .points_options(recipe, &self.sigil_data.skill_ranges);
                                    for range in options {
                                        ui.selectable_value(
                                            &mut req.points,
                                            range.clone(),
                                            range_fmt(&range),
                                        );
                                    }
                                });
                            if ui.add(Button::new("❌").frame(false)).clicked() {
                                remove = Some(req_i);
                            }
                        });
                    }
                    if state.requirements.len() < 3 && ui.button("Add requirement").clicked() {
                        state
                            .requirements
                            .push(recipe.normal_rolls.first().unwrap().skill_roll.clone());
                    }
                });
        });
        if let Some(idx) = remove {
            state.requirements.remove(idx);
        }

        if ui.add_sized([100.0, 20.0], Button::new("Snipe")).clicked() {
            let s = &state.search_settings;
            let seed_count = s.search_hours as usize * 60 * 60
                + s.search_minutes as usize * 60
                + s.search_seconds as usize;
            state.results = snipe(
                recipe,
                &self.sigil_data.skill_ranges,
                self.time.as_second() as u32 + self.offset as u32,
                seed_count,
                state.search_settings.search_depth,
                &Requirements::from_skill_rolls(&state.requirements),
            )
        }

        if !state.results.is_empty() {
            ui.separator();
        }
        ScrollArea::vertical().show(ui, |ui| {
            ui.take_available_space();
            let rarity_name = rarity_recipe_name(self.language);
            for (i, result) in state.results.iter().enumerate().take(1000) {
                let with_offset = result.seed - self.offset as u32;
                let timestamp = Timestamp::from_second(with_offset as i64).unwrap();
                let diff = (timestamp - self.time)
                    .round(
                        SpanRound::new()
                            .largest(Unit::Hour)
                            .smallest(Unit::Second)
                            .mode(RoundMode::Ceil),
                    )
                    .unwrap();
                let diff_color =
                    span_color(&diff).unwrap_or_else(|_| ui.visuals().strong_text_color());
                let diff_fmt = format!("{:#}", diff);
                let time = timestamp.to_zoned(self.time_zone.clone());
                let time_fmt = time.strftime("%F %H:%M:%S");
                ui.horizontal(|ui| {
                    CollapsingHeader::new(format!("{} | {}", with_offset, time_fmt))
                        .id_salt(i)
                        .show(ui, |ui| {
                            ui.allocate_ui(Vec2::new(210.0, 1.0), |ui| {
                                ui.label(format!("Distance from seed: {}", result.advances));
                                ui.label("Sequence:");
                                if result.rarity_full_batches > 0 {
                                    ui.label(format!(
                                        "10x {} - {} times",
                                        rarity_name, result.rarity_full_batches
                                    ));
                                }
                                let end_sequence: Vec<String> = result
                                    .rarity_crafts
                                    .iter()
                                    .map(|craft_count| format!("{}x {}", craft_count, rarity_name))
                                    .collect();
                                if !end_sequence.is_empty() {
                                    let joined = end_sequence.join(" -> ");
                                    ui.label(joined);
                                }
                                ui.label(recipe_name(*recipe_idx, self.language));
                                ui.separator();
                                if ui.button("Track").clicked() {
                                    state.tracked = Some(timestamp);
                                }
                                ui.label("Offset after sequence:");
                                match result.offset {
                                    AfterSnipeOffset::Certain(offset) => {
                                        ui.horizontal(|ui| {
                                            ui.label(offset.to_string());
                                            if ui.button("Apply").clicked() {
                                                self.offset = offset;
                                            }
                                        });
                                    }
                                    AfterSnipeOffset::Uncertain((no_red, red)) => {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("No red roll: {}", no_red));
                                            if ui.button("Apply").clicked() {
                                                self.offset = no_red;
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label(format!("Red roll: {}", red));
                                            if ui.button("Apply").clicked() {
                                                self.offset = red;
                                            }
                                        });
                                    }
                                }
                            });
                        });
                    ui.colored_label(diff_color, diff_fmt);
                });
            }
        });
    }

    fn info(&mut self, ui: &mut Ui) {
        ui.heading("How to use");
        ui.label("1. Select a recipe for offset finding. This should be one of the 100% predictable recipes, and does not need to be the recipe you want to snipe.");
        ui.label(
            "2. In-game open the sigil recipe list and input the time you opened it in the tool.",
        );
        ui.label("3. Craft the selected sigil 1 at a time and input the result into the tool until an offset is found. Don't make a batch of more than 1 sigil.");
        ui.label(
            "The offset you find will persist until you make a sigil, through quests and logouts.",
        );
        ui.label("4. You can now choose a sigil to snipe. Enter what you want into the calculator and press Snipe, then open the sigil recipe list at a time given by one of the results.");
        ui.label("5. Craft any of the rarity recipes exactly according to the sequence shown. \"10x - 2 times\" means craft a batch of 10 twice. \"8x -> 1x -> 1x\" means craft an 8 batch, then a 1 batch, then a 1 batch.");
        ui.label("6. Finally, craft your snipe target.");
    }
}

fn range_fmt(range: &RangeInclusive<i16>) -> String {
    format!("{} to {}", range.start(), range.end())
}

fn span_color(span: &Span) -> Result<Color32> {
    let seconds = span.total(Unit::Second)?;
    const HOUR_SEC: f64 = 60.0 * 60.0;
    const FIVE_MIN_SEC: f64 = 60.0 * 5.0;
    if !(0.0..HOUR_SEC).contains(&seconds) {
        Ok(*UNSUPPORTED_RECIPE)
    } else if seconds > FIVE_MIN_SEC {
        let t = 1.0 - (seconds / (HOUR_SEC - FIVE_MIN_SEC));
        Ok(UNSUPPORTED_RECIPE.lerp_to_gamma(*MOSTLY_SUPPORTED_RECIPE, t as f32))
    } else {
        let t = 1.0 - (seconds / FIVE_MIN_SEC);
        Ok(MOSTLY_SUPPORTED_RECIPE.lerp_to_gamma(*SUPPORTED_RECIPE, t as f32))
    }
}
