use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use futures::TryStreamExt;
use mongodb::{
	bson::{doc, oid::ObjectId, DateTime, Document},
	results::{DeleteResult, InsertOneResult, UpdateResult},
	Cursor,
};
use rand::prelude::ThreadRng;

use crate::{error::Result, upload::image::UploadImageType, words, Filename};

use super::{get_users_collection, AuthCollection, GalleryCollection, ImagesCollection, UsersCollection};


pub enum UserId {
	Id(ObjectId),
	UniqueId(String),
}

impl From<String> for UserId {
	fn from(id: String) -> Self {
		Self::UniqueId(id)
	}
}

impl From<&str> for UserId {
	fn from(id: &str) -> Self {
		Self::UniqueId(id.to_string())
	}
}

impl From<ObjectId> for UserId {
	fn from(id: ObjectId) -> Self {
		Self::Id(id)
	}
}


// USERS

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
	#[serde(rename = "_id")]
	pub id: ObjectId,

	pub twitter: Option<UserTwitter>,
	pub passwordless: Option<UserPasswordless>,

	#[serde(serialize_with = "bson_unsigned_fix")]
	pub upload_type: UploadImageType,

	pub is_banned: bool,

	pub join_date: DateTime,

	pub unique_id: String,

	pub image_count: i32,
	pub deletion_count: i32,

	#[serde(rename = "__v")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub version_key: Option<i32>,
}

