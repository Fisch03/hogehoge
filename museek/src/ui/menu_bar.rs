use super::State;
use egui::{
    vec2, Align2, Context, Frame, Grid, Margin, ScrollArea, TextEdit, TopBottomPanel, Window,
};
use egui_phosphor::bold as icon;

pub fn menu_bar(ctx: &Context, state: &mut State) {
    TopBottomPanel::top("menu_bar")
        .frame(Frame {
            inner_margin: Margin::same(6.0),
            fill: ctx.style().visuals.panel_fill,
            ..Default::default()
        })
        // .resizable(false)
        .show(ctx, |ui| {
            let bg_rect = ui.ctx().screen_rect();

            ui.vertical_centered(|ui| {
                let r = ui.add(
                    TextEdit::singleline(&mut state.search_query)
                        .hint_text(format!("{} Search", icon::MAGNIFYING_GLASS))
                        .desired_width(bg_rect.width() * 0.3),
                );

                state.search_open = r.has_focus();
            });

            if state.search_open {
                Window::new("search_results")
                    .title_bar(false)
                    .anchor(Align2::CENTER_TOP, vec2(0.0, 50.0))
                    .show(ctx, |ui| {
                        // let mut backdrop = ui.new_child(UiBuilder::new().max_rect(bg_rect));
                        // backdrop.set_min_size(bg_rect.size());
                        // ui.painter()
                        //     .rect_filled(bg_rect, 0.0, Color32::from_black_alpha(100));

                        ScrollArea::vertical().show(ui, |ui| {
                            Grid::new("search_results_grid")
                                .striped(true)
                                .min_col_width(bg_rect.width() * 0.8)
                                .show(ui, |ui| {
                                    lipsum::LOREM_IPSUM.split(' ').for_each(|word| {
                                        ui.label(word);
                                        ui.end_row();
                                    });
                                });
                        });
                    });
            }
        });
}
