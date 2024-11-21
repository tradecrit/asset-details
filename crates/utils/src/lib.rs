pub mod exponential_backoff;
pub mod env;
pub mod error;
pub mod parsers;
pub mod cache;

pub trait StripQuotes {
    fn strip_quotes(&self) -> String;
}

impl StripQuotes for String {
    fn strip_quotes(&self) -> String {
        self.trim_matches(|c| c == '\'' || c == '"').to_string()
    }
}
