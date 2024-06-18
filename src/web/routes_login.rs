use crate::{web, Error, Result};
use axum::{ Json, Router};
use serde_json::{json, Value};
use serde::Deserialize;
use axum::routing::post;
use tower_cookies::{Cookie, Cookies};

pub fn routes() -> Router {
    Router::new()
        .route("/api/login", post(api_login))
}


async fn api_login(cookies: Cookies ,payload: Json<LoginPayload>) ->Result<Json<Value>> {
    println!("->> {:<12} - API_LOGIN", "HANDLER");

    // TODO: implement login logic
    if payload.username !="demo1" || payload.pwd != "welcome" {
        return Err(Error::LoginFail);
    } 
    //TODO: Implement auth-token generation logic
   cookies.add(Cookie::new(web::AUTH_TOKEN, "user-1.exp.sign"));

    // Create the Success body
    let body= Json(json!({
        "result":{
            "success": true,
        }

    }));

    Ok(body)

}

#[derive(Debug, Deserialize)]
struct LoginPayload{
    username: String,
    pwd: String,
}