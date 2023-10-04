use crate::stores::cache::User;
use crate::stores::config::{get_config, get_key};
use crate::{AppState, DynCache, DynHttp, DynUserRepo};
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
use jwt_simple::prelude::{Claims, Duration, JWTClaims, MACLike, NoCustomClaims};
use uuid::Uuid;

pub fn configure(router: Router<AppState>) -> Router<AppState> {
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

async fn gen_session_token(user_id: Uuid) -> String {
    let mut claims = Claims::create(Duration::from_days(1));
    claims.subject = Some(user_id.to_string());
    let key = get_key();
    key.authenticate(claims).unwrap()
}

async fn req_proxy(
    State(state_repo): State<DynUserRepo>,
    State(proxy): State<DynHttp>,
    State(cache): State<DynCache>,
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
    let mut opt_user = cache.get_user(user_id).await;
    if opt_user.is_none() {
        opt_user = state_repo.get_user(user_id.clone()).await;
        if opt_user.is_none() {
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    }
    let user = opt_user.unwrap();
    let _ret = cache.create_user(&user);
    let route_config = get_config();
    let method = req.method().as_str();
    let path = req.uri().path();
    for route in &route_config.routes {
        if route.methods.contains(&method.to_owned()) && path.starts_with(&route.prefix) {
            if route.restrict_admin && !user.admin {
                return (StatusCode::FORBIDDEN, "Insuficient permissions").into_response();
            }
            let (code, body) = proxy.make_request(method, path, &user.id).await;

            let token = gen_session_token(user_id).await;
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
    }
    return (StatusCode::NOT_FOUND, "Not found").into_response();
}

//Our extremely simplified signup. Get the url to automatically register a new user and get a cookie

async fn sign_up(
    State(state_repo): State<DynUserRepo>,
    State(cache): State<DynCache>,
) -> impl IntoResponse {
    let u: User = User {
        id: Uuid::new_v4(),
        admin: false,
    };
    state_repo.create_user(&u).await.unwrap();
    cache.create_user(&u).await.unwrap();
    let token_str = gen_session_token(u.id).await;

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
