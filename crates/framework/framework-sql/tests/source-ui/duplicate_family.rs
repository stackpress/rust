//! One compiled family cannot select two Profile/Auth implementations at once.
#[path = "native/generated.rs"]
mod generated;
#[path = "../main/source_models/support.rs"]
mod support;
use stackpress_framework_sql::source::SessionModels;
struct Family;
impl SessionModels for Family {
    type Profile = generated::Profile;
    type Auth = generated::Auth;
}
impl SessionModels for Family {
    type Profile = generated::Profile;
    type Auth = generated::Auth;
}
fn main() {}
