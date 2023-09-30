use std::{
    error::Error,
    net::Ipv4Addr,
};
use actix_http::header::{AUTHORIZATION, HeaderValue};
use actix_web::{
    middleware::Logger,
    web,
    get,
    App, HttpServer,
    HttpResponse, Responder, HttpRequest
};

#[actix_web::main]
async fn main() -> Result<(), impl Error> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(configure())
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}


fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        route_config(config)
    }
}
fn route_config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/hello")
            .route("", web::get().to(hello))
    ).service(health);
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

async fn hello(req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body(req.headers().get(AUTHORIZATION).unwrap_or(&HeaderValue::from_str("stranger").unwrap()).as_bytes().to_owned())
}
