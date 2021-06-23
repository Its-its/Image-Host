use image::ImageFormat;

use crate::{
	Filename,
	Result,
	WordManager,

	config::{
		ConfigServiceB2,
		ConfigServiceFileSystem,
		ConfigServices
	}
};


pub mod b2;
pub mod log;
pub mod filesystem;



pub enum Service {
	B2(b2::Service),
	Log(log::Service),
	FileSystem(filesystem::Service)
}

impl Service {
	pub async fn pick_service_from_config(config: &ConfigServices) -> Result<Self> {
		let enabled_count = [config.logging.enabled, config.filesystem.enabled, config.b2.enabled].iter().filter(|v| **v).count();

		if enabled_count == 0 {
			panic!("Please enable a service.");
		} else if enabled_count > 1 {
			panic!("Only ONE service can be enabled at once currently.");
		}

		if config.logging.enabled {
			println!("Enabling Logging Only");
			return Ok(Self::new_log());
		}

		if config.filesystem.enabled {
			println!("Enabling Filesystem uploading");
			return Self::new_file_system(&config.filesystem);
		}

		if config.b2.enabled {
			println!("Enabling B2 Uploading");
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


	pub async fn process_files(&mut self, uid: String, file_data: Vec<u8>, content_type: String, ip_addr: String, words: &mut WordManager) -> Result<()> {
		match self {
			Self::Log(v) => v.process_files(uid, file_data, content_type, ip_addr, words).await,
			Self::B2(v) => v.process_files(uid, file_data, content_type, ip_addr, words).await,
			Self::FileSystem(v) => v.process_files(file_data, content_type, words).await
		}
	}

	pub async fn hide_file(&mut self, file_name: &str) -> Result<()> {
		match self {
			Self::Log(v) => v.hide_file(file_name),
			Self::B2(v) => v.hide_file(file_name).await,
			Self::FileSystem(v) => v.hide_file(file_name).await
		}
	}
}


pub async fn image_compress_and_create_icon(file_name: &Filename, image_data: Vec<u8>) -> Result<FileData> {
	// TODO: Compress. https://github.com/mozilla/mozjpeg

	let image = image::load_from_memory(&image_data)?;
	let icon = image.thumbnail_exact(128, 128);
	//Node.js uses: .resize(128, 128, FilterType::CatmullRom);

	let mut icon_data = Vec::new();
	icon.write_to(&mut icon_data, ImageFormat::Png)?;

	let mut image_data_new = Vec::new();
	image.write_to(&mut image_data_new, ImageFormat::from_extension(file_name.format()).unwrap())?;

	// Pick smallest image data size.
	let image_data = if image_data < image_data_new {
		image_data
	} else {
		image_data_new
	};

	Ok(FileData {
		image_name: file_name.as_filename(),
		image_data,

		icon_name: format!("i{}.png", file_name.name()),
		icon_data,
	})
}


pub struct FileData {
	image_name: String,
	image_data: Vec<u8>,

	icon_name: String,
	icon_data: Vec<u8>
}