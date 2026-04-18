#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{Result, anyhow};
use eframe::{
    egui::{Context, FontData, FontFamily, IconData, Theme, ViewportBuilder},
    epaint::text::{FontInsert, FontPriority, InsertFontFamily},
};

use crate::{dat::SigilData, ui::UiState};

mod dat;
mod prng;
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

fn main() -> Result<()> {
    let sigil_data = SigilData::load()?;

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_icon(IconData::default())
            .with_inner_size([700.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Sigil Sniper",
        options,
        Box::new(|ctx| {
            ctx.egui_ctx.set_theme(Theme::Dark);
            add_font(&ctx.egui_ctx);
            Ok(Box::new(UiState::new(sigil_data)))
        }),
    )
    .map_err(|e| anyhow!("{e}"))?;

    Ok(())
}
