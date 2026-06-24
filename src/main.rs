#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod config;
mod export;
mod overlay;
mod shape;
mod theme;

use clap::Parser;
use config::{AppConfig, ThemeMode};
use overlay::ShotrsApp;

#[derive(Debug, Parser)]
#[command(name = "shotrs", version, about = "Small cross-platform screenshot tool")]
struct Cli {
    /// Override theme: light, dark or system
    #[arg(long, value_parser = parse_theme)]
    theme: Option<ThemeMode>,
}

fn parse_theme(s: &str) -> Result<ThemeMode, String> {
    ThemeMode::parse(s).ok_or_else(|| "theme must be light, dark or system".to_owned())
}

impl eframe::App for ShotrsApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ShotrsApp::ui(self, ui, frame);
    }
}

fn main() -> eframe::Result {
    let cli = Cli::parse();
    let mut config = AppConfig::load();
    config.apply_cli_theme_override(cli.theme);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Shotrs")
            .with_inner_size([420.0, 560.0])
            .with_min_inner_size([360.0, 420.0])
            .with_resizable(false),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Shotrs",
        options,
        Box::new(move |_cc| Ok(Box::new(ShotrsApp::new(config)))),
    )
}
