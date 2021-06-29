use std::sync::RwLock;

use mongodb::{Client, Collection, Database};

use crate::{Result, config::ConfigDatabase};

use self::model::{Gallery, Image, ImageViews, User};

pub mod model;


pub type ImageViewsCollection = Collection<ImageViews>;
pub type ImagesCollection = Collection<Image>;
pub type UsersCollection = Collection<User>;
pub type GalleryCollection = Collection<Gallery>;


lazy_static! {
	static ref DATABASE: RwLock<Option<Database>> = RwLock::new(None);
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

pub fn get_gallery_collection() -> GalleryCollection {
	DATABASE.read().unwrap().as_ref().unwrap().collection("gallery")
}