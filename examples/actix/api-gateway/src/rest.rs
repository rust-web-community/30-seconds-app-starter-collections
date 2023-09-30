use actix_http::header::AUTHORIZATION;
use actix_web::{
    web,
    get,
    web::ServiceConfig,
    http::header,
    HttpResponse, Responder, HttpRequest, cookie::{Cookie, time::{OffsetDateTime, Duration}, SameSite},
};

use coi_actix_web::inject;
use crate::store_interface::UserRepository;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use jwt::{SignWithKey, ToBase64, VerifyWithKey};
use std::collections::BTreeMap;
use uuid::Uuid;

pub(super) fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        route_config(config)
    }
}
pub fn route_config(config: &mut ServiceConfig) {
    config.service(
        web::scope("/hello/") // Routes should be configuration driven
            .route("", web::get().to(get_proxy))
    ).service(health)
    .service(web::scope("/public")
            .route("sign-up", web::get().to(sign_up)));
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

async fn get_proxy(req: HttpRequest) -> impl Responder {
    let req_auth = req.cookie("session");
    if req_auth.is_none(){
        return HttpResponse::Forbidden().body("Forbidden");
    }
    let key = Hmac::<Sha256>::new_from_slice(b"some-key").unwrap();
    println!("{}", req_auth.clone().unwrap().value().to_base64().unwrap());
    let claims: BTreeMap<String, String> = req_auth.unwrap().value().verify_with_key(&key).unwrap();

    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(AUTHORIZATION, reqwest::header::HeaderValue::from_str(claims.get("sub").unwrap()).unwrap());
    let response = client.get(format!("http://hello_service:8080/hello")).headers(headers).send().await.unwrap();
    HttpResponse::Ok().body(response.bytes().await.unwrap())
}

#[inject]
async fn sign_up(#[inject] repository: Arc<dyn UserRepository>) -> impl Responder {
    let key = Hmac::<Sha256>::new_from_slice(b"some-key").unwrap();
    let mut claims = BTreeMap::new();
    let new_id: Uuid = Uuid::new_v4();
    claims.insert("sub", new_id.to_string());
    let token_str = claims.sign_with_key(&key).unwrap();
    let result = repository.create_user(new_id).await;
    if result.is_err(){
        return HttpResponse::InternalServerError().body("Failed to create user");
    }
    let mut cookie = Cookie::new("session", token_str);
    cookie.set_path("/");
    let mut future = OffsetDateTime::now_utc();
    future += Duration::weeks(1);
    cookie.set_expires(future);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_secure(true);
    HttpResponse::Ok().cookie(cookie).await.unwrap()

}

