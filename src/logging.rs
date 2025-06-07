use std::time::Instant;
use tracing::{Level, Subscriber};
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    prelude::*,
    registry::LookupSpan,
};

struct Format {
    start_time: Instant,
}

impl Format {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }
}

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
        use ansi_term::{Color, Style};

        let meta = event.metadata();

        let elapsed = self.start_time.elapsed();

        let style = Style::new().dimmed();
        write!(
            writer,
            "{}{:06}.{:03}{}",
            style.prefix(),
            elapsed.as_secs(),
            elapsed.subsec_millis(),
            style.suffix()
        )?;

        write!(
            writer,
            " [{}] ",
            match *meta.level() {
                Level::ERROR => Color::Red.paint("E"),
                Level::WARN => Color::Yellow.paint("!"),
                Level::INFO => Color::Green.paint("*"),
                Level::DEBUG => Color::Blue.paint("D"),
                Level::TRACE => Color::Purple.paint("T"),
            }
        )?;

        let style = Style::new().bold().italic();
        write!(
            writer,
            "{}{}{} ",
            style.prefix(),
            meta.target(),
            style.suffix()
        )?;

        ctx.format_fields(writer.by_ref(), event)?;

        writeln!(writer)?;

        Ok(())
    }
}

pub fn init() {
    let fmt_layer = tracing_subscriber::fmt::layer().event_format(Format::new());

    let fmt_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()
        .expect("Failed to build filter")
        .add_directive("freya_core=warn".parse().unwrap())
        .add_directive("freya_winit=warn".parse().unwrap())
        .add_directive("torin=warn".parse().unwrap())
        .add_directive("lofty=info".parse().unwrap());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(fmt_filter)
        .init();

    // tracing_subscriber::fmt().compact().init();
}
