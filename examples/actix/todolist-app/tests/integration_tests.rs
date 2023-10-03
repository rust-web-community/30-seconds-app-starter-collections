use example::schemas::Todo;
use reqwest;
use rstest::rstest;
use serde_json;

// Integration test(s)

// While unit testing will cover most of the domain logic and rest api logic, you are left on your own when it comes to testing the database queries.
// One way to guarantee your web app correctly integrates with the db is through integration testing.
// This test is stateful. It can be integrated on pipelines, but it requires a fully dedicated database, started and torn down in your CI integration.
// Because it's stateful, it's also purely sequential. While you could technically have several sets of test,
// reducing paralellism to 1 and having to clean state between each is basically equivalent to running a single full test as shown below.

#[rstest]
fn test_bin() {
    // Perform a few queries and check results semantically.
    let client = reqwest::blocking::Client::new();
    let resp = client.get("http://localhost:8080/health").send().unwrap();

    assert_eq!(resp.status(), 200);

    let resp = client.get("http://localhost:8080/todo").send().unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().unwrap(), "[]");

    let todo: Todo = Todo {
        id: 60,
        value: "test value".to_string(),
        checked: false,
    };
    let resp = client
        .post("http://localhost:8080/todo")
        .body(serde_json::to_string(&todo).unwrap())
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .send()
        .unwrap();
    assert_eq!(resp.status(), 201);

    let resp = client
        .get("http://localhost:8080/todo/search")
        .query(&[("value", "test")])
        .send()
        .unwrap();
    assert_eq!(resp.status(), 200);
    let res: Vec<Todo> = serde_json::from_slice::<Vec<Todo>>(&resp.bytes().unwrap()).unwrap();

    assert_eq!(res.len(), 1);
    assert_eq!(res.get(0).unwrap(), &todo);

    let resp = client.get("http://localhost:8080/todo/60").send().unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        &serde_json::from_slice::<Todo>(&resp.bytes().unwrap()).unwrap(),
        &todo
    );

    let resp = client
        .delete("http://localhost:8080/todo/60")
        .send()
        .unwrap();
    assert_eq!(resp.status(), 200);
}
