use std::{path::PathBuf, str::FromStr};

use actix_files::NamedFile;
use actix_http::header;
use actix_service::ServiceFactory;
use actix_web::{
	dev::ServiceRequest,
	guard, web, App, HttpRequest, HttpResponse,
};
use reqwest::Url;

use crate::{config::Config, Result, error::Error};

use super::ConfigDataService;

// If both urls are the same then icons use LOWERCASE 'i' to differentiate it from its' original.
pub fn create_services<T: ServiceFactory<ServiceRequest, Config = (), Error = actix_web::Error, InitError = ()>>(
	app: App<T>,
	image_url: String,
	icon_url: String,
	read: &Config,
) -> Result<App<T>> {
	let image_url_header = header::HeaderValue::from_str(&image_url)
		.map_err(|_| Error::ActixInvalidHeaderValue(icon_url.to_string()))?;

	let image_factory = web::scope("").guard(guard::fn_guard(move |req| {
		(|| -> Option<bool> {
			let host = req.head().headers().get(header::HOST)?;
			Some(host == image_url_header)
		})()
		.unwrap_or_default()
	}));

	let apples = if image_url == icon_url {
		if read.services.b2.enabled {
			app.service(image_factory.route(
				"/{name}",
				web::to(|mut name: web::Path<String>, config: ConfigDataService| async move {
					let url = {
						let read = config.read()?;

						let input = if name.starts_with('i') {
							// We don't prepend 'i' if the icon dir is different than the image one.
							if read.services.b2.icon_sub_directory != read.services.b2.image_sub_directory {
								name.remove(0);
							}

							&read.services.b2.icon_sub_directory
						} else {
							&read.services.b2.image_sub_directory
						};

						Url::from_str(&read.services.b2.public_url)?
							.join(&format!("{}/{}", input, &name))?
					};

					match reqwest::get(url).await {
						Ok(v) => Result::Ok(HttpResponse::Ok().streaming(v.bytes_stream())),
						Err(_) => Result::Ok(HttpResponse::NotFound().finish()),
					}
				}),
			))
		} else if read.services.filesystem.enabled {
			app.service(image_factory.route(
				"/{name}",
				web::get().to(
					|name: web::Path<String>, config: ConfigDataService, req: HttpRequest| async move {
						if name.is_empty() {
							Result::Ok(HttpResponse::NotFound().finish())
						} else {
							let read = config.read()?;

							let mut path = PathBuf::new();
							path.push(&read.services.filesystem.upload_directory);

							if name.starts_with('i') {
								path.push(&read.services.filesystem.icon_sub_directory);
							} else {
								path.push(&read.services.filesystem.image_sub_directory);
							}

							path.push(name.into_inner());

							match NamedFile::open_async(path).await {
								Ok(v) => Result::Ok(v.into_response(&req)),
								Err(_) => Result::Ok(HttpResponse::NotFound().finish()),
							}
						}
					},
				),
			))
		} else {
			app
		}
	} else {
		async fn b2_image_route(name: web::Path<String>, config: ConfigDataService) -> Result<HttpResponse> {
			let url = {
				let read = config.read()?;

				Url::from_str(&read.services.b2.public_url)?
					.join(&format!(
						"{}/{}",
						&read.services.b2.image_sub_directory, &name
					))?
			};

			match reqwest::get(url).await {
				Ok(v) => Ok(HttpResponse::Ok().streaming(v.bytes_stream())),
				Err(_) => Ok(HttpResponse::NotFound().finish()),
			}
		}

		async fn b2_icon_route(name: web::Path<String>, config: ConfigDataService) -> Result<HttpResponse> {
			let url = {
				let read = config.read()?;

				Url::from_str(&read.services.b2.public_url)?
					.join(&format!(
						"{}/{}",
						&read.services.b2.icon_sub_directory, &name
					))?
			};

			match reqwest::get(url).await {
				Ok(v) => Ok(HttpResponse::Ok().streaming(v.bytes_stream())),
				Err(_) => Ok(HttpResponse::NotFound().finish()),
			}
		}

		async fn fs_image_route(name: web::Path<String>, config: ConfigDataService, req: HttpRequest) -> Result<HttpResponse> {
			if name.is_empty() {
				Ok(HttpResponse::NotFound().finish())
			} else {
				let read = config.read()?;

				let mut path = PathBuf::new();
				path.push(&read.services.filesystem.upload_directory);
				path.push(&read.services.filesystem.image_sub_directory);
				path.push(name.into_inner());

				match NamedFile::open_async(path).await {
					Ok(v) => Ok(v.into_response(&req)),
					Err(_) => Ok(HttpResponse::NotFound().finish()),
				}
			}
		}

		async fn fs_icon_route(name: web::Path<String>, config: ConfigDataService, req: HttpRequest) -> Result<HttpResponse> {
			if name.is_empty() {
				Ok(HttpResponse::NotFound().finish())
			} else {
				let read = config.read()?;

				let mut path = PathBuf::new();
				path.push(&read.services.filesystem.upload_directory);
				path.push(&read.services.filesystem.icon_sub_directory);
				path.push(name.into_inner());

				match NamedFile::open_async(path).await {
					Ok(v) => Ok(v.into_response(&req)),
					Err(_) => Ok(HttpResponse::NotFound().finish()),
				}
			}
		}


		let icon_url_header = header::HeaderValue::from_str(&icon_url)
			.map_err(|_| Error::ActixInvalidHeaderValue(icon_url.to_string()))?;

		let icon_factory = web::scope("").guard(guard::fn_guard(move |req| {
			(|| -> Option<bool> {
				let host = req.head().headers().get(header::HOST)?;
				Some(host == icon_url_header)
			})()
			.unwrap_or_default()
		}));

		if read.services.b2.enabled {
			app.service(image_factory.route(
				"/{name}",
				web::to(b2_image_route),
			))
			.service(icon_factory.route(
				"/{name}",
				web::to(b2_icon_route),
			))
		} else if read.services.filesystem.enabled {
			app.service(image_factory.route(
				"/{name}",
				web::get().to(fs_image_route),
			))
			.service(icon_factory.route(
				"/{name}",
				web::get().to(fs_icon_route),
			))
		} else {
			app
		}
	};

	Ok(apples)
}