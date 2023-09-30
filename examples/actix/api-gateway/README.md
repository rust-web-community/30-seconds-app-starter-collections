# Actix API gateway


## What is this

This repository contains an example of the API Gateway. It will enable you to efficiently deploy microservices behind a simple and fast authentication proxy.
Proxied through a https reverse proxy, you will protect simple http microservices by entreprise-grade auth in minutes.

## What you may reuse pulling this example

- A multithreaded, fast gateway powered by actix, able to serve (tens of) thousands of queries a second
- A flexible proxy configured through a yml file, able to route to your microservices based on user permission, method, and query path
- Documented sources you can play around and make experimental changes with
- Multi stage docker file for building minimal images
- Async postgres client storage example
- Access logging

## Run and build the project

### With docker:

Run `docker compose up -d`. On the first creation of db, it may sends too soon a ready signal which may crash the app. Simply restart.

### Without docker

Out of the box you will need a running postgres local instance. The code is provided without Tls option enabled. 

Once your postgres server is running, run the `hello_service` by running `cargo_run` inside the folder.

`cargo run` in the main service will fail due to reusing port 8080. The ports are hardcoded in every project `main.rs`, so change as needed.

## License
MIT
Meant to be used, derived or commercialised freely and openly anywhere.