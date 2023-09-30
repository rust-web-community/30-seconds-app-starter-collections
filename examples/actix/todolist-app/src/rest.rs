use actix_web::{
    web,
    get,
    web::{Json, Path, Query, ServiceConfig},
    HttpResponse, Responder,
};
use serde::Deserialize;
use coi_actix_web::inject;

use crate::store_interface::TodoRepository;

use crate::schemas::{ErrorResponse, TodoUpdateRequest, Todo};
use utoipa::IntoParams;


pub(super) fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        route_config(config)
    }
}
pub fn route_config(config: &mut ServiceConfig) {
    config.service(
        web::scope("/todo")
            .route("", web::get().to(get_todos))
            .route("", web::post().to(create_todo))
            .route("/search", web::get().to(search_todos))
            .route("/{id}", web::delete().to(delete_todo))
            .route("/{id}", web::get().to(get_todo_by_id))
            .route("/{id}", web::put().to(update_todo))
    ).service(health);
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

/// Get list of todos.
///
/// List todos from todo store.
///
/// One could call the api endpoint with following curl.
/// ```text
/// curl localhost:8080/todo
/// ```
#[utoipa::path(
    get,
    path = "/todo",
    responses(
        (status = 200, description = "List current todo items", body = [Todo])
    )
)]
#[inject]
async fn get_todos(#[inject] repository: Arc<dyn TodoRepository>) -> impl Responder {
    HttpResponse::Ok().json(repository.read_all().await)
}

/// Create new Todo to storage.
///
/// Post a new `Todo` in request body as json to store it. Api will return
/// created `Todo` on success or `ErrorResponse::Conflict` if todo with same id already exists.
///
/// One could call the api with.
/// ```text
/// curl localhost:8080/todo -d '{"id": 1, "value": "Buy movie ticket", "checked": false}'
/// ```
#[utoipa::path(
    post,
    path = "/todo",
    request_body = Todo,
    responses(
        (status = 201, description = "Todo created successfully", body = Todo),
        (status = 409, description = "Todo with id already exists", body = ErrorResponse, example = json!(ErrorResponse::Conflict(String::from("id = 1"))))
    )
)]
#[inject]
async fn create_todo(todo: Json<Todo>, #[inject] repository: Arc<dyn TodoRepository>) -> impl Responder {
    let result = repository.create_one(&todo.into_inner()).await;
    match result {
        Ok(todo_ret) => HttpResponse::Created().json(todo_ret),
        Err(existing) =>  HttpResponse::Conflict().json(ErrorResponse::Conflict(format!("id = {}", existing.id)))
    }
}

/// Delete Todo by given path variable id.
///
/// This endpoint needs `api_key` authentication in order to call. Api key can be found from README.md.
///
/// Api will delete todo from storage by the provided id and return success 200.
/// If storage does not contain `Todo` with given id 404 not found will be returned.
#[utoipa::path(
    delete,
    path = "/todo",
    responses(
        (status = 200, description = "Todo deleted successfully"),
        (status = 401, description = "Unauthorized to delete Todo", body = ErrorResponse, example = json!(ErrorResponse::Unauthorized(String::from("missing api key")))),
        (status = 404, description = "Todo not found by id", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id", description = "Unique storage id of Todo")
    ),
    security(
        ("api_key" = [])
    )
)]
#[inject]
async fn delete_todo(id: Path<i64>, #[inject] repository: Arc<dyn TodoRepository>) -> impl Responder {
    let result = repository.delete_one(*id).await;
    match result {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(()) => HttpResponse::NotFound().json(ErrorResponse::NotFound(format!("id = {id}")))
    }
}

/// Get Todo by given todo id.
///
/// Return found `Todo` with status 200 or 404 not found if `Todo` is not found from shared in-memory storage.
#[utoipa::path(
    get,
    path = "/todo/{id}",
    responses(
        (status = 200, description = "Todo found from storage", body = Todo),
        (status = 404, description = "Todo not found by id", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id", description = "Unique storage id of Todo")
    )
)]
#[inject]
async fn get_todo_by_id(id: Path<i64>, #[inject] repository: Arc<dyn TodoRepository>) -> impl Responder {
    let result = repository.read_one(*id).await;
    match result {
        Ok(todo) => HttpResponse::Ok().json(todo),
        Err(()) => HttpResponse::NotFound().json(ErrorResponse::NotFound(format!("id = {id}")))
    }

}

/// Update Todo with given id.
///
/// This endpoint supports optional authentication.
///
/// Tries to update `Todo` by given id as path variable. If todo is found by id values are
/// updated according `TodoUpdateRequest` and updated `Todo` is returned with status 200.
/// If todo is not found then 404 not found is returned.
#[utoipa::path(
    put,
    path = "/todo/{id}",
    request_body = TodoUpdateRequest,
    responses(
        (status = 200, description = "Todo updated successfully", body = Todo),
        (status = 404, description = "Todo not found by id", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id", description = "Unique storage id of Todo")
    ),
    security(
        (),
        ("api_key" = [])
    )
)]
#[inject]
async fn update_todo(
    id: Path<i64>,
    todo: Json<TodoUpdateRequest>,
    #[inject] repository: Arc<dyn TodoRepository>,
) -> impl Responder {
    let result = repository.update_one(*id, todo.into_inner()).await;
    match result {
        Ok(todo) => HttpResponse::Ok().json(todo),
        Err(()) => HttpResponse::NotFound().json(ErrorResponse::NotFound(format!("id = {id}")))
    }
}

/// Search todos Query
#[derive(Deserialize, Debug, IntoParams)]
pub(super) struct SearchTodos {
    /// Content that should be found from Todo's value field
    value: String,
}

/// Search Todos with by value
///
/// Perform search from `Todo`s present in in-memory storage by matching Todo's value to
/// value provided as query parameter. Returns 200 and matching `Todo` items.
#[utoipa::path(
    get,
    path = "/todo/search",
    params(
        SearchTodos
    ),
    responses(
        (status = 200, description = "Search Todos did not result error", body = [Todo]),
    )
)]
#[inject]
async fn search_todos(
    query: Query<SearchTodos>,
    #[inject] repository: Arc<dyn TodoRepository>,
) -> impl Responder {
    HttpResponse::Ok().json(repository.read_filter(&query.value).await)
}


