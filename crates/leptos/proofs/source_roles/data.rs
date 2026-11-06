//! Serialize the actual compiled Rust records for a lossless comparison with read-only Frui data.
use stackpress_framework_schema::__private::serde_json;
use stackpress_leptos::components::source::{COLORS, COUNTRIES, CURRENCIES};
/// No source text inspection substitutes for serializing the compiled data types.
fn main() {
    println!(
        "{}",
        serde_json::json!({"countries":COUNTRIES,"currencies":CURRENCIES,"colors":COLORS})
    );
}
