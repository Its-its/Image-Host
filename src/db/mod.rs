use std::sync::RwLock;

use mongodb::{Client, Collection, Database};

use crate::{Result, config::ConfigDatabase};

use self::model::{Image, ImageViews, User};

pub mod model;


// pub type SessionsCollection = Collection<>;
pub type ImageViewsCollection = Collection<ImageViews>;
pub type ImagesCollection = Collection<Image>;
pub type UsersCollection = Collection<User>;


lazy_static! {
	pub static ref DATABASE: RwLock<Option<Database>> = RwLock::new(None);
}


pub async fn create_mongo_connection(config: &ConfigDatabase) -> Result<Client> {
	let client = Client::with_uri_str(&config.url).await?;

	DATABASE.write().unwrap().insert(client.database(&config.database));

	Ok(client)
}


pub fn get_image_views_collection() -> ImageViewsCollection {
	DATABASE.read().unwrap().as_ref().unwrap().collection("image-views")
}

pub fn get_images_collection() -> ImagesCollection {
	DATABASE.read().unwrap().as_ref().unwrap().collection("images")
}

pub fn get_users_collection() -> UsersCollection {
	DATABASE.read().unwrap().as_ref().unwrap().collection("users")
}