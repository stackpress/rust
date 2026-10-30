use stackpress_lib::Request;

fn main() {
    let request = Request::new();
    let _duplicate = request.clone();
}
