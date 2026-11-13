#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{
    egui::{Context, FontData, FontFamily, Theme},
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

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<()> {
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
            ctx.egui_ctx.set_theme(Theme::Dark);
            ctx.egui_ctx
                .all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);
            add_font(&ctx.egui_ctx);
            egui_extras::install_image_loaders(&ctx.egui_ctx);
            Ok(Box::new(UiState::new(sigil_data)))
        }),
    )
    .map_err(|e| anyhow!("{e}"))?;

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
                    ctx.egui_ctx.set_theme(Theme::Dark);
                    ctx.egui_ctx
                        .all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);
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
