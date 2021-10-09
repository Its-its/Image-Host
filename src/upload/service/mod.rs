use image::ImageFormat;

use crate::{
	Filename,
	Result,
	config::{
		ConfigServiceB2,
		ConfigServiceFileSystem,
		ConfigServices
	},
	db::model::{
		SlimImage,
		User
	},
	feature::compress::compress_if_enabled,
	web::{
		ConfigDataService,
		WordDataService
	}
};

use super::image::UploadImageType;

pub mod b2;
pub mod filesystem;
pub mod log;

pub enum Service {
	B2(b2::Service),
	Log(log::Service),
	FileSystem(filesystem::Service),
}

impl Service {
	pub async fn pick_service_from_config(config: &ConfigServices) -> Result<Self> {
		let enabled_count = [
			config.logging.enabled,
			config.filesystem.enabled,
			config.b2.enabled,
		]
		.iter()
		.filter(|v| **v)
		.count();

		if enabled_count == 0 {
			panic!("Please enable a service.");
		} else if enabled_count > 1 {
			panic!("Only ONE service can be enabled at once currently.");
		}

		if config.logging.enabled {
			println!("Service Logging Enabled");
			return Ok(Self::new_log());
		}

		if config.filesystem.enabled {
			println!("Service Filesystem Enabled");
			return Self::new_file_system(&config.filesystem);
		}

		if config.b2.enabled {
			println!("Service B2 Enabled");
			return Self::new_b2(&config.b2).await;
		}

		unreachable!()
	}

	pub async fn new_b2(config: &ConfigServiceB2) -> Result<Self> {
		Ok(Self::B2(b2::Service::new(config).await?))
	}

	pub fn new_log() -> Self {
		Self::Log(log::Service)
	}

	pub fn new_file_system(config: &ConfigServiceFileSystem) -> Result<Self> {
		Ok(Self::FileSystem(filesystem::Service::new(config)?))
	}

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
		match self {
			Self::Log(v) => {
				v.process_files(user, file_type, file_data, content_type, ip_addr, config, words)
					.await
			}
			Self::B2(v) => {
				v.process_files(user, file_type, file_data, content_type, ip_addr, config, words)
					.await
			}
			Self::FileSystem(v) => {
				v.process_files(user, file_type, file_data, content_type, config, words)
					.await
			}
		}
	}

	pub async fn hide_file(&self, file_name: Filename) -> Result<()> {
		match self {
			Self::Log(v) => v.hide_file(file_name),
			Self::B2(v) => v.hide_file(file_name).await,
			Self::FileSystem(v) => v.hide_file(file_name).await,
		}
	}
}

pub async fn process_image_and_create_icon(
	file_name: &Filename,
	image_data: Vec<u8>,
	config: &ConfigDataService,
) -> Result<FileData> {
	let image = image::load_from_memory(&image_data)?;
	let icon = image.thumbnail_exact(128, 128);

	let mut icon_data = Vec::new();
	icon.write_to(&mut icon_data, ImageFormat::Png)?;

	let image_data = compress_if_enabled(file_name, image_data, image, config)?;

	Ok(FileData {
		image_name: file_name.as_filename(),
		image_data,

		icon_name: format!("{}.png", file_name.name),
		icon_data,
	})
}

pub struct FileData {
	image_name: String,
	image_data: Vec<u8>,

	icon_name: String,
	icon_data: Vec<u8>,
}
