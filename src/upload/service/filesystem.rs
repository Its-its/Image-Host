use std::path::PathBuf;

use actix_web::error as web_error;
use mongodb::bson::DateTime;

use crate::config::ConfigServiceFileSystem;
use crate::db::model::{SlimImage, User};
use crate::error::Result;
use crate::upload::image::UploadImageType;
use crate::{db, Filename, WordManager};

use super::image_compress_and_create_icon;

pub struct Service {
	image_sub_directory: PathBuf,
	icon_sub_directory: PathBuf,
}

impl Service {
	pub fn new(config: &ConfigServiceFileSystem) -> Result<Self> {
		let mut image_sub_directory = PathBuf::from(config.upload_directory.clone());
		let mut icon_sub_directory = image_sub_directory.clone();
		icon_sub_directory.push(&config.icon_sub_directory);

		image_sub_directory.push(&config.image_sub_directory);

		Ok(Self {
			image_sub_directory,
			icon_sub_directory,
		})
	}

	pub async fn process_files(
		&mut self,
		user: User,
		file_type: Option<UploadImageType>,
		file_data: Vec<u8>,
		content_type: String,
		words: &mut WordManager,
	) -> Result<SlimImage> {
		let same_dirs = self.image_sub_directory == self.icon_sub_directory;

		// Directory check
		if tokio::fs::metadata(&self.image_sub_directory)
			.await
			.is_err()
		{
			tokio::fs::create_dir_all(&self.image_sub_directory).await?;
		}

		if !same_dirs && tokio::fs::metadata(&self.icon_sub_directory).await.is_err() {
			tokio::fs::create_dir_all(&self.icon_sub_directory).await?;
		}

		let collection = db::get_images_collection();

		let file_name = if let Some(upload_type) = file_type {
			upload_type.get_link_name(words, &collection).await?
		} else {
			user.upload_type
				.get_link_name(words, &collection)
				.await?
		};

		let file_name = file_name.set_format(content_type);

		if !file_name.is_accepted() {
			return Err(web_error::ErrorNotAcceptable(
				"Invalid file format. Expected gif, png, or jpeg.",
			)
			.into());
		}

		let size_original = file_data.len() as i64;

		let data = image_compress_and_create_icon(&file_name, file_data).await?;

		let size_compressed = data.image_data.len() as i64;

		{
			let mut path = self.image_sub_directory.clone();
			path.push(data.image_name);

			tokio::fs::write(path, data.image_data).await?;
		}

		{
			let mut path = self.icon_sub_directory.clone();
			path.push(if same_dirs {
				format!("i{}", data.icon_name)
			} else {
				data.icon_name
			});

			tokio::fs::write(path, data.icon_data).await?;
		}

		Ok(SlimImage {
			custom_name: None,
			name: file_name.name().to_string(),
			file_type: file_name.format().to_string(),
			size_original,
			size_compressed,
			is_edited: false,
			is_favorite: false,
			view_count: 0,
			upload_date: DateTime::now(),
		})
	}

	pub async fn hide_file(&mut self, file_name: Filename) -> Result<()> {
		{
			let mut path = self.image_sub_directory.clone();
			path.push(file_name.as_filename());

			tokio::fs::remove_file(path).await?;
		}

		{
			let mut path = self.icon_sub_directory.clone();
			path.push(if self.image_sub_directory == self.icon_sub_directory {
				format!("i{}", file_name.name())
			} else {
				file_name.into_name()
			});

			tokio::fs::remove_file(path).await?;
		}

		Ok(())
	}
}
