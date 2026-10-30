use stackpress_framework_session::Service;
fn main() {
    let mut session=Service::default().load("before");
    session.token="after".into();
}
