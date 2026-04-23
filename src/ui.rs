use std::{ops::RangeInclusive, sync::LazyLock, time::Duration};

use anyhow::Result;
use eframe::egui::{
    self, Align, Align2, Button, CollapsingHeader, Color32, ComboBox, DragValue, FontId, Frame,
    Grid, Layout, Margin, Panel, Rect, RichText, ScrollArea, Sense, Stroke, Ui, UiBuilder, Vec2,
    include_image,
};
use jiff::{
    RoundMode, Span, SpanRelativeTo, SpanRound, SpanTotal, Timestamp, ToSpan, Unit, Zoned,
    civil::{DateTime, DateTimeRound, datetime},
    tz::TimeZone,
};

use crate::{
    dat::SigilData,
    sigil::{
        calc::{RecipeSupport, SigilCraft, SigilType, SkillRoll},
        identify::{OffsetSigil, OffsetSigilRoll, ReferenceSigil, identify_seed_offset},
        recipe::{recipe_name, zenith_heroics_recipe_name, zenith_resistance_recipe_name},
        sigil_skills::{SigilSkill, SpecificEffect, points_named_effect},
        snipe::{Requirements, SnipeResult, ZenithStrategy, snipe},
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

#[derive(Debug, Clone, Copy, Default)]
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
        let timestamp = self.to_zoned(time_zone)?.timestamp();
        Some(timestamp)
    }

    fn to_zoned(self, time_zone: TimeZone) -> Option<Zoned> {
        self.to_date_time().to_zoned(time_zone).ok()
    }

    fn to_date_time(self) -> DateTime {
        datetime(
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
            0,
        )
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DesyncSearchStep {
    InitialBehind,
    InitialAhead,
    NarrowingBehind,
    NarrowingAhead,
    Done,
}

#[derive(Debug, Clone)]
struct DesyncFinderState {
    input_buff_start: EditingDateTime,
    input_buff_end: EditingDateTime,
    first_search_desync: Option<Span>,
    search_step: DesyncSearchStep,
    target_second: i8,
    search_low: i8,
    search_high: i8,
    searched: Vec<(i8, Span)>,
    result: Option<i32>,
}

impl DesyncFinderState {
    fn new(initial_timestamp: Timestamp, time_zone: TimeZone) -> Self {
        let input_buff_start =
            EditingDateTime::from_timestamp(initial_timestamp, time_zone.clone());
        let mut input_buff_end =
            EditingDateTime::from_timestamp(initial_timestamp + 90.minutes(), time_zone);
        input_buff_end.second = 0;
        Self {
            input_buff_start,
            input_buff_end,
            first_search_desync: None,
            search_step: DesyncSearchStep::InitialBehind,
            target_second: 59,
            search_low: 0,
            search_high: 60,
            searched: Vec::new(),
            result: None,
        }
    }

    fn calculate_desync(&mut self) {
        let start = self
            .input_buff_start
            .to_date_time()
            .round(
                DateTimeRound::new()
                    .smallest(Unit::Minute)
                    .mode(RoundMode::Trunc),
            )
            .unwrap();
        let expected_end = start + 90.minutes();
        let actual_end = self.input_buff_end.to_date_time();
        self.searched
            .push((self.target_second, expected_end - actual_end));
        match self.search_step {
            DesyncSearchStep::InitialBehind => {
                assert_eq!(59, self.target_second);
                if actual_end > expected_end {
                    self.first_search_desync = Some(expected_end - actual_end);
                    self.search_step = DesyncSearchStep::NarrowingBehind;
                    self.target_second = 30;
                } else {
                    self.search_step = DesyncSearchStep::InitialAhead;
                    self.target_second = 0;
                }
            }
            DesyncSearchStep::InitialAhead => {
                assert_eq!(0, self.target_second);
                if actual_end < expected_end {
                    self.first_search_desync = Some(expected_end - actual_end);
                    self.search_step = DesyncSearchStep::NarrowingAhead;
                    self.target_second = 30;
                }
            }
            DesyncSearchStep::NarrowingBehind => {
                let Some(original_behind) = self.first_search_desync else {
                    panic!(
                        "Reached desync search step narrowing behind without an original behind set."
                    );
                };
                let new_behind = expected_end - actual_end;
                if new_behind == original_behind.fieldwise() {
                    self.search_high = self.target_second;
                    self.target_second = (self.target_second + self.search_low) / 2;
                } else {
                    self.search_low = self.target_second;
                    self.target_second = (self.target_second + self.search_high) / 2;
                }
            }
            DesyncSearchStep::NarrowingAhead => {
                let Some(original_ahead) = self.first_search_desync else {
                    panic!(
                        "Reached desync search step narrowing ahead without an original ahead set."
                    );
                };
                let new_ahead = expected_end - actual_end;
                if new_ahead == original_ahead.fieldwise() {
                    self.search_low = self.target_second;
                    self.target_second = (self.target_second + self.search_high) / 2;
                } else {
                    self.search_high = self.target_second;
                    self.target_second = (self.target_second + self.search_low) / 2;
                }
            }
            DesyncSearchStep::Done => {}
        }
        if let Some((_, desync)) = self
            .searched
            .iter()
            .find(|(sec, _)| *sec == self.target_second)
        {
            self.search_step = DesyncSearchStep::Done;
            self.result = if let Some(original_desync) = self.first_search_desync {
                if original_desync.is_positive() {
                    let narrowed_secs = if desync == original_desync.fieldwise() {
                        self.target_second + 1
                    } else {
                        self.target_second
                    };
                    let actual_desync = original_desync
                        .checked_sub((60 - narrowed_secs).seconds())
                        .unwrap();
                    let seconds = actual_desync
                        .total(SpanTotal::from(Unit::Second).days_are_24_hours())
                        .unwrap() as i32;
                    Some(seconds)
                } else {
                    let narrowed_secs = if desync == original_desync.fieldwise() {
                        self.target_second
                    } else {
                        self.target_second + 1
                    };
                    let actual_desync = original_desync
                        .checked_add((narrowed_secs.seconds(), SpanRelativeTo::days_are_24_hours()))
                        .unwrap();
                    let seconds = actual_desync
                        .total(SpanTotal::from(Unit::Second).days_are_24_hours())
                        .unwrap() as i32;
                    Some(seconds)
                }
            } else {
                Some(0)
            };
        }
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
    selected_result: Option<usize>,
    zenith_strategy: ZenithStrategy,
}

impl SnipeState {
    fn new(recipe: &SigilCraft, search_settings: Option<SearchSettings>) -> Self {
        let initial_requirements =
            if recipe.sigil_type == SigilType::Zenith && recipe.support != RecipeSupport::ZAoe {
                let mut cooldown = None;
                let mut duration = None;
                let mut zskill = None;
                for roll in &recipe.normal_rolls {
                    if roll.skill_roll.skill == SigilSkill::ZCooldown {
                        cooldown.get_or_insert(roll.skill_roll.clone());
                    } else if roll.skill_roll.skill == SigilSkill::ZDuration {
                        duration.get_or_insert(roll.skill_roll.clone());
                    } else {
                        zskill.get_or_insert(roll.skill_roll.clone());
                    }
                }
                if let Some(cd) = cooldown
                    && let Some(dur) = duration
                    && let Some(z) = zskill
                {
                    vec![cd, dur, z]
                } else {
                    Vec::new()
                }
            } else {
                recipe
                    .normal_rolls
                    .first()
                    .map(|craft_roll| vec![craft_roll.skill_roll.clone(); 3])
                    .unwrap_or_default()
            };
        Self {
            recipe_idx: recipe.recipe_idx,
            search_settings: search_settings.unwrap_or_default(),
            requirements: initial_requirements,
            results: Vec::new(),
            selected_result: None,
            zenith_strategy: ZenithStrategy::Resistance,
        }
    }
}

#[derive(PartialEq)]
enum UiTab {
    Desync,
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
    desync_offset: i32,
    sigil_data: SigilData,
    offset_finder: OffsetFinderState,
    desync_finder: DesyncFinderState,
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
                UiTab::Desync => self.find_desync(ui),
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
            desync_offset: 0,
            sigil_data,
            offset_finder: OffsetFinderState::new(initial_timestamp, time_zone.clone()),
            desync_finder: DesyncFinderState::new(initial_timestamp, time_zone),
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
                .add(Button::selectable(self.tab == UiTab::Desync, "Find desync").corner_radius(0))
                .clicked()
            {
                self.tab = UiTab::Desync;
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
            && let Some(result_idx) = snipe.selected_result
        {
            let seed_with_offset = (snipe.results[result_idx].seed - self.offset as u32)
                .wrapping_add_signed(self.desync_offset);
            let tracked = Timestamp::from_second(seed_with_offset as i64).unwrap();
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

    fn find_desync(&mut self, ui: &mut Ui) {
        if ui.button("Reset").clicked() {
            self.desync_finder = DesyncFinderState::new(self.time, self.time_zone.clone());
        }
        ui.separator();
        let state = &mut self.desync_finder;
        state.input_buff_start.second = state.target_second;
        ui.horizontal(|ui| {
            ui.label("Desync:");
            ui.add(DragValue::new(&mut self.desync_offset));
        });

        ui.label("Use diva song buffs to find any desync with server time. The buff is applied when you exit the cutscene, so enter the cutscene before your target time and be ready to skip.");
        ui.label("The buff expiration time can be found in your personal status on the 8th page. Exit and re-enter the area to be able to receive the buff again.");
        ui.label("It's important that the timing you get the buff is consistent each time, since server time seconds are most likely not changing at the same time your system seconds are. Using a metronome is recommended.");
        let desync_s = match state.search_step {
            DesyncSearchStep::InitialBehind => {
                "Checking if behind server. Take a diva buff.".to_string()
            }
            DesyncSearchStep::InitialAhead => {
                "Checking if ahead server. Take a diva buff.".to_string()
            }
            DesyncSearchStep::NarrowingBehind => {
                "Behind some amount. Take a diva buff.".to_string()
            }
            DesyncSearchStep::NarrowingAhead => "Ahead some amount. Take a diva buff.".to_string(),
            DesyncSearchStep::Done => format!("Found {} second desync", self.desync_offset),
        };
        let color = match state.search_step {
            DesyncSearchStep::InitialBehind => *MOSTLY_SUPPORTED_RECIPE,
            DesyncSearchStep::InitialAhead => *MOSTLY_SUPPORTED_RECIPE,
            DesyncSearchStep::NarrowingBehind => *MOSTLY_SUPPORTED_RECIPE,
            DesyncSearchStep::NarrowingAhead => *MOSTLY_SUPPORTED_RECIPE,
            DesyncSearchStep::Done => {
                if self.desync_offset == 0 {
                    *SUPPORTED_RECIPE
                } else {
                    *UNSUPPORTED_RECIPE
                }
            }
        };
        ui.colored_label(color, desync_s);

        ui.scope(|ui| {
            if state.search_step == DesyncSearchStep::Done {
                ui.disable();
            }
            ui.label("Enter time you canceled the diva cutscene:");
            Grid::new("Desync finder start").show(ui, |ui| {
                ui.label("Year");
                ui.label("Month");
                ui.label("Day");
                ui.label("Hour");
                ui.label("Minute");
                ui.label("Second");

                ui.end_row();

                ui.add(DragValue::new(&mut state.input_buff_start.year).speed(0.02));
                ui.add(
                    DragValue::new(&mut state.input_buff_start.month)
                        .range(1..=12)
                        .speed(0.02),
                );
                ui.add(
                    DragValue::new(&mut state.input_buff_start.day)
                        .range(1..=31)
                        .speed(0.02),
                );
                ui.add(
                    DragValue::new(&mut state.input_buff_start.hour)
                        .range(0..=23)
                        .speed(0.02),
                );
                ui.add(
                    DragValue::new(&mut state.input_buff_start.minute)
                        .range(0..=59)
                        .speed(0.02),
                );
                ui.strong(state.input_buff_start.second.to_string());
            });

            ui.separator();

            ui.label("Enter buff expiration:");
            Grid::new("Desync finder end").show(ui, |ui| {
                ui.label("Year");
                ui.label("Month");
                ui.label("Day");
                ui.label("Hour");
                ui.label("Minute");

                ui.end_row();

                ui.add(DragValue::new(&mut state.input_buff_end.year).speed(0.02));
                ui.add(
                    DragValue::new(&mut state.input_buff_end.month)
                        .range(1..=12)
                        .speed(0.02),
                );
                ui.add(
                    DragValue::new(&mut state.input_buff_end.day)
                        .range(1..=31)
                        .speed(0.02),
                );
                ui.add(
                    DragValue::new(&mut state.input_buff_end.hour)
                        .range(0..=23)
                        .speed(0.02),
                );
                ui.add(
                    DragValue::new(&mut state.input_buff_end.minute)
                        .range(0..=59)
                        .speed(0.02),
                );
            });
            ui.separator();
            if ui.button("Find desync").clicked() {
                state.calculate_desync();
                if state.search_step == DesyncSearchStep::Done {
                    self.desync_offset = state.result.unwrap();
                }
            }
        });
    }

    fn recipe_choice(&mut self, ui: &mut Ui) {
        ui.style_mut().interaction.selectable_labels = false;
        let paint_area_size = Vec2::new(18.0, 18.0);
        let circle_radius = 6.0;
        let info_font_size = 10.0;
        let text_color = ui.visuals().strong_text_color().gamma_multiply(0.8);
        let circle_stroke = Stroke::new(1.0, text_color);

        Grid::new("Recipe explanations").show(ui, |ui| {
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
                if resp.contains_pointer() {
                    resp.show_tooltip_text("These recipes will always give you the predicted result.");
                }
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
                if resp.contains_pointer() {
                    resp.show_tooltip_text("These recipes can't be fully predicted due to being able to roll from the non-standard table.\nUsually these will still be correctly predicted.");
                }
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
                if resp.contains_pointer() {
                    resp.show_tooltip_text("These recipes use a random budget component that can't be predicted.\nPredictions are made with the assumption you roll the maximum budget.\nIf you roll max budget these become mostly predictable.");
                }
            });

            ui.end_row();

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
                if resp.contains_pointer() {
                    resp.show_tooltip_text("This recipe's point ranges can be reliably predicted, but which skills you get cannot.\nAs such, the results predicted from this only apply if the slot rolls something with a point value.");
                }
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
                if resp.contains_pointer() {
                    resp.show_tooltip_text("These recipes always roll the same result. They're used heavily in the sniping process to advance rng in a predictable way, but they can't be used for offset finding because the result never varies.");
                }
            });
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(paint_area_size, Sense::HOVER);
                ui.painter().rect_filled(rect, 0, Color32::GRAY.gamma_multiply(ui.visuals().disabled_alpha()));
                ui.strong("Unsupported");
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
                if resp.contains_pointer() {
                    resp.show_tooltip_text("These technically aren't any different from other recipes but due to being in their own category they're more restrictive to snipe and would require their own handling. They're not very good sigils anyway.");
                }
            });
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
                    RecipeSupport::Unsupported => Color32::GRAY,
                };

                let recipe_s = format!(
                    "{}. {}",
                    recipe.recipe_idx,
                    recipe_name(recipe.recipe_idx, self.language)
                );

                if ui
                    .add_enabled(
                        recipe.support != RecipeSupport::Unsupported,
                        Button::selectable(
                            self.selected_recipe_idx
                                .is_some_and(|idx| idx == recipe.recipe_idx),
                            RichText::new(recipe_s).color(color),
                        ),
                    )
                    .clicked()
                {
                    self.selected_recipe_idx = Some(recipe.recipe_idx);
                    self.offset_finder = OffsetFinderState::new(self.time, self.time_zone.clone());
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
        ui.label("Craft sigils 1 at a time. Do not back out of the recipe list or change the time above after you've started crafting.");
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
            .collect();
        possible_skills.sort();
        possible_skills.dedup();
        ui.push_id("Offset finder sigils", |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.take_available_space();
                for (sigil_i, crafted) in state.crafted_sigils.iter_mut().enumerate() {
                    ui.horizontal_top(|ui| {
                        Frame::new()
                            .stroke(stroke)
                            .inner_margin(4.0)
                            .show(ui, |ui| {
                                Grid::new(sigil_i).show(ui, |ui| {
                                    for (roll_i, roll_option) in
                                        crafted.rolls.iter_mut().enumerate()
                                    {
                                        if let Some(roll) = roll_option {
                                            let old = roll.skill;
                                            ComboBox::from_id_salt((sigil_i, roll_i))
                                                .width(150.0)
                                                .selected_text(roll.skill.name(self.language))
                                                .show_ui(ui, |ui| {
                                                    for possible_skill in &possible_skills {
                                                        if ui
                                                            .selectable_value(
                                                                &mut roll.skill,
                                                                *possible_skill,
                                                                possible_skill.name(self.language),
                                                            )
                                                            .clicked()
                                                        {
                                                            roll.points = 1;
                                                        }
                                                    }
                                                    ui.separator();
                                                    for skill in &recipe.not_blacklisted {
                                                        if ui
                                                            .selectable_value(
                                                                &mut roll.skill,
                                                                *skill,
                                                                skill.name(self.language),
                                                            )
                                                            .clicked()
                                                        {
                                                            roll.points = 1;
                                                        }
                                                    }
                                                });
                                            if roll.skill != old {
                                                calc = true;
                                            }
                                            if let Some(points_effect) =
                                                roll.skill.points_text(self.language, roll.points)
                                            {
                                                match points_effect {
                                                    SpecificEffect::None => {}
                                                    SpecificEffect::Named(name) => {
                                                        ComboBox::from_id_salt((
                                                            sigil_i,
                                                            roll_i,
                                                            "points name",
                                                        ))
                                                        .width(150.0)
                                                        .selected_text(name)
                                                        .show_ui(ui, |ui| {
                                                            for point in 1..13 {
                                                                if let Some(name) =
                                                                    points_named_effect(
                                                                        self.language,
                                                                        roll.skill,
                                                                        point,
                                                                    )
                                                                {
                                                                    ui.selectable_value(
                                                                        &mut roll.points,
                                                                        point,
                                                                        name,
                                                                    );
                                                                }
                                                            }
                                                        });
                                                    }
                                                    SpecificEffect::Points => {
                                                        let resp = ui.add(
                                                            DragValue::new(&mut roll.points)
                                                                .range(-10..=20)
                                                                .speed(0.02),
                                                        );
                                                        if resp.lost_focus() || resp.drag_stopped()
                                                        {
                                                            calc = true;
                                                        }
                                                    }
                                                }
                                            } else {
                                                ui.label("Invalid");
                                            }
                                        } else {
                                            if ComboBox::from_id_salt((sigil_i, roll_i))
                                                .width(150.0)
                                                .show_ui(ui, |_| {})
                                                .response
                                                .clicked()
                                            {
                                                let skill =
                                                    possible_skills.first().unwrap_or_else(|| {
                                                        recipe
                                                            .not_blacklisted
                                                            .first()
                                                            .expect("No possible skills on sigil")
                                                    });
                                                *roll_option = Some(OffsetSigilRoll::new(*skill));
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
                if ui.button("Add").clicked() {
                    state.crafted_sigils.push(OffsetSigil::new());
                }
            });
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
                (target.as_second() as u32).wrapping_sub_signed(self.desync_offset),
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
        if recipe.sigil_type == SigilType::Zenith {
            ui.horizontal(|ui| {
                ui.label("Zenith roll strategy:");
                ui.selectable_value(
                    &mut state.zenith_strategy,
                    ZenithStrategy::Resistance,
                    zenith_resistance_recipe_name(self.language),
                )
                .on_hover_text("Any Zenith Resistance sigil");
                ui.selectable_value(
                    &mut state.zenith_strategy,
                    ZenithStrategy::Heroics,
                    zenith_heroics_recipe_name(self.language),
                )
                .on_hover_text("Zenith Heroics sigil");
            });
        }
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
                                        if let Some(first_points) = req
                                            .skill
                                            .points_options(recipe, &self.sigil_data.skill_ranges)
                                            .first()
                                        {
                                            req.points = first_points.clone();
                                        } else {
                                            req.points = 0..=0;
                                        }
                                    }
                                });
                            let selected_text =
                                range_fmt_named(&req.points, req.skill, self.language)
                                    .unwrap_or_else(|| range_fmt(&req.points));
                            ComboBox::from_id_salt((req_i, 1))
                                .width(100.0)
                                .selected_text(selected_text)
                                .show_ui(ui, |ui| {
                                    let options = req
                                        .skill
                                        .points_options(recipe, &self.sigil_data.skill_ranges);
                                    for range in options {
                                        let text =
                                            range_fmt_named(&range, req.skill, self.language)
                                                .unwrap_or_else(|| range_fmt(&range));
                                        ui.selectable_value(&mut req.points, range.clone(), text);
                                    }
                                });
                            if ui.add(Button::new("❌").frame(false)).clicked() {
                                remove = Some(req_i);
                            }
                        });
                    }
                    if state.requirements.len() < 3 && ui.button("Add requirement").clicked() {
                        let roll = recipe
                            .normal_rolls
                            .first()
                            .map(|craft_roll| craft_roll.skill_roll.clone())
                            .unwrap_or_else(|| {
                                let skill = recipe
                                    .not_blacklisted
                                    .first()
                                    .expect("Recipe with no possible skills");
                                let ranges =
                                    &self.sigil_data.skill_ranges[*skill as usize].points_ranges;
                                let points = ranges
                                    .first()
                                    .map(|range| range.points_range.clone())
                                    .unwrap_or(0..=0);
                                SkillRoll {
                                    skill: *skill,
                                    points,
                                }
                            });
                        state.requirements.push(roll);
                    }
                });
        });
        if let Some(idx) = remove {
            state.requirements.remove(idx);
        }

        if ui.add_sized([100.0, 20.0], Button::new("Snipe")).clicked() {
            state.selected_result = None;
            let s = &state.search_settings;
            let seed_count = s.search_hours as usize * 60 * 60
                + s.search_minutes as usize * 60
                + s.search_seconds as usize;
            state.results = snipe(
                recipe,
                &self.sigil_data.skill_ranges,
                (self.time.as_second() as u32 + self.offset as u32)
                    .wrapping_sub_signed(self.desync_offset),
                seed_count,
                state.search_settings.search_depth,
                &Requirements::from_skill_rolls(&state.requirements),
                state.zenith_strategy,
            )
        }

        if !state.results.is_empty() {
            ui.separator();
        }
        ui.allocate_ui(Vec2::new(ui.available_width(), 100.0), |ui| {
            let stroke = ui.visuals().window_stroke;
            Frame::new()
                .stroke(stroke)
                .inner_margin(4.0)
                .show(ui, |ui| {
                    ui.take_available_space();
                    let Some(selected) = state.selected_result else {
                        return;
                    };
                    let result = &state.results[selected];

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Sequence:");
                            result.sequence.ui(ui, self.language);
                            ui.label(recipe_name(*recipe_idx, self.language));
                        });
                        ui.separator();
                        ui.vertical(|ui| {
                            ui.label(format!("Distance from seed: {}", result.advances));
                            CollapsingHeader::new("Early/late seeds").show(ui, |ui| {
                                for (early_i, early_result) in
                                    result.early_sigils.iter().enumerate().rev()
                                {
                                    Frame::new()
                                        .inner_margin(4.0)
                                        .stroke(ui.visuals().window_stroke)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(format!("-{}", early_i + 1));
                                                ui.vertical(|ui| {
                                                    for skill_roll in &early_result.sigil.rolls {
                                                        let s = format!(
                                                            "{} {}",
                                                            skill_roll.skill.name(self.language),
                                                            range_fmt_named(
                                                                &skill_roll.points,
                                                                skill_roll.skill,
                                                                self.language
                                                            )
                                                            .unwrap_or_else(|| range_fmt(
                                                                &skill_roll.points
                                                            ))
                                                        );
                                                        ui.label(s);
                                                    }
                                                });
                                                ui.separator();
                                                ui.label("Offset after:");
                                                if let Some(offset) = early_result.offset.ui(ui) {
                                                    self.offset = offset;
                                                }
                                            });
                                        });
                                }
                                for (late_i, late_result) in result.late_sigils.iter().enumerate() {
                                    Frame::new()
                                        .inner_margin(4.0)
                                        .stroke(ui.visuals().window_stroke)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(format!("+{}", late_i + 1));
                                                ui.vertical(|ui| {
                                                    for skill_roll in &late_result.sigil.rolls {
                                                        let s = format!(
                                                            "{} {}",
                                                            skill_roll.skill.name(self.language),
                                                            range_fmt_named(
                                                                &skill_roll.points,
                                                                skill_roll.skill,
                                                                self.language
                                                            )
                                                            .unwrap_or_else(|| range_fmt(
                                                                &skill_roll.points
                                                            ))
                                                        );
                                                        ui.label(s);
                                                    }
                                                });
                                                ui.separator();
                                                ui.label("Offset after:");
                                                if let Some(offset) = late_result.offset.ui(ui) {
                                                    self.offset = offset;
                                                }
                                            });
                                        });
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Offset after this sequence:");
                                if let Some(offset) = result.offset.ui(ui) {
                                    self.offset = offset;
                                }
                            });
                        });
                    });
                });
        });
        ScrollArea::vertical().show_rows(ui, 18.0, state.results.len(), |ui, row_range| {
            ui.take_available_space();
            for row in row_range {
                let result = &state.results[row];
                let with_offset =
                    (result.seed - self.offset as u32).wrapping_add_signed(self.desync_offset);
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
                let checked = if let Some(selected) = state.selected_result {
                    selected == row
                } else {
                    false
                };
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(checked, format!("{} | {}", with_offset, time_fmt))
                        .clicked()
                    {
                        state.selected_result = Some(row);
                    }
                    ui.colored_label(diff_color, diff_fmt);
                });
            }
        });
    }

    fn info(&mut self, ui: &mut Ui) {
        ui.heading("How to use");
        ui.label("0. Use diva song buffs and the Find desync tab to find any time desync with the server. If your system time is properly updated and you play on a reasonable server you can probably skip this.");
        ui.label("Regardless of overall time sync, you can expect the game's timestamp to be slightly behind your system's. It's recommended to feel out the timing with some 0 search depth snipes.");
        ui.label("1. Select a recipe for offset finding. This should be one of the 100% predictable recipes, and does not need to be the recipe you want to snipe.");
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("2. In-game open ");
            let resp = ui.link("the sigil recipe list");
            if resp.contains_pointer() {
                resp.show_tooltip_ui(|ui| {
                    ui.image(include_image!("../img/recipe_list.png"));
                });
            }
            ui.label(
                " and input the time you opened it in the Find offset tab. This must be done from ",
            );
            let resp =
                ui.link("the menu where you pick a sigil type to craft before the recipe list");
            if resp.contains_pointer() {
                resp.show_tooltip_ui(|ui| {
                    ui.image(include_image!("../img/before_recipe_list.png"));
                });
            }
            ui.label(", not after crafting a sigil.");
        });
        ui.label("3. Craft the selected sigil 1 at a time and input the result into the tool until an offset is found. Don't make a batch of more than 1 sigil.");
        ui.label("If no offset is found but all inputs are correct, your system clock is out of sync with the server's. Go to step 0.");
        ui.label(
            "The offset you find will persist until you make a sigil, through quests and logouts.",
        );
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("4. You can now choose a sigil to snipe. Enter what you want into the Snipe tab and press Snipe, then open ");
            let resp = ui.link("the sigil recipe list");
            if resp.contains_pointer() {
                resp.show_tooltip_ui(|ui| {
                    ui.image(include_image!("../img/recipe_list.png"));
                });
            }
            ui.label(" at a time given by one of the results. Again, this must be done from ");
            let resp =
                ui.link("the menu where you pick a sigil type.");
            if resp.contains_pointer() {
                resp.show_tooltip_ui(|ui| {
                    ui.image(include_image!("../img/before_recipe_list.png"));
                });
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("5. Craft sigils according to the sequence shown. \"10x - 2 times\" means craft ");
            let resp = ui.link("a batch of 10");
            if resp.contains_pointer() {
                resp.show_tooltip_ui(|ui| {
                    ui.image(include_image!("../img/rarity_batch.png"));
                });
            }
            ui.label(" twice. \"8x -> 1x -> 1x\" means craft an 8 batch, then a 1 batch, then a 1 batch. Do not back out of the menu in between batches.");
        });
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label(
                "5.1 Normal sigil: The sequence will tell you to use Rarity sigils. These are ",
            );
            let resp = ui.link("the white recipes at the end of the recipe list");
            if resp.contains_pointer() {
                resp.show_tooltip_ui(|ui| {
                    ui.image(include_image!("../img/rarity_batch.png"));
                });
            }
            ui.label(". You can buy the materials with RDP, GCP or N points.");
        });
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("5.2 Zenith sigil: The sequence will tell you to use ");
            let resp = ui.link("Resistance");
            if resp.contains_pointer() {
                resp.show_tooltip_ui(|ui| {
                    ui.image(include_image!("../img/zenith_resist.png"));
                });
            }
            ui.label(" and/or ");
            let resp = ui.link("Heroics");
            if resp.contains_pointer() {
                resp.show_tooltip_ui(|ui| {
                    ui.image(include_image!("../img/zenith_heroics.png"));
                });
            }
            ui.label(" sigils. Any Resistance recipe works.");
        });
        ui.label("6. Craft your snipe target. If you didn't get what you were sniping for, check the early/late seeds dropdown. If what you got is there, adjust your timing based on how early/late you were. If it's not there, there was some bigger issue.");
    }
}

