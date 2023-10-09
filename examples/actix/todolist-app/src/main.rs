use std::{
    error::Error,
    net::Ipv4Addr,
};

use actix_web::{
    middleware::Logger,
    App, HttpServer,
};
use coi::container;
//use stores::memory::TodoMemoryProvider;
use stores::postgres::TodoPostgresProvider;

mod rest;
mod store_interface;
mod schemas;
mod stores {
    pub mod memory;
    #[cfg(test)]
    pub mod test_memory;
    pub mod postgres;
}
#[cfg(test)]
pub mod test_rest;

use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::schemas::{ErrorResponse, Todo, TodoUpdateRequest};


#[actix_web::main]
async fn main() -> Result<(), impl Error> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    //Swap here as needed
    //let provider = TodoMemoryProvider{todo_list: Vec::new()};
    let provider = TodoPostgresProvider::new("localhost", "admin", "admin", "scottzuke", "5433").await;
    let _res = provider.migrate().await;
    let containers = container!{
        repository => provider; singleton,
    };
    #[derive(OpenApi)]
    #[openapi(
        paths(
            rest::get_todos,
            rest::create_todo,
            rest::delete_todo,
            rest::get_todo_by_id,
            rest::update_todo,
            rest::search_todos
        ),
        components(
            schemas(Todo, TodoUpdateRequest, ErrorResponse)
        ),
        tags(
            (name = "todo", description = "Todo management endpoints.")
        ),
    )]
    struct ApiDoc;
    // Make instance variable of ApiDoc so all worker threads gets the same instance.
    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        // This factory closure is called on each worker thread independently.
        App::new()
            .wrap(Logger::default())
            .app_data(containers.clone())
            .configure(rest::configure())
            .service(Redoc::with_url("/redoc", openapi.clone()))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            // There is no need to create RapiDoc::with_openapi because the OpenApi is served
            // via SwaggerUi instead we only make rapidoc to point to the existing doc.
            .service(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
            // Alternative to above
            // .service(RapiDoc::with_openapi("/api-docs/openapi2.json", openapi.clone()).path("/rapidoc"))
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}

