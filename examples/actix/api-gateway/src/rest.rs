use actix_http::StatusCode;
use actix_web::{
    web,
    get,
    web::ServiceConfig,
    HttpResponse, Responder, HttpRequest, cookie::{Cookie, time::{OffsetDateTime, Duration}, SameSite}, HttpResponseBuilder,
};

use coi_actix_web::inject;
use crate::store_interface::UserRepository;
use crate::gateway::{proxy, gen_user, gen_session_token};

pub(super) fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        route_config(config)
    }
}
pub fn route_config(config: &mut ServiceConfig) {
    config.service(web::scope("/public") // Everything that does not require any auth
    .route("sign-up", web::get().to(sign_up)).service(health)).service(
        web::scope("") // Routes are configuration driven
            .route("/{tail:.*}", web::get().to(req_proxy))
            .route("/{tail:.*}", web::post().to(req_proxy))
            .route("/{tail:.*}", web::put().to(req_proxy))
            .route("/{tail:.*}", web::delete().to(req_proxy))
    );
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

fn session_cookie_from_token(token_str: &str) -> Cookie {
    let mut cookie = Cookie::new("session", token_str);
    cookie.set_path("/");
    let mut future = OffsetDateTime::now_utc();
    // Some sane defaults
    future += Duration::days(1);
    cookie.set_expires(future);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_secure(true);
    cookie
}


#[inject]
async fn req_proxy(#[inject] repository: Arc<dyn UserRepository>, req: HttpRequest) -> impl Responder{
    let cookie = req.cookie("session");
    let opt_cookie = match cookie {
        Some(cookie_value) => Some(cookie_value.value().to_owned()),
        None => None
    };
    match proxy(repository, req.method().as_str(), req.path(), opt_cookie).await {
        Ok((body, code, token)) => HttpResponseBuilder::new(
                StatusCode::from_u16(code).unwrap()
            ).cookie(session_cookie_from_token(token.as_str())).body(body),
        Err(code) => match code {
            401_u16 => HttpResponse::Unauthorized().body("Need authentication"),
            403_u16 => HttpResponse::Forbidden().body("Insuficient permissions"),
            404_u16 => HttpResponse::NotFound().body("Not found"),
            _i32 => HttpResponse::BadRequest().body("Bad request")
        }
    }
}

#[inject]
// Our extremely simplified signup. Get the url to automatically register a new user and get a cookie
async fn sign_up(#[inject] repository: Arc<dyn UserRepository>) -> impl Responder {
    
    let user_id = gen_user(repository).await.unwrap();
    let token_str = gen_session_token(user_id).await;
    let cookie = session_cookie_from_token(token_str.as_str());
    HttpResponse::Ok().cookie(cookie).body("Signed up ! Check http://localhost:8080/hello/")

}