impl From<User> for SlimUser {
    fn from(val: User) -> Self {
        SlimUser {
			id: val.id,
			unique_id: val.unique_id,
		}
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SlimUser {
	pub id: ObjectId,
	pub unique_id: String,
}

impl SlimUser {
	pub async fn upgrade(&self) -> Result<Option<User>> {
		find_user_by_id(self.id, &get_users_collection()).await
	}
}


#[derive(Debug, Serialize, Deserialize)]
pub struct NewUser {
	#[serde(serialize_with = "bson_unsigned_fix")]
	pub upload_type: UploadImageType,

	pub is_banned: bool,

	pub join_date: DateTime,

	pub unique_id: String,

	pub image_count: i32,
	pub deletion_count: i32,

	pub twitter: Option<UserTwitter>,
	pub passwordless: Option<UserPasswordless>,
}

impl NewUser {
	pub fn into_user(self, id: ObjectId) -> User {
		User {
			id,

			upload_type: self.upload_type,
			is_banned: self.is_banned,
			join_date: self.join_date,
			unique_id: self.unique_id,
			image_count: self.image_count,
			deletion_count: self.deletion_count,

			twitter: self.twitter,
			passwordless: self.passwordless,

			version_key: None,
		}
	}
}

pub async fn find_user_by_id<I: Into<UserId>>(
	user_id: I,
	collection: &UsersCollection,
) -> Result<Option<User>> {
	match user_id.into() {
		UserId::Id(user_id) => Ok(collection.find_one(doc! { "_id": user_id }, None).await?),

		UserId::UniqueId(unique_id) => Ok(collection
			.find_one(doc! { "unique_id": unique_id }, None)
			.await?),
	}
}

fn bson_unsigned_fix<S>(
	value: &UploadImageType,
	serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
	S: serde::Serializer,
{
	serializer.serialize_i32(value.to_num() as i32)
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UserTwitter {
	pub id: i64,
	pub username: String,
	pub display_name: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UserPasswordless {
	pub email: String
}


// IMAGE VIEWS

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageViews {
	pub id: i32,

	pub image_id: i32,

	pub view_count: i64,
}

fn is_false(value: &bool) -> bool {
	!value
}


// IMAGES

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
	#[serde(rename = "_id")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub id: Option<ObjectId>,

	pub name: String,
	pub file_type: String,

	pub size_original: i64,
	pub size_compressed: i64,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub custom_name: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub tags: Option<Vec<String>>,

	#[serde(default)]
	#[serde(skip_serializing_if = "is_false")]
	pub is_edited: bool,

	#[serde(default)]
	#[serde(skip_serializing_if = "is_false")]
	pub is_favorite: bool,

	pub view_count: i32,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub deleted: Option<DateTime>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub uploader_id: Option<ObjectId>,

	pub uploader: ImageUploader,

	pub upload_date: DateTime,
}

impl Image {
	pub fn get_file_name(&self) -> Result<Filename> {
		Filename::new(self.name.clone(), Some(self.file_type.clone()))
	}

	pub async fn upload(&self, collection: &ImagesCollection) -> Result<InsertOneResult> {
		Ok(collection.insert_one(self, None).await?)
	}

	pub async fn delete_document(self, collection: &ImagesCollection) -> Result<DeleteResult> {
		Ok(collection
			.delete_one(doc! { "_id": self.id.unwrap() }, None)
			.await?)
	}

	pub async fn delete_request(self, collection: &ImagesCollection) -> Result<UpdateResult> {
		Ok(collection
			.update_one(
				doc! { "_id": self.id.unwrap() },
				doc! {
					"$set": {
						"deleted": DateTime::now()
					}
				},
				None,
			)
			.await?)
	}

	pub async fn restore_request(self, collection: &ImagesCollection) -> Result<UpdateResult> {
		Ok(collection
			.update_one(
				doc! { "_id": self.id.unwrap() },
				doc! {
					"$unset": {
						"deleted": ""
					}
				},
				None,
			)
			.await?)
	}
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ImageUploader {
	pub uid: String,
	pub ip: Option<String>,
}


// Image sent to front-end
#[derive(Debug, Serialize, Deserialize)]
pub struct SlimImage {
	pub custom_name: Option<String>,

	pub name: String,
	pub file_type: String,

	pub size_original: i64,
	pub size_compressed: i64,

	pub is_edited: bool,
	pub is_favorite: bool,
	pub view_count: i32,

	pub upload_date: DateTime,
}

impl SlimImage {
	pub fn full_file_name(&self) -> String {
		format!("{}.{}", self.name, self.file_type)
	}
}

impl From<Image> for SlimImage {
	fn from(img: Image) -> Self {
		Self {
			custom_name: img.custom_name,

			name: img.name,
			file_type: img.file_type,
			size_original: img.size_original,
			size_compressed: img.size_compressed,

			is_edited: img.is_edited,
			is_favorite: img.is_favorite,
			view_count: img.view_count,

			upload_date: img.upload_date,
		}
	}
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Gallery {
	#[serde(rename = "_id")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub id: Option<ObjectId>,

	pub user_id: ObjectId,

	pub name: String,

	pub title: Option<String>,

	pub images: Vec<GalleryImage>,

	pub indexed: i64,

	pub updated_at: DateTime,
	pub created_at: DateTime,
}

impl Gallery {
	pub fn add_image(&mut self, image: Image) {
		self.images.push(GalleryImage {
			id: image.id.unwrap(),
			index: self.indexed,
			description: None,
		});

		self.indexed += 1;
	}

	pub async fn update(self, collection: &GalleryCollection) -> Result<UpdateResult> {
		let mut doc = Document::new();

		doc.insert("indexed", self.indexed);
		doc.insert("updated_at", DateTime::now());
		doc.insert(
			"images",
			self.images
				.iter()
				.map(|v| mongodb::bson::to_bson(v).unwrap())
				.collect::<Vec<_>>(),
		);
		if let Some(value) = self.title {
			doc.insert("title", value);
		}

		Ok(collection
			.update_one(
				doc! { "_id": self.id.unwrap() },
				doc! {
					"$set": doc
				},
				None,
			)
			.await?)
	}

	pub async fn delete(self, collection: &GalleryCollection) -> Result<DeleteResult> {
		Ok(collection
			.delete_one(doc! { "_id": self.id.unwrap() }, None)
			.await?)
	}
}


#[derive(Debug, Serialize, Deserialize)]
pub struct GalleryImage {
	pub id: ObjectId,
	pub index: i64,
	pub description: Option<String>,
}

pub async fn does_gallery_exist(name: &str, collection: &GalleryCollection) -> Result<bool> {
	Ok(collection
		.find_one(
			doc! {
				"name": name
			},
			None,
		)
		.await?
		.is_some())
}

pub async fn gallery_count(id: &ObjectId, collection: &GalleryCollection) -> Result<u64> {
	Ok(collection
		.count_documents(
			doc! {
				"user_id": id
			},
			None,
		)
		.await?)
}

pub async fn create_empty_gallery(
	user_id: ObjectId,
	rng: &mut ThreadRng,
	collection: &GalleryCollection,
) -> Result<String> {
	let mut name = words::gen_sample_alphanumeric(8, rng);

	// TODO: While Loop
	while does_gallery_exist(&name, collection).await? {
		name = words::gen_sample_alphanumeric(8, rng);
	}

	let gallery = Gallery {
		id: None,
		user_id,
		name,
		title: None,
		indexed: 0,
		images: Vec::new(),
		updated_at: DateTime::now(),
		created_at: DateTime::now(),
	};

	collection.insert_one(&gallery, None).await?;

	Ok(gallery.name)
}

pub async fn find_gallery_by_name(
	f_name: &str,
	collection: &GalleryCollection,
) -> Result<Option<Gallery>> {
	Ok(collection.find_one(doc! { "name": f_name }, None).await?)
}

pub async fn find_images_from_gallery(
	images: &[GalleryImage],
	collection: &ImagesCollection,
) -> Result<Vec<Image>> {
	let image_ids: Vec<_> = images.iter().map(|v| &v.id).collect();

	let cursor = collection
		.find(
			doc! {
				"_id": {
					"$all": image_ids
				}
			},
			None,
		)
		.await?;

	Ok(cursor.try_collect().await?)
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AuthVerify {
	#[serde(rename = "_id")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub id: Option<ObjectId>,

	pub oauth_token: String,

	pub oauth_token_secret: String,

	pub created_at: DateTime,
}

pub async fn create_auth_verify(
	oauth_token: String,
	oauth_token_secret: String,
	collection: &AuthCollection,
) -> Result<()> {
	collection
		.insert_one(
			AuthVerify {
				id: None,
				oauth_token,
				oauth_token_secret,
				created_at: DateTime::now()
			},
			None,
		)
		.await?;

	Ok(())
}

pub async fn find_and_remove_auth_verify(
	oauth_token: &str,
	collection: &AuthCollection,
) -> Result<Option<AuthVerify>> {
	Ok(collection
		.find_one_and_delete(doc! { "oauth_token": oauth_token }, None)
		.await?)
}

pub async fn does_image_name_exist(f_name: &str, collection: &ImagesCollection) -> Result<bool> {
	Ok(collection
		.count_documents(doc! { "name": f_name }, None)
		.await? != 0)
}

pub async fn find_image_by_name(
	f_name: &str,
	collection: &ImagesCollection,
) -> Result<Option<Image>> {
	Ok(collection.find_one(doc! { "name": f_name }, None).await?)
}

pub async fn find_images_by_date<I: Into<UserId>>(
	f_id: I,
	f_year: u32,
	f_month: u32,
	collection: &ImagesCollection,
) -> Result<Cursor<Image>> {
	let naive_cm =
		NaiveDate::from_ymd(f_year as i32, f_month, 1).and_time(NaiveTime::from_hms(0, 0, 0));
	let naive_nm = to_end_of_month_with_hours(naive_cm);

	match f_id.into() {
		UserId::Id(f_id) => {
			let found = collection
				.find(
					doc! {
						"uploader_id": f_id,
						"upload_date": {
							"$gte": DateTime::from_millis(naive_cm.timestamp_millis()),
							"$lte": DateTime::from_millis(naive_nm.timestamp_millis())
						}
					},
					None,
				)
				.await?;

			Ok(found)
		}

		UserId::UniqueId(f_id) => {
			let doc = doc! {
				"uploader.uid": f_id,
				"upload_date": {
					"$gte": DateTime::from_millis(naive_cm.timestamp_millis()),
					"$lte": DateTime::from_millis(naive_nm.timestamp_millis())
				}
			};

			let found = collection.find(doc, None).await?;

			Ok(found)
		}
	}
}


// Util

// TODO: Remove panics.

pub fn to_next_month(current_month: NaiveDateTime) -> NaiveDateTime {
	if current_month.month() == 12 {
		current_month
			.with_month(1)
			.unwrap()
			.with_year(current_month.year() + 1)
			.unwrap()
	} else {
		current_month.with_month(current_month.month() + 1).unwrap()
	}
}

pub fn to_end_of_month_with_hours(current_month: NaiveDateTime) -> NaiveDateTime {
	// current_month = 2021-06-01T00:00:00

	let days_in_month = days_in_month(current_month);

	current_month
		.with_day(days_in_month as u32)
		.expect("unable to inc days")
		.with_hour(23)
		.expect("unable to inc hours")
		.with_minute(59)
		.expect("unable to inc minutes")
		.with_second(59)
		.expect("unable to inc seconds")
}

pub fn days_in_month(current_month: NaiveDateTime) -> i64 {
	to_next_month(current_month)
		.signed_duration_since(current_month)
		.num_days()
}

pub fn to_end_of_month(current_month: NaiveDateTime) -> NaiveDateTime {
	let days_in_month = days_in_month(current_month);
	current_month.with_day(days_in_month as u32).unwrap()
}