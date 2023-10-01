use serde::{Deserialize, Serialize};
use serde_yaml::{self};
use lazy_static::lazy_static;
use jwt_simple::prelude::HS384Key;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub routes: Vec<Route>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Route {
    pub methods: Vec<String>,
    pub prefix: String,
    pub service: String,
    pub restrict_admin: bool
}

pub fn get_config() -> &'static Config {
    lazy_static!{static ref CONFIG: Config = serde_yaml::from_reader(std::fs::File::open("routes.yml").expect("Could not open config file.")).expect("Could not read config file.");};
    return &CONFIG;
}

pub fn get_key() -> &'static HS384Key {
    // Generate a key when its not set in environment, for demo purpose
    // Kept empty, the sessions will drop at every server restart, so make sure to provide a key in production
    lazy_static!{static ref KEY: HS384Key = match env::var("GATEWAY_API_KEY") {
        Ok(str) => HS384Key::from_bytes(str.as_bytes()),
        Err(_) => HS384Key::generate()
    };};
    return &KEY;
}