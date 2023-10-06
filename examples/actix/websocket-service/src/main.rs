use actix::*;
use actix_web::{
    middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::time::Instant;

mod server;
mod session;

/// Entry point for our websocket route
async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::NotifyServer>>,
    user_id: web::Query<ConnectId>,
) -> Result<HttpResponse, Error> {
    /*
    let opt_user_id: Option<&actix_web::http::header::HeaderValue> = req.headers().get("X-User");
    if opt_user_id.is_none() {
        return Err(actix_web::error::ErrorUnauthorized("Needs authentication"));
    }
    let user_id: uuid::Uuid =
        uuid::Uuid::try_parse(opt_user_id.unwrap().to_str().unwrap()).unwrap();
    */
    ws::start(
        session::WsNotifySession {
            id: user_id.into_inner().user_id,
            path: req.path().to_owned(),
            hb: Instant::now(),
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

#[derive(Deserialize, Serialize, Clone)]
struct ConnectId {
    user_id: uuid::Uuid,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct PostData {
    user_id: Option<uuid::Uuid>,
    data: String,
}

#[derive(Deserialize, Serialize, Clone, Message, Debug)]
#[rtype(result = "()")]
struct PushData {
    path: String,
    post: PostData,
}

async fn push_route(
    srv: web::Data<Addr<server::NotifyServer>>,
    post_param: web::Json<PostData>,
    req: HttpRequest,
) -> impl Responder {
    let _ = srv
        .send(PushData {
            post: post_param.0,
            path: req.path().to_owned(),
        })
        .await;
    actix_web::HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // start Notify server actor
    let server = server::NotifyServer::new().start(); // NotifyServer::new(app_state)

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .route("/{tail:.*}", web::post().to(push_route))
            .route("/{tail:.*}", web::get().to(ws_route))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
