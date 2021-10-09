use actix_web::error as web_error;
use mongodb::bson::DateTime;

use crate::db::model::{SlimImage, User};
use crate::error::Result;
use crate::upload::image::UploadImageType;
use crate::web::{ConfigDataService, WordDataService};
use crate::{db, Filename};

use super::process_image_and_create_icon;

#[derive(Default)]
pub struct Service;

impl Service {
	pub async fn process_files(
		&self,
		user: User,
		file_type: Option<UploadImageType>,
		file_data: Vec<u8>,
		content_type: String,
		ip_addr: String,
		config: &ConfigDataService,
		words: &WordDataService,
	) -> Result<SlimImage> {
		let collection = db::get_images_collection();

		let file_name = {
			let mut words = words.lock()?;

			if let Some(upload_type) = file_type {
				upload_type.get_link_name(&mut *words, false, &collection).await?
			} else {
				user.upload_type
					.get_link_name(&mut *words, false, &collection)
					.await?
			}
		};

		let file_name = file_name.set_format(content_type);

		if !file_name.is_accepted() {
			return Err(web_error::ErrorNotAcceptable(
				"Invalid file format. Expected gif, png, or jpeg.",
			)
			.into());
		}

		let size_original = file_data.len() as i64;

		let data = process_image_and_create_icon(&file_name, file_data, config).await?;

		let size_compressed = data.image_data.len() as i64;

		println!(
			"[LOG]: User Uploaded Image UID: {}, IP: {}",
			user.id, ip_addr
		);
		println!("[LOG]: \tImage original size: {} bytes", size_original);
		println!("[LOG]: \tImage compressed size: {} bytes", size_compressed);
		println!(
			"[LOG]: \tImage Info: \"{}\" = {} bytes",
			data.image_name,
			data.image_data.len()
		);
		println!(
			"[LOG]: \tIcon Info: \"i{}\" = {} bytes",
			data.icon_name,
			data.icon_data.len()
		);

		Ok(SlimImage {
			custom_name: None,
			file_type: file_name.format().to_string(),
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
