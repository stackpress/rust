//! An app's integer Profile.id cannot satisfy the source session role.
#[path = "../main/source_models/support.rs"]
mod support;
use stackpress_framework_sql::source::{AuthRole, ProfileRole, SessionModels};
struct Family;
impl SessionModels for Family {
    type Profile = support::Model<support::CompileSpec<i64>, ProfileRole>;
    type Auth = support::Model<support::CompileSpec<String>, AuthRole>;
}
fn main() {}
