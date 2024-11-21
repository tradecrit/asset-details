pub mod exponential_backoff;
pub mod env;
pub mod error;
pub mod parsers;
pub mod cache;

/// Trait to strip quotes from a string, used to normalize env var values
pub trait StripQuotes {
    fn strip_quotes(&self) -> String;
}

/// Implement the `StripQuotes` trait for `String`
impl StripQuotes for String {
    fn strip_quotes(&self) -> String {
        self.trim_matches(|c| c == '\'' || c == '"').to_string()
    }
}
