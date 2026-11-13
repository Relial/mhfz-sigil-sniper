#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{
    egui::{
        Color32, Context, CornerRadius, FontData, FontFamily, Spacing, Stroke, Style, Visuals,
        style::{Interaction, WidgetVisuals, Widgets},
        vec2,
    },
    epaint::text::{FontInsert, FontPriority, InsertFontFamily},
};

use crate::{dat::SigilData, ui::UiState};

mod dat;
mod rng;
mod sigil;
mod ui;

fn add_font(ctx: &Context) {
    ctx.add_font(FontInsert::new(
        "NotoSansJP-Regular",
        FontData::from_static(include_bytes!("../font/NotoSansJP-Regular.ttf")),
        vec![
            InsertFontFamily {
                family: FontFamily::Proportional,
                priority: FontPriority::Lowest,
            },
            InsertFontFamily {
                family: FontFamily::Monospace,
                priority: FontPriority::Lowest,
            },
        ],
    ));
}

fn get_style() -> Style {
    Style {
        animation_time: 0.1,
        interaction: Interaction {
            tooltip_delay: 0.0,
            ..Default::default()
        },
        spacing: Spacing {
            button_padding: vec2(4.0, 1.0),
            interact_size: vec2(40.0, 18.0),
            ..Default::default()
        },
        visuals: Visuals {
            widgets: Widgets {
                noninteractive: WidgetVisuals {
                    weak_bg_fill: Color32::from_gray(27),
                    bg_fill: Color32::from_gray(27),
                    bg_stroke: Stroke::new(1.0, Color32::from_gray(60)), // separators, indentation lines
                    fg_stroke: Stroke::new(1.0, Color32::from_gray(140)), // normal text color
                    corner_radius: CornerRadius::same(2),
                    expansion: 0.0,
                },
                inactive: WidgetVisuals {
                    weak_bg_fill: Color32::from_gray(60), // button background
                    bg_fill: Color32::from_gray(60),      // checkbox background
                    bg_stroke: Default::default(),
                    fg_stroke: Stroke::new(1.0, Color32::from_gray(180)), // button text
                    corner_radius: CornerRadius::same(2),
                    expansion: 0.0,
                },
                hovered: WidgetVisuals {
                    weak_bg_fill: Color32::from_gray(70),
                    bg_fill: Color32::from_gray(70),
                    bg_stroke: Stroke::new(1.0, Color32::from_gray(150)), // e.g. hover over window edge or button
                    fg_stroke: Stroke::new(1.5, Color32::from_gray(240)),
                    corner_radius: CornerRadius::same(3),
                    expansion: 0.0,
                },
                active: WidgetVisuals {
                    weak_bg_fill: Color32::from_gray(55),
                    bg_fill: Color32::from_gray(55),
                    bg_stroke: Stroke::new(1.0, Color32::WHITE),
                    fg_stroke: Stroke::new(2.0, Color32::WHITE),
                    corner_radius: CornerRadius::same(2),
                    expansion: 0.0,
                },
                open: WidgetVisuals {
                    weak_bg_fill: Color32::from_gray(45),
                    bg_fill: Color32::from_gray(27),
                    bg_stroke: Stroke::new(1.0, Color32::from_gray(60)),
                    fg_stroke: Stroke::new(1.0, Color32::from_gray(210)),
                    corner_radius: CornerRadius::same(2),
                    expansion: 0.0,
                },
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    use eframe::egui::{IconData, ViewportBuilder};

    let sigil_data = SigilData::load()?;

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_icon(IconData::default())
            .with_inner_size([800.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Sigil Sniper",
        options,
        Box::new(|ctx| {
            ctx.egui_ctx.set_global_style(get_style());
            add_font(&ctx.egui_ctx);
            egui_extras::install_image_loaders(&ctx.egui_ctx);
            Ok(Box::new(UiState::new(sigil_data)))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    let sigil_data = SigilData::load().expect("Failed to load sigil data");

    let options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                options,
                Box::new(|ctx| {
                    ctx.egui_ctx.set_global_style(get_style());
                    add_font(&ctx.egui_ctx);
                    egui_extras::install_image_loaders(&ctx.egui_ctx);
                    Ok(Box::new(UiState::new(sigil_data)))
                }),
            )
            .await;

        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(()) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html("<p> The app has crashed.</p>");
                    panic!("Failed to start eframe: {e:#?}");
                }
            }
        }
    });
}
