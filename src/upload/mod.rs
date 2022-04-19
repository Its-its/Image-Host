use crate::{Filename, Result, db::{ImagesCollection, model::User}, web::WordDataService};

use self::image::UploadImageType;

pub mod image;
pub mod service;


pub struct UploadProcessData {
	pub user: User,
	pub file_type: Option<UploadImageType>,
	pub file_data: Vec<u8>,
	pub content_type: String,
	pub ip_addr: String
}

impl UploadProcessData {
	pub async fn get_file_name(&self, image_icon_same_dir: bool, words: &WordDataService, collection: &ImagesCollection) -> Result<Filename> {
		let mut words = words.lock()?;

		if let Some(upload_type) = self.file_type {
			upload_type.get_link_name(&mut *words, image_icon_same_dir, collection).await?
		} else {
			self.user.upload_type
				.get_link_name(&mut *words, image_icon_same_dir, collection)
				.await?
		}.set_format(self.content_type.clone())
	}
}