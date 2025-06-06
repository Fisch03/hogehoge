use eframe::NativeOptions;
use egui::ViewportBuilder;
use tracing::{Level, Subscriber};
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields},
    prelude::*,
    registry::LookupSpan,
};

use museek::{background_task::BackgroundTaskLogSubscriber as BackgroundTaskLogLayer, App};

struct Format;
impl<S, N> FormatEvent<S, N> for Format
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        use nu_ansi_term::Color;

        let meta = event.metadata();

        write!(
            writer,
            "[{}] ",
            match *meta.level() {
                Level::ERROR => Color::Red.paint("E"),
                Level::WARN => Color::Yellow.paint("!"),
                Level::INFO => Color::Green.paint("*"),
                Level::DEBUG => Color::Blue.paint("D"),
                Level::TRACE => Color::Purple.paint("T"),
            }
        )?;

        ctx.format_fields(writer.by_ref(), event)?;

        writeln!(writer)?;

        Ok(())
    }
}

fn main() -> eframe::Result {
    let fmt_layer = tracing_subscriber::fmt::layer().event_format(Format);

    let bg_task_log_layer = BackgroundTaskLogLayer::new();
    let bg_task_logs = bg_task_log_layer.get_logs();

    let global_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()
        .expect("Failed to build filter")
        .add_directive("lofty=off".parse().unwrap());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(global_filter)
        .with(bg_task_log_layer)
        .init();

    // tracing_subscriber::fmt().compact().init();

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default().with_min_inner_size([320.0, 280.0]),
        ..Default::default()
    };

    eframe::run_native(
        "museek!",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, bg_task_logs)))),
    )
}
