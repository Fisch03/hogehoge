use eframe::{CreationContext, Frame};
use egui::Context;
use tracing::error;

pub mod library;
use library::Library;
pub mod background_task;
use background_task::{BackgroundTaskLogs, BackgroundTaskManager};
mod ui;

pub struct App {
    // current_mode: Mode,
    ui_state: ui::State,
    library: Library,
    bg_task: BackgroundTaskManager,
}
// enum Mode {
//     Library,
//     Explore,
//     Social,
//
//     Downloads,
//     Settings,
// }

impl App {
    pub fn new(cc: &CreationContext, bg_task_logs: BackgroundTaskLogs) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Bold);
        cc.egui_ctx.set_fonts(fonts);

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let library = match rt.block_on(Library::new()) {
            Ok(library) => library,
            Err(e) => {
                error!("Failed to initialize library: {}", e);
                std::process::exit(1);
            }
        };

        let mut bg_task = BackgroundTaskManager::new(rt, bg_task_logs);

        {
            let library = library.clone();
            bg_task.spawn("update_library", true, async move {
                library.update().await;

                Ok(())
            });
        }

        Self {
            library,
            ui_state: ui::State::default(),
            bg_task,
            // current_mode: Mode::Library,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.bg_task.update();

        ui::menu_bar(ctx, &mut self.ui_state);

        ui::now_playing(ctx);

        // ui::mode_select(ctx, &mut self.current_mode);

        egui::CentralPanel::default().show(ctx, |ui| {
            // match self.current_mode {
            //     Mode::Library => ui::library(ctx),
            //     Mode::Explore => ui::explore(ctx),
            //     Mode::Social => ui::social(ctx),
            //
            //     Mode::Downloads => ui::downloads(ctx),
            //     Mode::Settings => ui::settings(ctx),
            // }
        });
    }
}
