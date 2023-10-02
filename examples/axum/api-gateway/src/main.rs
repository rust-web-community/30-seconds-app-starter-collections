use std::{
    net::SocketAddr,
    sync::Arc
};

use stores::postgres::PostgresUser;
use store_interface::UserRepository;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod rest;
mod gateway;
mod store_interface;
mod schemas;
mod stores {
    pub mod postgres;
    pub mod config;
}
use axum::{
    Router, Server
};
use crate::rest::configure;

/// Type alias that makes it easier to extract `UserRepo` trait objects.
type DynUserRepo = Arc<dyn UserRepository + Send + Sync>;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_error_handling_and_dependency_injection=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // Local access to a dockerized postgres


    let init_repo = PostgresUser::new("some_postgres", "postgres", "replacethisplease", "postgres").await;
    let _res = init_repo.migrate().await.unwrap();

    let state_repo = Arc::new(init_repo) as DynUserRepo;

    // Build our application with some routes
    let app: Router = configure(Router::new()).with_state(state_repo);

    // Run our application
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}
