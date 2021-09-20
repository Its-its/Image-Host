use std::sync::RwLock;

use mongodb::{Client, Collection, Database};

use crate::{Result, config::ConfigDatabase};

use self::model::{Gallery, Image, ImageViews, User, AuthVerify};

pub mod model;


pub type ImageViewsCollection = Collection<ImageViews>;
pub type ImagesCollection = Collection<Image>;
pub type UsersCollection = Collection<User>;
pub type GalleryCollection = Collection<Gallery>;
pub type AuthCollection = Collection<AuthVerify>;


lazy_static! {
	static ref DATABASE: RwLock<Option<Database>> = RwLock::new(None);
}


pub async fn create_mongo_connection(config: &ConfigDatabase) -> Result<Client> {
	let client = Client::with_uri_str(&config.url).await?;

	*DATABASE.write().unwrap() = Some(client.database(&config.database));

	Ok(client)
}


pub fn get_image_views_collection() -> ImageViewsCollection {
	get_collection(CollectionType::ImageViews)
}

pub fn get_images_collection() -> ImagesCollection {
	get_collection(CollectionType::Images)
}

pub fn get_users_collection() -> UsersCollection {
	get_collection(CollectionType::Users)
}

pub fn get_gallery_collection() -> GalleryCollection {
	get_collection(CollectionType::Gallery)
}

pub fn get_auth_collection() -> AuthCollection {
	get_collection(CollectionType::Auths)
}

pub fn get_collection<T>(value: CollectionType) -> Collection<T> where T: serde::Serialize + serde::de::DeserializeOwned + Unpin + std::fmt::Debug, {
	DATABASE.read().unwrap().as_ref().unwrap().collection(value.collection_name())
}

#[derive(Debug, Clone, Copy)]
pub enum CollectionType {
	ImageViews,
	Images,
	Users,
	Gallery,
	Auths
}

impl CollectionType {
	pub fn collection_name(self) -> &'static str {
		match self {
			Self::ImageViews => "images-views",
			Self::Images => "images",
			Self::Users => "users",
			Self::Gallery => "gallery",
			Self::Auths => "auths"
		}
	}
}