use std::collections::HashMap;

use logging::Colorize;
use logging::LogLevel;

struct EngineLogTarget;

impl logging::target::Target for EngineLogTarget {
    fn write(&self, level: LogLevel, message: &str) -> Result<(), logging::LogError> {
        let mut logs = globals::logs::get().write();

        logs.push((level, message.to_string()));

        Ok(())
    }
}

struct EngineLogFormatter;

impl logging::Formatter for EngineLogFormatter {
    fn format(&self, record: &logging::Record) -> String {
        let level = format!("[{}]", record.level).color(record.level.console_color());
        let padding = " ".repeat(LogLevel::MAX_WIDTH - record.level.to_string().len());
        let level = format!("{}{}", level, padding);

        if record.context.is_empty() {
            format!(
                "{} {} {}",
                record
                    .date
                    .format("[%Y/%m/%d %H:%M:%S]")
                    .to_string()
                    .color(logging::Color::BrightCyan),
                level,
                record.message
            )
        } else {
            let ctx = format_context(&record.context);
            let ctx_formatted = format!("{{ {} }}", ctx);

            format!(
                "{} {} {} {}",
                record
                    .date
                    .format("[%Y/%m/%d %H:%M:%S]")
                    .to_string()
                    .color(logging::Color::BrightCyan),
                level,
                ctx_formatted,
                record.message
            )
        }
    }
}

pub fn format_context(ctx: &HashMap<String, String>) -> String {
    if ctx.is_empty() {
        return String::new();
    }

    let mut pairs: Vec<_> = ctx.iter().collect();
    pairs.sort_by_key(|(k, _)| *k);

    pairs
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn init_logging() {
    logging::init(logging::Config::new(
        LogLevel::Info,
        vec![
            logging::TargetConfig {
                target: Box::new(logging::target::Console),
                formatter: Some(Box::new(EngineLogFormatter)),
            },
            logging::TargetConfig {
                target: Box::new(EngineLogTarget),
                formatter: None,
            },
        ],
        Box::new(EngineLogFormatter),
    ));
}
