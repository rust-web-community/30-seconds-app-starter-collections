# Actix service bootstrap


## What is this

This repository contain a fully working example of a service written with actix. It is meant to be as close as production ready as possible while remaining minimal and easy to understand.
The service itself is a basic Create / Read / Update / Delete of a todo list, showcasing good production practices for web applications

## What you may reuse pulling this example

- A fast web service in an extensible port and adapter pattern, using dependency injection
- Documented sources you can play around and make experimental changes with
- Multi stage docker file for building minimal images
- API documentation and manual testing with Swagger UI, Redoc, Rapi
- Async postgres client storage example
- Unit testing using fixtures
- Integration testing
- Access logging

## Run and build the project

### With docker:

Run `docker compose up -d`. On the first creation of db, it sends too soon a ready signal which may crash the web app. You may do `docker compose start app` to restart the service

### Without docker

Out of the box you will need a running postgres local instance. The code is provided without Tls option enabled. Once your postgres server is running, simply `cargo run`. Alternatively
you can use an in-memory store provided. Swapping storage method is only a couple of line changes in the main.rs.

### Testing

- Unit testing  `cargo test  --lib --bins`
- Integration testing `cargo test --test '*'`. You will need to run the server instance to be able to pass integration test. See notes in the source file.

## License
MIT
Meant to be used, derived or commercialised freely and openly anywhere.