use std::path::PathBuf;

use actix_web::error as web_error;

use crate::config::ConfigServiceFileSystem;
use crate::{WordManager, db};
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

	pub async fn process_files(&mut self, file_data: Vec<u8>, content_type: String, words: &mut WordManager) -> Result<()> {
		// Directory check
		if tokio::fs::metadata(&self.directory).await.is_err() {
			tokio::fs::create_dir_all(&self.directory).await?;
		}

		let file_name = words.get_next_filename_prefix_suffix(&db::get_images_collection()).await?;

		let file_name = file_name.set_format(content_type);

		if !file_name.is_accepted() {
			return Err(web_error::ErrorNotAcceptable("Invalid file format. Expected gif, png, or jpeg.").into());
		}

		let data = image_compress_and_create_icon(&file_name, file_data).await?;

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

		Ok(())
	}
}

