use stackpress_framework_session::{Service,Session};
fn main() {
    let original=Service::default().load("token");
    let duplicate:Session=original.clone();
}
