use std::rc::Rc;
use stackpress_sql::SqlFuture;

fn local_future() -> SqlFuture<'static, ()> {
    Box::pin(async {
        let local = Rc::new(7);
        tokio::task::yield_now().await;
        assert_eq!(*local, 7);
        Ok(())
    })
}

fn main() {}
