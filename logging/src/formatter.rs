use crate::Record;

pub trait Formatter {
    fn format(&self, record: &Record) -> String;
}

pub struct DefaultFormatter;

impl Formatter for DefaultFormatter {
    fn format(&self, record: &Record) -> String {
        format!("[{}]: {}", record.level, record.message)
    }
}
