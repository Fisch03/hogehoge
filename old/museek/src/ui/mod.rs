mod menu_bar;
pub use menu_bar::menu_bar;
//
// mod library;
// pub use library::library;
// mod explore;
// pub use explore::explore;
// mod social;
// pub use social::social;
//
// mod downloads;
// pub use downloads::downloads;
// mod settings;
// pub use settings::settings;

#[derive(Default)]
pub struct State {
    search_open: bool,
    search_query: String,
}

pub fn now_playing(ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("now_playing")
        .resizable(false)
        .exact_height(100.0)
        .show(ctx, |_| {});
}

// pub fn mode_select(ctx: &Context, mode: &mut Mode) {
//     egui::SidePanel::left("mode_select")
//         .resizable(false)
//         .frame(Frame {
//             inner_margin: Margin {
//                 left: 0.0,
//                 right: 0.0,
//                 top: 0.0,
//                 bottom: 0.0,
//             },
//             ..Default::default()
//         })
//         .exact_width(48.0)
//         .show(ctx, |ui| {
//             ui.scope(|ui| {
//                 ui.spacing_mut().item_spacing = Vec2::splat(0.0);
//                 ui.spacing_mut().button_padding = Vec2::splat(8.0);
//
//                 macro_rules! mode_button {
//                     ($ui:ident, $mode:path, $hover:expr, $icon:expr) => {
//                         if $ui
//                             .selectable_label(
//                                 matches!(mode, $mode),
//                                 RichText::new(format!("{}", $icon)).size(24.0),
//                             )
//                             .on_hover_text($hover)
//                             .clicked()
//                         {
//                             *mode = $mode;
//                         }
//                         $ui.add(egui::Separator::default().spacing(0.0));
//                     };
//                 }
//
//                 ui.with_layout(
//                     Layout::top_down(Align::Center).with_cross_justify(true),
//                     |ui| {
//                         mode_button!(ui, Mode::Library, "Library", icon::MUSIC_NOTES);
//                         mode_button!(ui, Mode::Explore, "Explore", icon::MAGNIFYING_GLASS);
//                         mode_button!(ui, Mode::Social, "Social", icon::USERS);
//                     },
//                 );
//
//                 ui.with_layout(
//                     Layout::bottom_up(Align::Center).with_cross_justify(true),
//                     |ui| {
//                         mode_button!(ui, Mode::Settings, "Settings", icon::GEAR);
//                         mode_button!(ui, Mode::Downloads, "Downloads", icon::DOWNLOAD_SIMPLE);
//                     },
//                 );
//             });
//         });
// }
