//! Generated source model family; no plugin dependency or hidden external table.
/// The application family has exactly one Profile definition shared with API.
pub struct Models;
[%#?:session%]
impl stackpress_framework_sql::source::SessionModels for Models {
    type Profile=super::[%profile%]::ProfileModel;
    type Auth=super::[%auth%]::AuthModel;
}
[%/?:session%]
[%#?:api%]
impl stackpress_framework_sql::source::ApiModels for Models {
    type Application=super::[%application%]::ApplicationModel;
    type Session=super::[%session_module%]::SessionModel;
}
[%/?:api%]
