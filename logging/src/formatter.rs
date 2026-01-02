use crate::{LogLevel, Record};
use std::collections::HashMap;

pub trait Formatter {
    fn format(&self, record: &Record) -> String;
}

pub struct DefaultFormatter;

impl Formatter for DefaultFormatter {
    fn format(&self, record: &Record) -> String {
        let level = format!("[{}]", record.level);
        let padding = " ".repeat(LogLevel::MAX_WIDTH - record.level.to_string().len());
        let level = format!("{}{}", level, padding);

        let ctx = format_context(&record.context);

        if record.context.is_empty() {
            format!("{} {}", level, record.message)
        } else {
            let ctx_formatted = format!("{{ {} }}", ctx);

            format!("{} {} {}", level, ctx_formatted, record.message)
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
