//! Safe test-only diagnostics shared by both native M2 runners.
/// Print only sanitized first-party context and structured native categories.
pub(super) fn report_error(error: &(dyn std::error::Error + 'static)) {
    // Walk a bounded cause chain, formatting only safe first-party contexts and typed codes.
    let mut current = Some(error);
    for _ in 0..8 {
        let Some(error) = current else { break };
        if let Some(error) = error.downcast_ref::<stackpress_framework_sql::DataError>() {
            eprintln!("data failure category {:?}: {error}", error.kind());
        }
        if let Some(error) = error.downcast_ref::<stackpress_sql::SqlError>() {
            eprintln!("SQL failure category {:?}: {error}", error.kind());
        }
        if let Some(native) = error.downcast_ref::<sqlx::Error>() {
            match native {
                sqlx::Error::Database(database) => {
                    eprintln!("native structured code: {:?}", database.code())
                }
                sqlx::Error::Io(io) => eprintln!("native IO category: {:?}", io.kind()),
                sqlx::Error::ColumnDecode { .. } | sqlx::Error::Decode(_) => {
                    eprintln!("native decode failure")
                }
                _ => eprintln!("native failure (detail redacted)"),
            }
        }
        current = error.source();
    }
}
