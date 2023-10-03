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

## Manual testing

Run the project. Check you are forbidden to access localhost:8000/hello

Go to public/sign-up. You shouuld have a prompt saying you are signed up as your user is created.

Browse /hello and /restricted. One should display your user id, the other be restricted as the default signup is created as not an admin.

## Benchmarks

Ran client and server locally, using docker, on my own hardware:

- When the request is bounced and an error is sent:

```
$ wrk -t6 -c80 -d10s http://localhost:8000/hello -H "Cookie: session=invalid-cookie"
Running 10s test @ http://localhost:8000/hello
  6 threads and 80 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     0.86ms    3.50ms 103.29ms   99.36%
    Req/Sec    21.30k     2.08k   38.69k    97.68%
  1277871 requests in 10.10s, 127.96MB read
  Non-2xx or 3xx responses: 1277871
Requests/sec: 126528.81
Transfer/sec:     12.67MB
```

- Performing actual proxy request on a cached user:

```
$ wrk -t6 -c80 -d10s http://localhost:8000/hello -H "Cookie: session=eyJhbGciOiJIUzM4NCIsInR5cCI6I[...]"
Running 10s test @ http://localhost:8000/hello
  6 threads and 80 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.79ms    1.22ms  49.06ms   90.25%
    Req/Sec     7.52k   503.93     8.44k    71.40%
  452694 requests in 10.10s, 190.39MB read
Requests/sec:  44822.54
Transfer/sec:     18.85MB
```

- Performing proxy request on a user without cache (cache miss scenario):

```
$ wrk -t6 -c80 -d10s http://localhost:8000/hello -H "Cookie: session=eyJhbGciOiJIUzM4NCIsInR5cCI6I[...]"
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     3.99ms    3.34ms  81.46ms   99.32%
    Req/Sec     3.45k   352.30     6.88k    98.51%
  207160 requests in 10.10s, 87.13MB read
Requests/sec:  20512.04
Transfer/sec:      8.63MB
```

## License
MIT
Meant to be used, derived or commercialised freely and openly anywhere.
