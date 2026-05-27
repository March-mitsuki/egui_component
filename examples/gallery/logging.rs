use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct LoggerGuard {
    _guard: WorkerGuard,
}

pub fn init_logging() -> LoggerGuard {
    let (stderr_nb, stderr_nb_guard) = tracing_appender::non_blocking(std::io::stderr());
    let layer = tracing_subscriber::fmt::layer()
        .with_writer(stderr_nb)
        .with_ansi(true);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(layer)
        .init();

    LoggerGuard {
        _guard: stderr_nb_guard,
    }
}
