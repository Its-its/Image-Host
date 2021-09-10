use actix_web::error as web_error;
use mongodb::bson::DateTime;

use crate::db::model::{SlimImage, User};
use crate::upload::image::UploadImageType;
use crate::{Filename, WordManager, db};
use crate::error::Result;

use super::image_compress_and_create_icon;


#[derive(Default)]
pub struct Service;

impl Service {
	pub async fn process_files(&mut self, user: User, file_type: Option<UploadImageType>, file_data: Vec<u8>, content_type: String, ip_addr: String, words: &mut WordManager) -> Result<SlimImage> {
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

		let orig_file_size = file_data.len() as i64;

		let data = image_compress_and_create_icon(&file_name, file_data).await?;

		println!("[LOG]: User Uploaded Image UID: {}, IP: {}", user.id, ip_addr);
		println!("[LOG]: \tImage original size: {} bytes", orig_file_size);
		println!("[LOG]: \tImage Info: \"{}\" = {} bytes", data.image_name, data.image_data.len());
		println!("[LOG]: \tIcon Info: \"i{}\" = {} bytes", data.icon_name, data.icon_data.len());

		Ok(SlimImage {
			custom_name: None,
			name: file_name.name().to_string(),
			file_type: file_name.format().to_string(),
			file_size: data.image_data.len() as i64,
			is_edited: false,
			is_favorite: false,
			view_count: 0,
			upload_date: DateTime::now(),
		})
	}

	pub fn hide_file(&mut self, file_name: Filename) -> Result<()> {
		println!("[LOG]: Removing File Name {:?}", file_name);

		Ok(())
	}
}