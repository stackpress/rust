use generated_workspace_app::generated::{contracts::profile::CreateProfile, stores::profile::Profiles};
use stackpress_framework_sql::DataSession;
async fn alias(mut session: DataSession<'_>) {
    let pending = Profiles::create(&mut session, CreateProfile::default());
    let other = Profiles::get(&mut session, "u");
    pending.await.unwrap();
    other.await.unwrap();
}
fn main() {}
