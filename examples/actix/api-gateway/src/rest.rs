use actix_http::header::{HeaderMap, AUTHORIZATION};
use actix_web::{
    web,
    get,
    web::{Path, ServiceConfig},
    HttpResponse, Responder, HttpRequest,
};
//use coi_actix_web::inject;

//use crate::store_interface::UserRepository;

//use crate::schemas::User;


pub(super) fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        route_config(config)
    }
}
pub fn route_config(config: &mut ServiceConfig) {
    config.service(
        web::scope("/hello/") // Routes should be configuration driven
            .route("", web::get().to(get_proxy))
    ).service(health);
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

async fn get_proxy(req: HttpRequest) -> impl Responder {
    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(AUTHORIZATION, reqwest::header::HeaderValue::from_str("some-id").unwrap());
    let response = client.get(format!("http://hello_service:8080/hello")).headers(headers).send().await.unwrap();
    HttpResponse::Ok().body(response.bytes().await.unwrap())
}