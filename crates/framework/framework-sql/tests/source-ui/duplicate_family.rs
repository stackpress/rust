//! One compiled family cannot select two Profile/Auth implementations at once.
#[path = "../main/source_models/support.rs"]
mod support;
use stackpress_framework_sql::source::{AuthRole, ProfileRole, SessionModels};
struct Family;
impl SessionModels for Family {
    type Profile = support::Model<support::CompileSpec<String>, ProfileRole>;
    type Auth = support::Model<support::CompileSpec<String>, AuthRole>;
}
impl SessionModels for Family {
    type Profile = support::Model<support::CompileSpec<String>, ProfileRole>;
    type Auth = support::Model<support::CompileSpec<String>, AuthRole>;
}
fn main() {}
