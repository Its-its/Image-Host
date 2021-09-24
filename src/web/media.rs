use std::{path::PathBuf, str::FromStr};

use actix_files::NamedFile;
use actix_http::{body::MessageBody, header};
use actix_service::ServiceFactory;
use actix_web::{
	dev::{ServiceRequest, ServiceResponse},
	guard, web, App, HttpRequest, HttpResponse,
};
use reqwest::Url;

use crate::config::Config;

use super::ConfigDataService;

// If both urls are the same then icons use LOWERCASE 'i' to differentiate it from its' original.
pub fn create_services<
	T: ServiceFactory<
		ServiceRequest,
		Response = ServiceResponse<B>,
		Error = actix_web::Error,
		InitError = (),
		Config = (),
	>,
	B: MessageBody,
>(
	app: App<T, B>,
	image_url: String,
	icon_url: String,
	read: &Config,
) -> App<T, B> {
	let image_url_header = header::HeaderValue::from_str(&image_url).unwrap();

	let image_factory = web::scope("").guard(guard::fn_guard(move |req| {
		(|| -> Option<bool> {
			let host = req.headers().get(header::HOST)?;
			Some(host == image_url_header)
		})()
		.unwrap_or_default()
	}));

	if image_url == icon_url {
		if read.services.b2.enabled {
			app.service(image_factory.route(
				"/{name}",
				web::to(move |name: web::Path<String>, config: ConfigDataService| {
					let url = {
						let read = config.read().unwrap();

						let input = if name.starts_with('i') {
							&read.services.b2.icon_sub_directory
						} else {
							&read.services.b2.image_sub_directory
						};

						Url::from_str(&read.services.b2.public_url)
							.unwrap()
							.join(&format!("{}/{}", input, &name))
							.unwrap()
					};

					#[allow(clippy::async_yields_async)]
					async {
						match reqwest::get(url).await {
							Ok(v) => HttpResponse::Ok().streaming(v.bytes_stream()),
							Err(_) => HttpResponse::NotFound().finish(),
						}
					}
				}),
			))
		} else if read.services.filesystem.enabled {
			app.service(image_factory.route(
				"/{name}",
				web::get().to(
					move |name: web::Path<String>, config: ConfigDataService, req: HttpRequest| {
						if name.is_empty() {
							HttpResponse::NotFound().finish()
						} else {
							let read = config.read().unwrap();

							let mut path = PathBuf::new();
							path.push(&read.services.filesystem.upload_directory);

							if name.starts_with('i') {
								path.push(&read.services.filesystem.icon_sub_directory);
							} else {
								path.push(&read.services.filesystem.image_sub_directory);
							}

							path.push(name.into_inner());

							match NamedFile::open(path) {
								Ok(v) => v.into_response(&req),
								Err(_) => HttpResponse::NotFound().finish(),
							}
						}
					},
				),
			))
		} else {
			app
		}
	} else {
		let icon_url_header = header::HeaderValue::from_str(&icon_url).unwrap();

		let icon_factory = web::scope("").guard(guard::fn_guard(move |req| {
			(|| -> Option<bool> {
				let host = req.headers().get(header::HOST)?;
				Some(host == icon_url_header)
			})()
			.unwrap_or_default()
		}));

		if read.services.b2.enabled {
			app.service(image_factory.route(
				"/{name}",
				web::to(move |name: web::Path<String>, config: ConfigDataService| {
					let url = {
						let read = config.read().unwrap();

						Url::from_str(&read.services.b2.public_url)
							.unwrap()
							.join(&format!(
								"{}/{}",
								&read.services.b2.image_sub_directory, &name
							))
							.unwrap()
					};

					#[allow(clippy::async_yields_async)]
					async {
						match reqwest::get(url).await {
							Ok(v) => HttpResponse::Ok().streaming(v.bytes_stream()),
							Err(_) => HttpResponse::NotFound().finish(),
						}
					}
				}),
			))
			.service(icon_factory.route(
				"/{name}",
				web::to(move |name: web::Path<String>, config: ConfigDataService| {
					let url = {
						let read = config.read().unwrap();

						Url::from_str(&read.services.b2.public_url)
							.unwrap()
							.join(&format!(
								"{}/{}",
								&read.services.b2.icon_sub_directory, &name
							))
							.unwrap()
					};

					#[allow(clippy::async_yields_async)]
					async {
						match reqwest::get(url).await {
							Ok(v) => HttpResponse::Ok().streaming(v.bytes_stream()),
							Err(_) => HttpResponse::NotFound().finish(),
						}
					}
				}),
			))
		} else if read.services.filesystem.enabled {
			app.service(image_factory.route(
				"/{name}",
				web::get().to(
					move |name: web::Path<String>, config: ConfigDataService, req: HttpRequest| {
						if name.is_empty() {
							HttpResponse::NotFound().finish()
						} else {
							let read = config.read().unwrap();

							let mut path = PathBuf::new();
							path.push(&read.services.filesystem.upload_directory);
							path.push(&read.services.filesystem.image_sub_directory);
							path.push(name.into_inner());

							match NamedFile::open(path) {
								Ok(v) => v.into_response(&req),
								Err(_) => HttpResponse::NotFound().finish(),
							}
						}
					},
				),
			))
			.service(icon_factory.route(
				"/{name}",
				web::get().to(
					move |name: web::Path<String>, config: ConfigDataService, req: HttpRequest| {
						if name.is_empty() {
							HttpResponse::NotFound().finish()
						} else {
							let read = config.read().unwrap();

							let mut path = PathBuf::new();
							path.push(&read.services.filesystem.upload_directory);
							path.push(&read.services.filesystem.icon_sub_directory);
							path.push(name.into_inner());

							match NamedFile::open(path) {
								Ok(v) => v.into_response(&req),
								Err(_) => HttpResponse::NotFound().finish(),
							}
						}
					},
				),
			))
		} else {
			app
		}
	}
}
