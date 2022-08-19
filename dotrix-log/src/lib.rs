pub use log::{debug, error, info, log, log_enabled, trace, warn, Level, LevelFilter};
use once_cell::sync::OnceCell;

static LOG: OnceCell<Log> = OnceCell::new();

pub struct Log {
    pub instant: std::time::Instant,
    pub targets: Vec<(String, LevelFilter)>,
}

impl Default for Log {
    fn default() -> Self {
        Self {
            instant: std::time::Instant::now(),
            targets: vec![
                (String::from("wgpu"), LevelFilter::Warn),
                (String::from("dotrix"), LevelFilter::Info),
                (String::from(""), LevelFilter::Debug),
            ],
        }
    }
}

impl log::Log for Log {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        let target = metadata.target();
        let level = metadata.level();
        LOG.get()
            .map(|logger| {
                for (target_filter, level_filter) in logger.targets.iter() {
                    if target.starts_with(target_filter) {
                        return level.to_level_filter() <= *level_filter;
                    }
                }
                false
            })
            .unwrap_or(false)
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let elapsed = LOG
                .get()
                .map(|l| l.instant.elapsed().as_secs_f64() / 1000.0)
                .unwrap_or(0.0);
            let level_mark = match record.level() {
                log::Level::Error => "!!",
                log::Level::Warn => "!~",
                log::Level::Info => "--",
                log::Level::Debug => "**",
                log::Level::Trace => "->",
            };
            println!(
                "{:.4} {} {} - {}",
                elapsed,
                level_mark,
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

pub fn subscribe(logger: Log) {
    let mut max_level = LevelFilter::Off;

    for (_, level_filter) in logger.targets.iter() {
        if *level_filter > max_level {
            max_level = *level_filter;
        }
    }

    if LOG.set(logger).is_ok() {
        log::set_logger(LOG.get().unwrap())
            .map(|()| log::set_max_level(max_level))
            .expect("Other log subscription already exists");
    } else {
        panic!("Log subscription must be initiated only once");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
