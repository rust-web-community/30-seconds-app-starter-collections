use std::{error::Error, net::Ipv4Addr};

use actix_web::{middleware::Logger, App, HttpServer};
use coi::container;
use stores::postgres::UserPostgresProvider;

use crate::stores::cache::UserMemoryProvider;
use crate::stores::http::RqClientProvider;

mod gateway;
mod rest;
mod schemas;
mod store_interface;
mod stores {
    pub mod cache;
    pub mod config;
    pub mod http;
    pub mod postgres;
}

#[actix_web::main]
async fn main() -> Result<(), impl Error> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // Local access to a dockerized postgres
    let provider =
        UserPostgresProvider::new("some_postgres", "postgres", "replacethisplease", "postgres")
            .await;
    let _res = provider.migrate().await;

    let containers = container! {
        repository => provider; singleton,
        cache => UserMemoryProvider; singleton,
        client => RqClientProvider; singleton,
    };

    HttpServer::new(move || {
        // This factory closure is called on each worker thread independently.
        App::new()
            .wrap(Logger::default())
            .app_data(containers.clone())
            .configure(rest::configure())
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8000))?
    .run()
    .await
}
