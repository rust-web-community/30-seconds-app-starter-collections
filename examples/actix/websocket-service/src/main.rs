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

#[derive(Deserialize, Serialize, Clone, Message)]
#[rtype(result = "()")]
struct PushData {
    user_id: uuid::Uuid,
    data: String,
}

async fn push_route(
    srv: web::Data<Addr<server::NotifyServer>>,
    post_param: web::Json<PushData>,
) -> impl Responder {
    let _ = srv.send(post_param.0).await;
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
            .route("/", web::post().to(push_route))
            .route("/ws", web::get().to(ws_route))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
