use crate::gateway::{gen_session_token, proxy};
use crate::stores::config::get_key;
use crate::DynUserRepo;
use axum::headers::Cookie;
use axum::routing::{delete, get, post, put};
use axum::{
    self,
    body::Body,
    extract::{State, TypedHeader},
    http::header::SET_COOKIE,
    http::status::StatusCode,
    http::Request,
    response::{AppendHeaders, IntoResponse},
    Router,
};
use jwt_simple::prelude::{JWTClaims, MACLike, NoCustomClaims};
use uuid::Uuid;

pub fn configure(router: Router<DynUserRepo>) -> Router<DynUserRepo> {
    router
        .route("/public/sign-up", get(sign_up))
        .route("/health", get(health))
        .route("/*path", get(req_proxy))
        .route("/*path", post(req_proxy))
        .route("/*path", put(req_proxy))
        .route("/*path", delete(req_proxy))
}

async fn health() -> impl IntoResponse {
    "OK"
}

async fn req_proxy(
    State(state_repo): State<DynUserRepo>,
    TypedHeader(cookie): TypedHeader<Cookie>,
    req: Request<Body>,
) -> impl IntoResponse {
    // For the purpose of browser testing, we use a session cookie.
    let sess_cookie = cookie.get("session");
    let cookie_val = match sess_cookie {
        Some(cookie_value) => cookie_value.to_owned(),
        // Not authenticated - Could redirect to a front signup page
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    // Check token signature
    // In real life, keys should be taken from env, and injected in your production via CI/CD tools
    let key = get_key();

    let try_claims: Result<JWTClaims<NoCustomClaims>, jwt_simple::Error> =
        key.verify_token(cookie_val.as_str(), None);
    if try_claims.is_err() {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }
    let claims = try_claims.unwrap();
    let try_user_id = claims.subject;
    if try_user_id.is_none() {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }
    let user_id_str = try_user_id.unwrap();
    let user_id = Uuid::parse_str(&user_id_str).unwrap();
    // We also verify the user exists. This enable for permission check
    let user = state_repo.get_user(user_id.clone()).await;
    if user.is_none() {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }
    let resp = proxy(req.method().as_str(), req.uri().path(), user.unwrap()).await;
    if resp.is_ok() {
        let (token, code, body) = resp.unwrap();

        return (
            StatusCode::from_u16(code).unwrap(),
            AppendHeaders([(
                SET_COOKIE,
                format!(
                    "session={}; Max-Age=86400; Path=/; SameSite=Lax; Secure",
                    token
                ),
            )]),
            String::from_utf8(body.to_vec()).unwrap(),
        )
            .into_response();
    }
    match resp.unwrap_err() {
        401_u16 => (
            StatusCode::from_u16(401).unwrap(),
            "Need authentication".to_owned(),
        )
            .into_response(),
        403_u16 => (
            StatusCode::from_u16(403).unwrap(),
            "Insuficient permissions".to_owned(),
        )
            .into_response(),
        404_u16 => (StatusCode::from_u16(404).unwrap(), "Not found".to_owned()).into_response(),
        other_code => (
            StatusCode::from_u16(other_code).unwrap(),
            "Bad request".to_owned(),
        )
            .into_response(),
    }
}

//Our extremely simplified signup. Get the url to automatically register a new user and get a cookie

async fn sign_up(State(state_repo): State<DynUserRepo>) -> impl IntoResponse {
    let uuid = Uuid::new_v4();
    state_repo.create_user(uuid).await.unwrap();
    let token_str = gen_session_token(uuid).await;
    (
        StatusCode::OK,
        AppendHeaders([(
            SET_COOKIE,
            format!(
                "session={}; Max-Age=86400; Path=/; SameSite=Lax; Secure",
                token_str.as_str()
            ),
        )]),
        "Signed up ! Check http://localhost:8080/hello/",
    )
}
