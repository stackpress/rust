use stackpress_framework_session::{columns::Record,storage::{ProfileActions,StoreQuery}};
use stackpress_framework_sql::DataSession;
fn overlap(actions:&ProfileActions,db:&mut DataSession<'_>){
    let input=Record::new();
    let query=StoreQuery::default();
    let first=actions.create_in(db,&input);
    let second=actions.find_in(db,&query);
    drop((first,second));
}
fn main(){}
