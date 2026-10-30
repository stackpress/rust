use stackpress_framework_session::Clock;
use std::rc::Rc;
struct Local(Rc<()>);
impl Clock for Local { fn now(&self)->f64 { 0.0 } }
fn main() {}
