#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;

use std::env;

use db::model;
use config::ConfigHelper;
use upload::service::Service;

pub use error::Result;
pub use words::{WordManager, Filename};
pub use config::ConfigInner;



pub mod auth;
pub mod upload;
pub mod error;
pub mod words;
pub mod config;
pub mod web;
pub mod db;


#[actix_web::main]
async fn main() -> Result<()> {
	env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

	let config = ConfigHelper::<ConfigInner>::create_and_load("./app/config.json").await?;

	if config.services.b2.enabled && !config.services.b2.public_url.ends_with('/') {
		panic!(r#"Please end the b2 "public_url" with a "/""#);
	}


	std::mem::forget(db::create_mongo_connection(&config.database).await?);


	// Upload Service
	let service = Service::pick_service_from_config(&config.services).await?;

	web::init(config, service).await?;

	Ok(())
}