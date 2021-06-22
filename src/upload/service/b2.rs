use std::path::PathBuf;
use std::time::Duration;

use actix_web::error as web_error;
use crypto::sha1::Sha1;
use crypto::digest::Digest;
use base64::encode as b64encode;
use mongodb::bson::DateTime;

use crate::config::ConfigServiceB2;
use crate::db::model;
use crate::{WordManager, db};
use crate::error::{InternalError, Result};

use super::image_compress_and_create_icon;

// const API_URL_V5: &str = "https://api.backblazeb2.com/b2api/v5";
// const API_URL_V4: &str = "https://api.backblazeb2.com/b2api/v4";
// const API_URL_V3: &str = "https://api.backblazeb2.com/b2api/v3";
const API_URL_V2: &str = "https://api.backblazeb2.com/b2api/v2";
// const API_URL_V1: &str = "https://api.backblazeb2.com/b2api/v1";





pub struct Service {
	credentials: Credentials,
	auth: B2Authorization,

	bucket_id: String,

	image_sub_directory: PathBuf,
	icon_sub_directory: PathBuf
}

impl Service {
	pub async fn new(config: &ConfigServiceB2) -> Result<Self> {
		if config.id.is_empty() {
			panic!("B2 Service ID is empty.");
		}

		if config.key.is_empty() {
			panic!("B2 Service Key is empty.");
		}

		if config.bucket_id.is_empty() {
			panic!("B2 Service Bucked ID is empty.");
		}

		let credentials = Credentials::new(&config.id, &config.key);
		let auth = credentials.authorize().await?;

		Ok(Self {
			credentials,
			auth,

			bucket_id: config.bucket_id.clone(),

			image_sub_directory: PathBuf::from(&config.image_sub_directory),
			icon_sub_directory: PathBuf::from(&config.icon_sub_directory)
		})
	}

	pub async fn process_files(&mut self, uid: String, file_data: Vec<u8>, content_type: String, ip_addr: String, words: &mut WordManager) -> Result<()> {
		let collection = db::get_images_collection();

		let file_name = words.get_next_filename_prefix_suffix(&collection).await?;

		let file_name = file_name.set_format(content_type);

		if !file_name.is_accepted() {
			return Err(web_error::ErrorNotAcceptable("Invalid file format. Expected gif, png, or jpeg.").into());
		}

		let file_size = file_data.len() as i64;

		let data = image_compress_and_create_icon(&file_name, file_data).await?;

		{ // Image Upload
			let mut path = self.image_sub_directory.clone();
			path.push(data.image_name);

			try_upload_file_multi(path.to_str().unwrap(), data.image_data, &self.auth, &self.bucket_id).await?;
		}

		{ // Icon Upload
			let mut path = self.icon_sub_directory.clone();
			path.push(data.icon_name);

			try_upload_file_multi(path.to_str().unwrap(), data.icon_data, &self.auth, &self.bucket_id).await?;
		}


		let new_image = model::Image {
			id: None,

			name: file_name.name().to_string(),
			file_type: file_name.format().to_string(),
			file_size,

			is_edited: false,
			is_favorite: false,
			view_count: 0,

			uploader: model::ImageUploader {
				uid,
				ip: Some(ip_addr)
			},

			upload_date: DateTime::now(),

			encryption_code: None,
			tags: None,
			custom_name: None,
		};

		println!("{:#?}", new_image);

		let res = new_image.upload(&collection).await?;
		println!("Insert: {:?}", res);

		Ok(())
	}
}




async fn try_upload_file_multi(file_name: &str, image_buffer: Vec<u8>, auth: &B2Authorization, bucket_id: &str) -> Result<()> {
	let mut prev_error = None;


	for _ in 0..5 {
		// For Some reason getting the upload url errors.
		let upload_url = match auth.get_upload_url(bucket_id).await {
			Ok(v) => v,
			Err(e) => {
				prev_error = Some(e);
				tokio::time::sleep(Duration::from_millis(1000)).await;
				continue;
			}
		};

		// TODO: Remove clone()
		if let Err(e) = auth.upload_file(&upload_url, file_name, image_buffer.clone()).await {
			prev_error = Some(e);
			tokio::time::sleep(Duration::from_millis(1000)).await;
			continue;
		}
	}

	Err(prev_error.unwrap())
}


