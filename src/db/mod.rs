use std::sync::RwLock;

use mongodb::{Client, Collection, Database, IndexModel, bson::doc, options::{Collation, CollationStrength, IndexOptions}};

use crate::{config::ConfigDatabase, Result};

use self::model::{AuthVerify, Gallery, Image, ImageViews, User};

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

	*DATABASE.write()? = Some(client.database(&config.database));

	create_indexes().await?;

	Ok(client)
}

// Create Indexes if they don't exist.
async fn create_indexes() -> Result<()> {
	{ // Users
		let collection = get_users_collection();

		let indexes = collection.list_index_names().await?;

		if !indexes.iter().any(|v| v == "email-cs-index") {
			collection.create_index(
				IndexModel::builder()
					.keys(doc! { "passwordless.email": 1 })
					.options(
						IndexOptions::builder()
							.name("email-cs-index".to_string())
							.collation(Collation::builder().locale("en").strength(CollationStrength::Secondary).build())
							.build()
					)
					.build(),
				None
			).await?;
		}
	}

	{ // Auths
		let collection = get_auth_collection();

		let indexes = collection.list_index_names().await?;

		if !indexes.iter().any(|v| v == "created_at-ttl-index") {
			collection.create_index(
				IndexModel::builder()
					.keys(doc! { "created_at": 1 })
					.options(
						IndexOptions::builder()
							.name("created_at-ttl-index".to_string())
							.expire_after(std::time::Duration::from_secs(60 * 60))
							.build()
					)
					.build(),
				None
			).await?;
		}
	}

	Ok(())
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

pub fn get_collection<T>(value: CollectionType) -> Collection<T>
where
	T: serde::Serialize + serde::de::DeserializeOwned + Unpin + std::fmt::Debug,
{
	#[allow(clippy::unwrap_used)]
	DATABASE
		.read()
		.unwrap()
		.as_ref()
		.unwrap()
		.collection(value.collection_name())
}

#[derive(Debug, Clone, Copy)]
pub enum CollectionType {
	ImageViews,
	Images,
	Users,
	Gallery,
	Auths,
}

impl CollectionType {
	pub fn collection_name(self) -> &'static str {
		match self {
			Self::ImageViews => "images-views",
			Self::Images => "images",
			Self::Users => "users",
			Self::Gallery => "gallery",
			Self::Auths => "auths",
		}
	}
}
