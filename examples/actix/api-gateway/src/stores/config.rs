use serde::{Deserialize, Serialize};
use serde_yaml::{self};
use lazy_static::lazy_static;

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