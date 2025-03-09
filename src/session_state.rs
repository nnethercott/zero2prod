use std::future::{ready, Ready};

use actix_session::{Session, SessionExt, SessionGetError, SessionInsertError};
use actix_web::{dev::Payload, FromRequest, HttpRequest};
use uuid::Uuid;

pub struct TypedSession(Session);

//NOTE: this helps us avoid typos down the line & minimizes refactor pains
// becomes our Extractor

impl TypedSession{
    const USER_ID_KEY: &'static str = "user_id";

    pub fn renew(&self){
        self.0.renew();
    }
    pub fn insert_user_id(&self, user_id: Uuid)->Result<(), SessionInsertError>{
        self.0.insert(Self::USER_ID_KEY, user_id)
    }
    pub fn get_user_id(&self)->Result<Option<Uuid>, SessionGetError>{
        self.0.get(Self::USER_ID_KEY)
    }
}

// custom extractor implementation 
impl FromRequest for TypedSession{
    type Error = <Session as FromRequest>::Error;

    type Future = Ready<Result<TypedSession, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // NOTE: `get_session` defined in SessionExt trait from actix-session
        // and there they impl SessionExt for HttpRequest !!
        ready(Ok(TypedSession(req.get_session())))
    }
}
