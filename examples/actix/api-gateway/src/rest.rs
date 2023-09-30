use actix_web::{
    web,
    get,
    web::ServiceConfig,
    HttpResponse, Responder, HttpRequest, cookie::{Cookie, time::{OffsetDateTime, Duration}, SameSite},
};

use coi_actix_web::inject;
use crate::store_interface::UserRepository;
use crate::stores::config;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use jwt::{SignWithKey, VerifyWithKey};
use std::{collections::BTreeMap, str::FromStr, sync::Arc};
use uuid::Uuid;

pub(super) fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        route_config(config)
    }
}
pub fn route_config(config: &mut ServiceConfig) {
    config.service(web::scope("/public") // Everything that does not require any auth
    .route("sign-up", web::get().to(sign_up)).service(health)).service(
        web::scope("") // Routes are configuration driven
            .route("/{tail:.*}", web::get().to(get_proxy))
            .route("/{tail:.*}", web::post().to(post_proxy))
            .route("/{tail:.*}", web::put().to(put_proxy))
            .route("/{tail:.*}", web::delete().to(delete_proxy))

    );
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[inject]
async fn get_proxy(#[inject] repository: Arc<dyn UserRepository>, req: HttpRequest) -> impl Responder{
    proxy(repository, req).await
}

#[inject]
async fn post_proxy(#[inject] repository: Arc<dyn UserRepository>, req: HttpRequest) -> impl Responder{
    proxy(repository, req).await
}
#[inject]
async fn put_proxy(#[inject] repository: Arc<dyn UserRepository>, req: HttpRequest) -> impl Responder{
    proxy(repository, req).await
}
#[inject]
async fn delete_proxy(#[inject] repository: Arc<dyn UserRepository>, req: HttpRequest) -> impl Responder{
    proxy(repository, req).await
}


// Implements proxying any method towards authenticated microservices.
// TODO: extract to an actix-independant module
async fn proxy(repository: Arc<dyn UserRepository>, req: HttpRequest) -> impl Responder {
    // For the purpose of browser testing, we use a session cookie.
    let req_auth = req.cookie("session");
    // Not authenticated - Could redirect to a front signup page
    if req_auth.is_none(){
        return HttpResponse::Forbidden().body("Forbidden");
    }
    // Check token signature
    // In real life, keys should be taken from env, and injected in your production via CI/CD tools
    let key = Hmac::<Sha256>::new_from_slice(b"some-key").unwrap();
    let claims: BTreeMap<String, String> = req_auth.unwrap().value().verify_with_key(&key).unwrap();
    let user_id_str = claims.get("sub").unwrap();
    let user_id = Uuid::from_str(&user_id_str).unwrap();
    // We also verify the user exists. This enable for permission check
    let user = repository.get_user(user_id.clone()).await;
    if user.is_none(){
        return HttpResponse::Forbidden().body("Forbidden");
    }
    // User is authenticated. Cookie jwt could be refreshed starting from here

    // Lazy load of the dynamic routing config file
    let my_config = config::get_config();
    let path = req.path();
    for route in &my_config.routes {
        // See config for more details
        if route.methods.contains(&req.method().as_str().to_owned()) && path.starts_with(&route.prefix) {
            if !user.unwrap().admin && route.restrict_admin{
                return HttpResponse::Forbidden().body("Forbidden")
            }
            // Prepare proxy request
            let client = reqwest::Client::new();
            let mut headers = reqwest::header::HeaderMap::new();
            // Transmit all necessary user info through HTTP headers; its agnostic of query methods and simplifies handling for services
            let header_name = reqwest::header::HeaderName::from_str("X-User").unwrap();
            let header_value = reqwest::header::HeaderValue::from_str(user_id_str).unwrap();
            headers.insert(header_name, header_value);
            let stripped_path = path.strip_prefix(&route.prefix).unwrap();
            // We support proxying variable path this way
            let response = client.get(format!("http://{}{}", route.service, stripped_path)).headers(headers).send().await.unwrap();
            return HttpResponse::Ok().body(response.bytes().await.unwrap())
        }
    }
    HttpResponse::NotFound().body("Not found")
}


#[inject]
// Our extremely simplified signup. You get the url, you automatically register a new user and get a cookie
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
    // Some sane defaults
    future += Duration::weeks(1);
    cookie.set_expires(future);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_secure(true);
    HttpResponse::Ok().cookie(cookie).body("Signed up ! Check http://localhost:8080/hello/")

}

