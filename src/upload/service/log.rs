use actix_web::error as web_error;

use crate::{Filename, WordManager, db};
use crate::error::Result;

use super::image_compress_and_create_icon;


#[derive(Default)]
pub struct Service;

impl Service {
	pub async fn process_files(&mut self, uid: String, file_data: Vec<u8>, content_type: String, ip_addr: String, words: &mut WordManager) -> Result<()> {
		let file_name = words.get_next_filename_prefix_suffix(&db::get_images_collection()).await?;

		let file_name = file_name.set_format(content_type);

		if !file_name.is_accepted() {
			return Err(web_error::ErrorNotAcceptable("Invalid file format. Expected gif, png, or jpeg.").into());
		}

		let orig_file_size = file_data.len() as i64;

		let data = image_compress_and_create_icon(&file_name, file_data).await?;

		println!("[LOG]: User Uploaded Image UID: {}, IP: {}", uid, ip_addr);
		println!("[LOG]: \tImage original size: {} bytes", orig_file_size);
		println!("[LOG]: \tImage Info: {:?} = {} bytes", data.image_name, data.image_data.len());
		println!("[LOG]: \tIcon Info: {:?} = {} bytes", data.icon_name, data.icon_data.len());

		Ok(())
	}

	pub fn hide_file(&mut self, file_name: Filename) -> Result<()> {
		println!("[LOG]: Removing File Name {:?}", file_name);

		Ok(())
	}
}