fn range_fmt(range: &RangeInclusive<i16>) -> String {
    format!("{} to {}", range.start(), range.end())
}

fn range_fmt_named(
    range: &RangeInclusive<i16>,
    skill: SigilSkill,
    language: Language,
) -> Option<String> {
    let start = range.start();
    let start_name = points_named_effect(language, skill, *start)?;
    let end = range.end();
    if start == end {
        Some(start_name.to_string())
    } else {
        let end_name = points_named_effect(language, skill, *end).unwrap_or("?");
        Some(format!("{} - {}", start_name, end_name))
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn desynced_buff_end(state: &mut DesyncFinderState, desync: i32) {
        state.input_buff_start.second = state.target_second;
        let local_buff_start = state.input_buff_start.to_date_time();
        let buff_end = local_buff_start + 90.minutes() - desync.seconds();
        state.input_buff_end.year = buff_end.year();
        state.input_buff_end.month = buff_end.month();
        state.input_buff_end.day = buff_end.day();
        state.input_buff_end.hour = buff_end.hour();
        state.input_buff_end.minute = buff_end.minute();
    }

    #[test]
    fn test_desync() {
        let minutes_tests = [0, 3, 5, 6, 7, 8, 9, 15, 33, 90];
        let initial_timestamp = Timestamp::now();
        let time_zone = TimeZone::system();

        // ahead seconds
        let desync_minute = 60;
        for second in 1..60 {
            let desync = desync_minute + second;
            let mut state = DesyncFinderState::new(initial_timestamp, time_zone.clone());
            while state.result.is_none() {
                desynced_buff_end(&mut state, desync);
                state.calculate_desync();
            }
            assert_eq!(state.result.unwrap(), desync);
        }

        // ahead minutes
        let desync_seconds = 33;
        for minute in minutes_tests {
            let desync = minute + desync_seconds;
            let mut state = DesyncFinderState::new(initial_timestamp, time_zone.clone());
            while state.result.is_none() {
                desynced_buff_end(&mut state, desync);
                state.calculate_desync();
            }
            assert_eq!(state.result.unwrap(), desync)
        }

        // behind seconds
        let desync_minute = -60;
        for second in (-59..0).rev() {
            let desync = desync_minute + second;
            let mut state = DesyncFinderState::new(initial_timestamp, time_zone.clone());
            while state.result.is_none() {
                desynced_buff_end(&mut state, desync);
                state.calculate_desync();
            }
            assert_eq!(state.result.unwrap(), desync)
        }

        // behind minutes
        let desync_seconds = -33;
        for minute in minutes_tests {
            let desync = -minute + desync_seconds;
            let mut state = DesyncFinderState::new(initial_timestamp, time_zone.clone());
            while state.result.is_none() {
                desynced_buff_end(&mut state, desync);
                state.calculate_desync();
            }
            assert_eq!(state.result.unwrap(), desync)
        }
    }
}
