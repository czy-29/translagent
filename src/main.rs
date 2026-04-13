use snafu::Whatever;
use tracing_subscriber::fmt::{format::FmtSpan, time::ChronoLocal};

#[snafu::report]
#[tokio::main]
async fn main() -> Result<(), Whatever> {
    init_tracing_with_ansi();
    tracing::info!("Hello, world!");
    Ok(())
}

// 参见：https://github.com/tokio-rs/tracing/issues/3068
fn init_tracing_with_ansi() {
    #[cfg(windows)]
    nu_ansi_term::enable_ansi_support().ok();
    init_tracing();
}

#[cfg(debug_assertions)]
fn init_tracing() {
    tracing_subscriber::fmt()
        .with_timer(ChronoLocal::new("%m-%d %H:%M:%S".into()))
        .with_max_level(tracing::Level::DEBUG)
        .with_span_events(FmtSpan::FULL)
        .with_thread_names(true)
        .init();
}

#[cfg(not(debug_assertions))]
fn init_tracing() {
    tracing_subscriber::fmt()
        .with_timer(ChronoLocal::new("%m-%d %H:%M:%S".into()))
        .with_span_events(FmtSpan::FULL)
        .with_thread_names(true)
        .init();
}
