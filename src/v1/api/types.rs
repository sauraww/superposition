use std::{
    future::{ready, Ready},
    println,
};

use actix_web::{error, web::Data, Error, FromRequest};

use crate::db::utils::AppState;

#[derive(Clone)]
pub struct AuthenticationInfo(pub String);
impl FromRequest for AuthenticationInfo {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let opt_token = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| {
                if h.starts_with("Bearer") {
                    Some(h)
                } else {
                    None
                }
            })
            .and_then(|h| {
                h.split(' ')
                    .collect::<Vec<_>>()
                    .get(1)
                    .map(|token| token.to_string())
            });
        dbg!(format!("Token is \"{:?}\"", opt_token));
        let opt_admin_token = req
            .app_data()
            .map(|d: &Data<AppState>| d.admin_token.as_str());

        let result = match (opt_token, opt_admin_token) {
            (_, None) => {
                println!("ERROR: ADMIN TOKEN NOT FOUND!!!!");
                Err(error::ErrorInternalServerError(""))
            }
            (None, _) => Err(error::ErrorUnauthorized("Bearer token required.")),
            (Some(token), Some(admin_token)) if token != admin_token => {
                Err(error::ErrorUnauthorized(""))
            }
            (Some(_token), Some(_admin_token)) => {
                let email = "cac.admin@juspay.in";
                let auth_info = AuthenticationInfo(email.to_string());
                Ok(auth_info)
            }
        };
        ready(result)
    }
}