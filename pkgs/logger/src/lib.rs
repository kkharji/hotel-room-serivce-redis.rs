use std::borrow::Cow;

use fern::colors::Color;
use fern::colors::ColoredLevelConfig;

/// Initialize logger
pub fn init(pname: &str) {
    let pname = Cow::from(pname.to_string());
    let verbosity = match std::env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "trace".into())
        .as_str()
    {
        "warn" => 0,
        "info" => 1,
        "debug" => 2,
        "trace" => 3,
        "full" => 4,
        v => unreachable!("{v} is not supported"),
    };

    let levels = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Blue)
        .debug(Color::Magenta)
        .trace(Color::White);

    let mut logger = fern::Dispatch::new();

    logger = logger.format(move |out, message, record| {
        out.finish(format_args!(
            "{b}{time}{r} {l}{kind:<5}{r} {c}{name}{r} {l}{message}{r}",
            l = format_args!("\x1B[{}m", levels.get_color(&record.level()).to_fg_str()),
            b = format_args!("\x1B[{}m", Color::BrightBlack.to_fg_str()),
            c = format_args!("\x1B[{}m", Color::Cyan.to_fg_str()),
            r = "\x1B[0m",
            time = chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
            kind = record.level(),
            name = record.target(),
            message = message,
        ))
    });

    logger = match verbosity {
        0 => logger.level_for(pname, log::LevelFilter::Warn),
        1 => logger.level_for(pname, log::LevelFilter::Info),
        2 => logger.level_for(pname, log::LevelFilter::Debug),
        _ => logger.level_for(pname, log::LevelFilter::Trace),
    };

    logger = match verbosity {
        3 => logger.level(log::LevelFilter::Info),
        4 => logger.level(log::LevelFilter::Trace),
        _ => logger.level(log::LevelFilter::Error),
    };

    logger = logger.chain(std::io::stderr());

    logger.apply().unwrap();
}
