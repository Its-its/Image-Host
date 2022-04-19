use mongodb::bson::DateTime;

use crate::db::model::SlimImage;
use crate::error::Result;
use crate::upload::UploadProcessData;
use crate::web::{ConfigDataService, WordDataService};
use crate::{db, Filename};

use super::process_image_and_create_icon;

#[derive(Default)]
pub struct Service;

impl Service {
	pub async fn process_files(
		&self,
		upload_data: UploadProcessData,
		config: &ConfigDataService,
		words: &WordDataService,
	) -> Result<SlimImage> {
		let collection = db::get_images_collection();

		let file_name = upload_data.get_file_name(false, words, &collection)
			.await?;

		let size_original = upload_data.file_data.len() as i64;

		let file_data = process_image_and_create_icon(&file_name, upload_data.file_data, config).await?;

		let size_compressed = file_data.image_data.len() as i64;

		println!(
			"[LOG]: User Uploaded Image UID: {}, IP: {}",
			upload_data.user.id, upload_data.ip_addr
		);
		println!("[LOG]: \tImage original size: {} bytes", size_original);
		println!("[LOG]: \tImage compressed size: {} bytes", size_compressed);
		println!(
			"[LOG]: \tImage Info: \"{}\" = {} bytes",
			file_data.image_name,
			file_data.image_data.len()
		);
		println!(
			"[LOG]: \tIcon Info: \"i{}\" = {} bytes",
			file_data.icon_name,
			file_data.icon_data.len()
		);

		Ok(SlimImage {
			custom_name: None,
			file_type: file_name.format_name()?.to_string(),
			name: file_name.name,
			size_original,
			size_compressed,
			is_edited: false,
			is_favorite: false,
			view_count: 0,
			upload_date: DateTime::now(),
		})
	}

	pub fn hide_file(&self, file_name: Filename) -> Result<()> {
		println!("[LOG]: Removing File Name {:?}", file_name);

		Ok(())
	}
}
