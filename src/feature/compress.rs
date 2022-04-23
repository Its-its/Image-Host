use std::panic::catch_unwind;

use image::{ColorType, DynamicImage};

use crate::{Filename, Result, web::ConfigDataService, error::{InternalError, Error}};



pub fn compress_if_enabled(file_name: &Filename, image_data: Vec<u8>, image: DynamicImage, config: &ConfigDataService) -> Result<Vec<u8>> {
	{
		let config = config.read()?;
		if !config.features.compression.enabled {
			return Ok(image_data);
		}
	}

	if file_name.is_format(mime::IMAGE_PNG) {
		drop(image);

		let new_data = oxipng::optimize_from_memory(
			&image_data,
			&oxipng::Options {
				strip: oxipng::Headers::Safe,
				..Default::default()
			},
		)?;

		// Pick smallest image data size.
		if new_data.len() < image_data.len() {
			Result::Ok(new_data)
		} else {
			Result::Ok(image_data)
		}
	} else if file_name.is_format(mime::IMAGE_JPEG)
		&& (image.color() == ColorType::Rgb8 || image.color() == ColorType::Rgb16)
	{
		drop(image);

		let res = catch_unwind(|| {
			// Decode it.
			let (width, height, pixels) =
				match mozjpeg::Decompress::new_mem(&image_data)?.image()? {
					mozjpeg::Format::RGB(mut d) => (
						d.width(),
						d.height(),
						d.read_scanlines::<[u8; 3]>().ok_or_else(|| Error::from(InternalError::MozJpegScanLines))?,
					),
					mozjpeg::Format::Gray(_) => return Result::Err(InternalError::MozJpegUnimplementedFormat.into()),
					mozjpeg::Format::CMYK(_) => return Result::Err(InternalError::MozJpegUnimplementedFormat.into()),
				};

			// Re-encode it.
			let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
			comp.set_mem_dest();
			comp.set_size(width, height);

			{
				let read = config.read()?;
				comp.set_quality(read.features.compression.quality);
			}
			// comp.set_scan_optimization_mode(mozjpeg::ScanMode::Auto);

			comp.start_compress();

			comp.write_scanlines(bytemuck::cast_slice(&pixels));

			comp.finish_compress();

			let new_data = comp.data_to_vec().map_err(|_| Error::from(InternalError::MozJpegDataRetrive))?;

			// Pick smallest image data size.
			if new_data.len() < image_data.len() {
				Result::Ok(new_data)
			} else {
				Result::Ok(image_data)
			}
		});

		match res {
			Ok(v) => Ok(v?),
			// TODO: Output Error.
			Err(_e) => {
				Err(crate::error::InternalError::MozJpegError.into())
			}
		}
	} else {
		if file_name.is_format(mime::IMAGE_JPEG) {
			println!("Unknown JPEG Color: {:?}", image.color());
		}

		Ok(image_data)
	}
}