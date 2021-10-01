use std::panic::catch_unwind;

use image::{ColorType, ImageFormat};

use crate::{
	config::{ConfigServiceB2, ConfigServiceFileSystem, ConfigServices},
	db::model::{SlimImage, User},
	Filename, Result, WordManager,
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

	pub async fn process_files(
		&mut self,
		user: User,
		file_type: Option<UploadImageType>,
		file_data: Vec<u8>,
		content_type: String,
		ip_addr: String,
		words: &mut WordManager,
	) -> Result<SlimImage> {
		match self {
			Self::Log(v) => {
				v.process_files(user, file_type, file_data, content_type, ip_addr, words)
					.await
			}
			Self::B2(v) => {
				v.process_files(user, file_type, file_data, content_type, ip_addr, words)
					.await
			}
			Self::FileSystem(v) => {
				v.process_files(user, file_type, file_data, content_type, words)
					.await
			}
		}
	}

	pub async fn hide_file(&mut self, file_name: Filename) -> Result<()> {
		match self {
			Self::Log(v) => v.hide_file(file_name),
			Self::B2(v) => v.hide_file(file_name).await,
			Self::FileSystem(v) => v.hide_file(file_name).await,
		}
	}
}

pub async fn image_compress_and_create_icon(
	file_name: &Filename,
	image_data: Vec<u8>,
) -> Result<FileData> {
	let image = image::load_from_memory(&image_data)?;
	let icon = image.thumbnail_exact(128, 128);

	let mut icon_data = Vec::new();
	icon.write_to(&mut icon_data, ImageFormat::Png)?;

	let image_data_new = if file_name.mime_format() == Some(mime::IMAGE_PNG) {
		drop(image);

		oxipng::optimize_from_memory(
			&image_data,
			&oxipng::Options {
				strip: oxipng::Headers::Safe,
				..Default::default()
			},
		)?
	} else if file_name.mime_format() == Some(mime::IMAGE_JPEG)
		&& (image.color() == ColorType::Rgb8 || image.color() == ColorType::Rgb16)
	{
		let res = catch_unwind(|| {
			drop(image);

			// Decode it.
			let (width, height, pixels) =
				match mozjpeg::Decompress::new_mem(&image_data)?.image()? {
					mozjpeg::Format::RGB(mut d) => (
						d.width(),
						d.height(),
						d.read_scanlines::<[u8; 3]>().unwrap(),
					),
					mozjpeg::Format::Gray(_) => unimplemented!(),
					mozjpeg::Format::CMYK(_) => unimplemented!(),
				};

			// Re-encode it.
			let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
			comp.set_mem_dest();
			comp.set_size(width, height);
			comp.set_quality(80.0);
			// comp.set_scan_optimization_mode(mozjpeg::ScanMode::Auto);

			comp.start_compress();

			comp.write_scanlines(bytemuck::cast_slice(&pixels));

			comp.finish_compress();

			Result::Ok(comp.data_to_vec().unwrap())
		});

		match res {
			Ok(v) => v?,
			// TODO: Output Error.
			Err(_e) => {
				return Err(crate::error::InternalError::MozJpegError.into());
			}
		}
	} else {
		if file_name.mime_format() == Some(mime::IMAGE_JPEG) {
			println!("Unknown JPEG Color: {:?}", image.color());
		}

		let mut w = Vec::new();
		image.write_to(
			&mut w,
			ImageFormat::from_extension(file_name.format()).unwrap(),
		)?;
		w
	};

	// Pick smallest image data size.
	let image_data = if image_data.len() < image_data_new.len() {
		image_data
	} else {
		image_data_new
	};

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
