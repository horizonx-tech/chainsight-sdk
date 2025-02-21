use anyhow::Error;
use std::fmt::Display;

pub trait Logger: Clone {
    fn info(&self, s: &str);
    fn err(&self, e: &Error);
    fn err_with_msg(&self, e: &Error, msg: &str);
}

pub enum LogLevel {
    Info,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}
