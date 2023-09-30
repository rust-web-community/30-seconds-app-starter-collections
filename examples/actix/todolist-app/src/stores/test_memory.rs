// Unit testing for a in-memory store implementation

use rstest::*;

use crate::stores::memory::*;
use crate::schemas::Todo;


#[fixture]
fn repository() -> InMemoryTodo {
    InMemoryTodo::new([Todo { id: 1, value: "some_value".to_owned(), checked: false }, Todo { id: 2, value: "some_other".to_owned(), checked: true }].to_vec())
}

#[rstest]
async fn test_read_all(repository: InMemoryTodo){
    let read_vals = repository.read_all().await;
    assert_eq!( read_vals.len(), 2);

    for val in read_vals.into_iter(){
        assert_eq!(repository.todos.lock().unwrap().get(&val.id).unwrap(), &val);
    }
}

#[rstest]
async fn test_read_one(repository: InMemoryTodo){
    let read_val = repository.read_one(1).await;
    assert_eq!( &read_val.unwrap(), repository.todos.lock().unwrap().get(&1).unwrap());
}

#[rstest]
async fn test_read_one_fail(repository: InMemoryTodo){
    let read_val = repository.read_one(42).await;
    assert_eq!( read_val.unwrap_err(), ());
}

#[rstest]
async fn create_one(repository: InMemoryTodo){
    let todo = Todo { id: 3, value: "new_value".to_owned(), checked: true };
    let result = repository.create_one(&todo).await;
    assert!(result.is_ok());
    assert_eq!(repository.todos.lock().unwrap().get(&3).unwrap(), &todo);
}

#[rstest]
async fn create_fail(repository: InMemoryTodo){
    let todo = Todo { id: 2, value: "new_value".to_owned(), checked: true };
    let result = repository.create_one(&todo).await;
    assert_eq!(&result.unwrap_err(), repository.todos.lock().unwrap().get(&2).unwrap());
}

#[rstest]
async fn delete_one(repository: InMemoryTodo){
    let result_delete = repository.delete_one(1).await;
    assert!(result_delete.is_ok());
    assert_eq!(repository.todos.lock().unwrap().len(), 1);
    assert!(repository.todos.lock().unwrap().get(&1).is_none());
}

#[rstest]
async fn update_one(repository: InMemoryTodo){
    let result = repository.update_one(1, TodoUpdateRequest { value: None, checked: Some(true) }).await;
    assert!(result.is_ok());
    assert_eq!(repository.todos.lock().unwrap().get(&1).unwrap(), &Todo{ id: 1, value: "some_value".to_owned(), checked: true });
}

#[rstest]
async fn search_text(repository: InMemoryTodo){
    let read_vals = repository.read_filter("value").await;
    assert_eq!( read_vals.len(), 1);
    assert_eq!( read_vals.first().unwrap(), repository.todos.lock().unwrap().get(&1).unwrap());
}