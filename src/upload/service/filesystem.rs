use std::path::PathBuf;

use actix_web::error as web_error;
use mongodb::bson::DateTime;

use crate::config::ConfigServiceFileSystem;
use crate::db::model::{SlimImage, User};
use crate::upload::image::UploadImageType;
use crate::{Filename, WordManager, db};
use crate::error::Result;

use super::image_compress_and_create_icon;


pub struct Service {
	directory: PathBuf
}

impl Service {
	pub fn new(config: &ConfigServiceFileSystem) -> Result<Self> {
		Ok(Self {
			directory: PathBuf::from(config.upload_directory.clone())
		})
	}

	pub async fn process_files(&mut self, user: User, file_type: Option<UploadImageType>, file_data: Vec<u8>, content_type: String, words: &mut WordManager) -> Result<SlimImage> {
		// Directory check
		if tokio::fs::metadata(&self.directory).await.is_err() {
			tokio::fs::create_dir_all(&self.directory).await?;
		}

		let collection = db::get_images_collection();

		let file_name = if let Some(upload_type) = file_type {
			upload_type.get_link_name(words, &collection).await?
		} else {
			user.data.upload_type.get_link_name(words, &collection).await?
		};

		let file_name = file_name.set_format(content_type);

		if !file_name.is_accepted() {
			return Err(web_error::ErrorNotAcceptable("Invalid file format. Expected gif, png, or jpeg.").into());
		}

		let data = image_compress_and_create_icon(&file_name, file_data).await?;

		let file_size = data.image_data.len();

		{
			let mut path = self.directory.clone();
			path.push(data.image_name);

			tokio::fs::write(path, data.image_data).await?;
		}

		{
			let mut path = self.directory.clone();
			path.push(data.icon_name);

			tokio::fs::write(path, data.icon_data).await?;
		}

		Ok(SlimImage {
			custom_name: None,
			name: file_name.name().to_string(),
			file_type: file_name.format().to_string(),
			file_size: file_size as i64,
			is_edited: false,
			is_favorite: false,
			view_count: 0,
			upload_date: DateTime::now(),
		})
	}

	pub async fn hide_file(&mut self, file_name: Filename) -> Result<()> {
		{
			let mut path = self.directory.clone();
			path.push(file_name.as_filename());

			tokio::fs::remove_file(path).await?;
		}

		{
			let mut path = self.directory.clone();
			path.push(format!("i{}.png", file_name.name()));

			tokio::fs::remove_file(path).await?;
		}

		Ok(())
	}
}

