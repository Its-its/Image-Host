#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::env;

use config::ConfigHelper;
use db::model;
use upload::service::Service;

pub use config::ConfigInner;
pub use error::Result;
pub use words::{Filename, WordManager};

pub mod flipstore;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod upload;
pub mod web;
pub mod words;
pub mod feature;

#[actix_web::main]
async fn main() -> Result<()> {
	env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
	env_logger::init();

	let config = ConfigHelper::<ConfigInner>::create_and_load("./app/config.json").await?;

	if config.services.b2.enabled && !config.services.b2.public_url.ends_with('/') {
		panic!(r#"Please end the b2 "public_url" with a "/""#);
	}

	std::mem::forget(db::create_mongo_connection(&config.database).await?);

	println!(
		"Feature Gallery {}",
		if config.features.gallery.enabled {
			"enabled"
		} else {
			"disabled"
		}
	);

	println!(
		"Feature Compression {}",
		if config.features.compression.enabled {
			"enabled"
		} else {
			"disabled"
		}
	);

	// Upload Service
	let service = Service::pick_service_from_config(&config.services).await?;

	web::init(config, service).await?;

	Ok(())
}
