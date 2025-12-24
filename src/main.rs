use eframe::egui;
use dev_launcher::app::DevLauncher;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_title("Dev Stack Launcher"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Dev Stack Launcher",
        options,
        Box::new(|cc| Ok(Box::new(DevLauncher::new(cc)))),
    )
}