pub struct Credentials {
	pub id: String,
	pub key: String
}

impl Credentials {
	pub fn new<S: Into<String>>(id: S, key: S) -> Self {
		Self {
			id: id.into(),
			key: key.into()
		}
	}

	fn header_name(&self) -> &str {
		"Authorization"
	}

	fn id_key(&self) -> String {
		format!("{}:{}", self.id, self.key)
	}

	pub fn auth_string(&self) -> String {
		format!("Basic {}", b64encode(&self.id_key()))
	}

	pub async fn authorize(&self) -> Result<B2Authorization> {
		let client = reqwest::Client::new();

		let resp = client.get(format!("{}/b2_authorize_account", API_URL_V2).as_str())
			.header(self.header_name(), self.auth_string())
			.send()
			.await?;

		if resp.status().is_success() {
			Ok(B2Authorization::new(self.id.clone(), resp.json().await?))
		} else {
			println!("{:#?}", resp.text().await?);
			Err(InternalError::B2Authorization.into())
		}
	}
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct B2AuthResponse {
	// account_id: String,
	// allowed: Object,
	absolute_minimum_part_size: usize,
	api_url: String,
	authorization_token: String,
	download_url: String,
	recommended_part_size: usize,
}


#[derive(Debug, Clone)]
pub struct B2Authorization {
	pub account_id: String,
	pub authorization_token: String,
	pub api_url: String,
	pub download_url: String,
	pub recommended_part_size: usize,
	pub absolute_minimum_part_size: usize
}

impl B2Authorization {
	fn new(id: String, resp: B2AuthResponse) -> B2Authorization {
		B2Authorization {
			account_id: id,
			authorization_token: resp.authorization_token,
			api_url: resp.api_url,
			download_url: resp.download_url,
			recommended_part_size: resp.recommended_part_size,
			absolute_minimum_part_size: resp.absolute_minimum_part_size
		}
	}

	pub async fn get_upload_url(&self, bucket_id: &str) -> Result<UploadUrlResponse> {
		let client = reqwest::Client::new();

		let body = serde_json::json!({
			"bucketId": bucket_id
		});

		let resp = client.post(format!("{}/b2api/v2/b2_get_upload_url", self.api_url).as_str())
			.header("Authorization", self.authorization_token.as_str())
			.body(serde_json::to_string(&body)?)
			.send()
			.await?;

		if resp.status().is_success() {
			Ok(resp.json().await?)
		} else {
			println!("get_upload_url: {:?}", resp.text().await?);
			Err(InternalError::B2GetUploadUrl.into())
		}
	}

	pub async fn upload_file(&self, upload: &UploadUrlResponse, file_name: &str, image: Vec<u8>) -> Result<serde_json::Value> {
		let client = reqwest::Client::new();

		let mut sha = Sha1::new();
		sha.input(image.as_ref());
		let sha = sha.result_str();

		println!("Size: {}", image.len());
		println!("Sha1: {}", sha);

		let resp = client.post(upload.upload_url.as_str())
			.header("Authorization", upload.authorization_token.as_str())
			.header("Content-Type", "b2/x-auto")
			.header("Content-Length", image.len())
			.header("X-Bz-File-Name", encode_file_name(file_name).as_str())
			.header("X-Bz-Content-Sha1", sha.as_str())
			.body(image)
			.send()
			.await?;

		if resp.status().is_success() {
			Ok(resp.json().await?)
		} else {
			println!("upload_file: {:?}", resp.text().await?);
			Err(InternalError::B2UploadFile.into())
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadUrlResponse {
	authorization_token: String,
	bucket_id: String,
	upload_url: String
}


// Names can be pretty much any UTF-8 string up to 1024 bytes long. There are a few picky rules:
// No character codes below 32 are allowed.
// Backslashes are not allowed.
// DEL characters (127) are not allowed.
// File names cannot start with "/", end with "/", or contain "//".

pub fn encode_file_name(file_name: &str) -> String {
	file_name.replace("\\", "-")
	.replace(" ", "%20")
}