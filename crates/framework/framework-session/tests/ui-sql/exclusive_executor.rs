#[path = "../fixtures/models/mod.rs"] #[allow(dead_code, missing_docs, unexpected_cfgs)] mod models; use models::stores::profile::ProfileActions; use stackpress_framework_session::{columns::Record,storage::StoreQuery};
use stackpress_framework_sql::DataSession;
fn overlap(actions:&ProfileActions,db:&mut DataSession<'_>){
    let input=Record::new();
    let query=StoreQuery::default();
    let first=actions.create_in(db,&input);
    let second=actions.find_in(db,&query);
    drop((first,second));
}
fn main(){}
