use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use mongodb::{Cursor, bson::{DateTime, doc, oid::ObjectId}, results::{DeleteResult, InsertOneResult, UpdateResult}};

use crate::{error::Result, upload::image::UploadImageType};

use super::{ImagesCollection, UsersCollection, get_users_collection};

pub enum UserId {
	Id(ObjectId),
	UniqueId(String)
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

	pub data: UserData,

	#[serde(rename = "__v")]
	pub version_key: i32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlimUser {
	pub id: ObjectId
}

impl SlimUser {
	pub async fn find_user(&self) -> Result<Option<User>> {
		find_user_by_id(self.id, &get_users_collection()).await
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewUser<'a> {
	pub twitter_id: &'a str,
	pub twitter_token: &'a str,
	pub twitter_display_name: &'a str,
	pub twitter_username: &'a str,

	pub upload_type: i32,
	pub is_banned: bool,
	pub join_date: NaiveDateTime,
	pub unique_id: String,
	pub image_count: i32,
	pub deletion_count: i32,
}

pub async fn find_user_by_id<I: Into<UserId>>(user_id: I, collection: &UsersCollection) -> Result<Option<User>> {
	match user_id.into() {
		UserId::Id(user_id) => {
			Ok(collection.find_one(doc! { "_id": user_id }, None).await?)
		}

		UserId::UniqueId(user_id) => {
			Ok(collection.find_one(doc! { "unqiue_id": user_id }, None).await?)
		}
	}
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
	#[serde(rename = "type")]
	pub upload_type: UploadImageType,
	#[serde(rename = "banned")]
	pub is_banned: bool,
	#[serde(rename = "joinDate")]
	pub join_date: DateTime,
	#[serde(rename = "uid")]
	pub unique_id: String,
	#[serde(rename = "images")]
	pub image_count: i32,
	#[serde(rename = "deletions")]
	pub deletion_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTwitter {
	pub id: String,
	pub token: String,
	pub username: String,
	#[serde(rename = "displayName")]
	pub display_name: String,
}



// IMAGE VIEWS

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageViews {
	pub id: i32,

	pub image_id: i32,

	#[serde(rename = "viewCount")]
	pub view_count: i64
}


// IMAGES

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
	#[serde(rename = "_id")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub id: Option<ObjectId>,

	pub name: String,
	#[serde(rename = "type")]
	pub file_type: String,
	#[serde(rename = "size")]
	pub file_size: i64,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub custom_name: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub tags: Option<Vec<String>>,

	#[serde(default)]
	pub is_edited: bool,

	#[serde(rename = "favorite")]
	pub is_favorite: bool,

	#[serde(rename = "views")]
	pub view_count: i32,

	pub deleted: Option<DateTime>,

	pub uploader_id: Option<ObjectId>,

	pub uploader: ImageUploader,

	#[serde(rename = "date")]
	pub upload_date: DateTime,
}

impl Image {
	pub fn full_file_name(&self) -> String {
		format!("{}.{}", self.name, self.file_type)
	}

	pub async fn upload(self, collection: &ImagesCollection) -> Result<InsertOneResult> {
		Ok(collection.insert_one(self, None).await?)
	}

	pub async fn delete_document(self, collection: &ImagesCollection) -> Result<DeleteResult> {
		Ok(collection.delete_one(doc! { "_id": self.id.unwrap() }, None).await?)
	}

	pub async fn delete_request(self, collection: &ImagesCollection) -> Result<UpdateResult> {
		Ok(collection.update_one(
			doc! { "_id": self.id.unwrap() },
			doc! {
				"$set": {
					"deleted": DateTime::now()
				}
			},
			None
		).await?)
	}

	pub async fn restore_request(self, collection: &ImagesCollection) -> Result<UpdateResult> {
		Ok(collection.update_one(
			doc! { "_id": self.id.unwrap() },
			doc! {
				"$unset": {
					"deleted": ""
				}
			},
			None
		).await?)
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
	pub file_size: i64,

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
			file_size: img.file_size,


			is_edited: img.is_edited,
			is_favorite: img.is_favorite,
			view_count: img.view_count,

			upload_date: img.upload_date
		}
	}
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Gallery {
	#[serde(rename = "_id")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub id: Option<ObjectId>,

	pub user_id: ObjectId,

	pub title: Option<String>,

	pub images: Vec<GalleryImage>,

	pub uploaded_at: DateTime,
	pub created_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GalleryImage {
	pub id: ObjectId,

	pub file_name: String,

	pub description: Option<String>,

	pub created_at: DateTime,
}



pub async fn does_image_name_exist(f_name: &str, collection: &ImagesCollection) -> Result<bool> {
	Ok(collection.count_documents(doc! { "name": f_name }, None).await? != 0)
}

pub async fn find_image_by_name(f_name: &str, collection: &ImagesCollection) -> Result<Option<Image>> {
	Ok(collection.find_one(doc! { "name": f_name }, None).await?)
}

pub async fn find_images_by_date<I: Into<UserId>>(f_id: I, f_year: u32, f_month: u32, collection: &ImagesCollection) -> Result<Cursor<Image>> {
	let naive_cm = NaiveDate::from_ymd(f_year as i32, f_month, 1).and_time(NaiveTime::from_hms(0, 0, 0));
	let naive_nm = to_end_of_month_with_hours(naive_cm);

	match f_id.into() {
		UserId::Id(f_id) => {
			let found = collection.find(
				doc! {
					"uploader_id": f_id,
					"date": {
						"$gte": DateTime::from_millis(naive_cm.timestamp_millis()),
						"$lte": DateTime::from_millis(naive_nm.timestamp_millis())
					}
				},
				None
			).await?;

			Ok(found)
		},

		UserId::UniqueId(f_id) => {
			// let user = find_user_by_id(f_id, &get_users_collection()).await?.expect("Unable to find user");

			let doc = doc! {
				"uploader.uid": f_id,
				"date": {
					"$gte": DateTime::from_millis(naive_cm.timestamp_millis()),
					"$lte": DateTime::from_millis(naive_nm.timestamp_millis())
				}
			};

			println!("{:#?}", doc);

			let found = collection.find(
				doc,
				None
			).await?;

			Ok(found)
		}
	}
}



// Util

// TODO: Remove panics.

pub fn to_next_month(current_month: NaiveDateTime) -> NaiveDateTime {
    if current_month.month() == 12 {
        current_month.with_month(1).unwrap()
        .with_year(current_month.year() + 1).unwrap()
    } else {
        current_month.with_month(current_month.month() + 1).unwrap()
    }
}

pub fn to_end_of_month_with_hours(current_month: NaiveDateTime) -> NaiveDateTime {
	// current_month = 2021-06-01T00:00:00

    let days_in_month = days_in_month(current_month);

    current_month.with_day(days_in_month as u32).expect("unable to inc days")
    .with_hour(23).expect("unable to inc hours")
    .with_minute(59).expect("unable to inc minutes")
    .with_second(59).expect("unable to inc seconds")
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
