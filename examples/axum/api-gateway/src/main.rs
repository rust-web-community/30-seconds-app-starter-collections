use std::{net::SocketAddr, sync::Arc};

use crate::store_interface::{CacheRepository, Proxy, UserRepository};
use stores::cache::InMemoryUser;
use stores::http::RqClient;
use stores::postgres::PostgresUser;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod rest;
mod schemas;
mod store_interface;
mod stores {
    pub mod cache;
    pub mod config;
    pub mod http;
    pub mod postgres;
}
use crate::rest::configure;
use axum::{extract::FromRef, Router, Server};

/// Type alias that makes it easier to extract `UserRepo` trait objects.
type DynUserRepo = Arc<dyn UserRepository + Send + Sync>;
type DynHttp = Arc<dyn Proxy + Send + Sync>;
type DynCache = Arc<dyn CacheRepository + Send + Sync>;

#[derive(Clone)]
pub struct AppState {
    // that holds some api specific state
    user_repo: DynUserRepo,
    proxy: DynHttp,
    cache: DynCache,
}

// the api specific state
#[derive(Clone)]
struct ApiDynUserRepo;
// support converting an `AppState` in an `ApiState`
impl FromRef<AppState> for DynUserRepo {
    fn from_ref(app_state: &AppState) -> DynUserRepo {
        app_state.user_repo.clone()
    }
}

#[derive(Clone)]
struct ApiDynHttp;
// support converting an `AppState` in an `ApiState`
impl FromRef<AppState> for DynHttp {
    fn from_ref(app_state: &AppState) -> DynHttp {
        app_state.proxy.clone()
    }
}

#[derive(Clone)]
struct ApiDynCache;
// support converting an `AppState` in an `ApiState`
impl FromRef<AppState> for DynCache {
    fn from_ref(app_state: &AppState) -> DynCache {
        app_state.cache.clone()
    }
}

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

    let postgres_user =
        PostgresUser::new("some_postgres", "postgres", "replacethisplease", "postgres").await;
    let _res = postgres_user.migrate().await.unwrap();

    let state_repo = Arc::new(postgres_user) as DynUserRepo;

    let proxy = Arc::new(RqClient::new()) as DynHttp;
    let cache = InMemoryUser::new();
    let arc_cache = Arc::new(cache) as DynCache;

    // Build our application with some routes
    let app: Router = configure(Router::new()).with_state(AppState {
        user_repo: state_repo,
        proxy: proxy,
        cache: arc_cache,
    });

    // Run our application
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
