// Unit testing for the rest logic

#[cfg(test)]
mod tests {
    use actix_web::{test, App, web::Data};
    use std::collections::HashSet;
    use coi::{container, Container};
    use rstest::{fixture, rstest};
    use crate::{schemas::Todo, stores::memory::{TodoRepository, TodoMemoryProvider}};
    use crate::rest::configure;

    #[fixture]
    fn fixt_container() -> fn(Vec<Todo>) -> Container {
        fn prepare(data: Vec<Todo>) -> Container {
            let memory_provider: TodoMemoryProvider = TodoMemoryProvider{todo_list: data};
            let repo = container!{
                repository => memory_provider; singleton
            };
            repo
        }
        prepare
    }

    #[fixture]
    fn test_data() -> Vec<Todo> {
        [Todo{id:1, value:String::from("some value"), checked:true},
         Todo{id:2, value:String::from("something completely different"), checked:false}
        ].to_vec()
    }

    #[rstest]
    async fn test_todo_get(fixt_container:fn(Vec<Todo>) -> Container , test_data: Vec<Todo>) {
        let container = fixt_container(test_data.clone());
        let app = test::init_service(App::new().app_data(container).configure(configure())).await;
        let req = test::TestRequest::get().uri("/todo");
        let resp = test::call_and_read_body_json::<_, _, Vec<Todo>>(&app, req.to_request()).await;
        assert_eq!(HashSet::<&Todo>::from_iter(resp.iter()), HashSet::<&Todo>::from_iter(test_data.iter()));
    }

    #[rstest]
    async fn test_todo_get_by_id(fixt_container:fn(Vec<Todo>) -> Container , test_data: Vec<Todo>) {
        let container = fixt_container(test_data.clone());
        let app = test::init_service(App::new().app_data(container).configure(configure())).await;
        let req = test::TestRequest::get().uri("/todo/1");
        let resp = test::call_and_read_body_json::<_, _, Todo>(&app, req.to_request()).await;
        assert_eq!(&resp, test_data.get(0).unwrap());
    }

    #[rstest]
    async fn test_todo_post(fixt_container:fn(Vec<Todo>) -> Container, test_data: Vec<Todo>) {
        let container = fixt_container(test_data);
        let app = test::init_service(App::new().app_data(container.clone()).configure(configure())).await;
        let req = test::TestRequest::post().uri("/todo").set_json(Data::new(Todo{checked: false, value: "some_value".to_owned(), id: 42}));
        let resp =test::call_service(&app, req.to_request()).await;
        assert!(resp.status().is_success());
        assert_eq!(container.resolve::<dyn TodoRepository>("repository").unwrap().read_all().await.len(), 3)
    }

    #[rstest]
    async fn test_todo_search(fixt_container:fn(Vec<Todo>) -> Container, test_data: Vec<Todo>) {
        let container = fixt_container(test_data.clone());
        let app = test::init_service(App::new().app_data(container).configure(configure())).await;

        let expected_todo = test_data.get(0).unwrap().clone();
        let todo_expected_list : Vec<Todo> = [expected_todo].to_vec();

        let req = test::TestRequest::get().uri("/todo/search?value=value");
        let resp = test::call_and_read_body_json::<_, _, Vec<Todo>>(&app, req.to_request()).await;

        assert_eq!(resp, todo_expected_list);
    }
    // [...]
}