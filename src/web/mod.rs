use std::sync::RwLock;
use std::{convert::TryInto, sync::Mutex};

use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_web::guard;
use actix_web::web::Data;
use futures::TryStreamExt;

use handlebars::Handlebars;

use actix_multipart::{Field, Multipart};
use actix_web::{
	delete, get,
	http::header,
	middleware::Logger,
	post,
	web::{self, JsonConfig},
	App, HttpRequest, HttpResponse, HttpServer,
};

use mongodb::bson::{doc, Document};

use crate::config::Config;
use crate::db::get_users_collection;
use crate::db::model::{find_user_by_id, SlimUser};
use crate::upload::UploadProcessData;
use crate::upload::image::UploadImageType;
use crate::upload::service::Service;
use crate::{
	db::{get_images_collection, model},
	error::InternalError,
	words, Result, WordManager,
};

pub mod gallery;
pub mod media;
pub mod profile;

// Services
pub type UploadDataService = web::Data<Service>;
pub type ConfigDataService = web::Data<RwLock<Config>>;
pub type WordDataService = web::Data<Mutex<WordManager>>;
pub type HandlebarsDataService<'a> = web::Data<Handlebars<'a>>;

pub fn get_slim_user_identity(identity: Identity) -> Option<model::SlimUser> {
	let id = identity.identity()?;
	serde_json::from_str(&id).ok()
}

pub fn remember_identity<I: Into<SlimUser>>(identity: &Identity, value: I) -> Result<()> {
	identity.remember(serde_json::to_string(&value.into())?);
	Ok(())
}


#[get("/")]
async fn index(
	identity: Identity,
	hb: HandlebarsDataService<'_>,
	config: ConfigDataService,
) -> Result<HttpResponse> {
	let is_logged_in = identity.identity().is_some();

	let config = config.read()?;

	let body = hb.render(
		"home",
		&json!({
			"title": config.website.title,
			"is_logged_in": is_logged_in,

			"is_auth_twitter_enabled": config.auth.twitter.enabled,
			"twitter_auth_path": config.auth.twitter.auth_path,
			"is_auth_passwordless_enabled": config.auth.passwordless.enabled,
			"passwordless_auth_path": config.auth.passwordless.auth_path,
		}),
	)?;

	Ok(HttpResponse::Ok().body(body))
}

#[get("/logout")]
async fn logout(identity: Identity, config: ConfigDataService) -> Result<HttpResponse> {
	identity.forget();
	Ok(HttpResponse::Ok()
		.insert_header((
			header::LOCATION,
			config.read()?.website.base_host_with_proto(),
		))
		.finish())
}

#[get("/image/{name}")]
async fn get_image_info(identity: Identity, path: web::Path<String>) -> Result<HttpResponse> {
	let collection = get_images_collection();

	let user = match get_slim_user_identity(identity) {
		Some(u) => u,
		None => {
			return Ok(HttpResponse::Unauthorized().body("Not Logged in."));
		}
	};

	let image = collection
		.find_one(
			doc! {
				"uploader_id": user.id,
				"name": path.as_ref()
			},
			None,
		)
		.await?;

	Ok(HttpResponse::Found().json(image))
}

#[derive(Serialize, Deserialize)]
struct UpdateImage {
	favorite: Option<bool>,
	custom_name: Option<String>,
	tags: Option<Vec<String>>,
}

#[post("/image/{name}")]
async fn update_image(
	identity: Identity,
	path: web::Path<String>,
	form: web::Form<UpdateImage>,
) -> Result<HttpResponse> {
	let collection = get_images_collection();

	let user = match get_slim_user_identity(identity) {
		Some(u) => u,
		None => {
			return Ok(HttpResponse::Unauthorized().body("Not Logged in."));
		}
	};

	let form = form.into_inner();

	let mut doc = Document::new();

	if let Some(favorite) = form.favorite {
		doc.insert("favorite", favorite);
	}
	// if let Some(custom_name) = form.custom_name { doc.insert("custom_name", custom_name); }
	// if let Some(tags) = form.tags { doc.insert("tags", tags); }

	let res = collection
		.update_one(
			doc! {
				"name": path.as_ref(),
				"uploader_id": user.id
			},
			doc! {
				"$set": doc
			},
			None,
		)
		.await?;

	Ok(HttpResponse::Found().json(res))
}

#[delete("/image/{name}")]
async fn remove_image(
	identity: Identity,
	file_name: web::Path<String>,
	service: UploadDataService,
) -> Result<HttpResponse> {
	let collection = get_images_collection();

	let user = match get_slim_user_identity(identity) {
		Some(u) => u,
		None => {
			return Ok(HttpResponse::Unauthorized().body("Not Logged in."));
		}
	};

	let res = collection
		.find_one(
			doc! {
				"uploader_id": user.id,
				"name": file_name.as_ref()
			},
			None,
		)
		.await?;

	if let Some(image) = res {
		let file_name = image.get_file_name();

		service.hide_file(file_name).await?;

		let res = image.delete_request(&collection).await?;

		if res.modified_count == 0 {
			Ok(HttpResponse::Unauthorized().body("Unable to delete image. Unmodified."))
		} else {
			Ok(HttpResponse::Ok().body("Deleted Image."))
		}
	} else {
		Ok(HttpResponse::NotFound().body("Unable to find Image uploaded by user."))
	}
}

#[post("/upload")]
async fn upload(
	req: HttpRequest,
	mut multipart: Multipart,
	service: UploadDataService,
	words: WordDataService,
	config: ConfigDataService,
	identity: Identity,
) -> Result<HttpResponse> {
	let is_gallery_upload = req
		.headers()
		.get(header::REFERER)
		.and_then(|v| v.to_str().ok())
		.map(|v| v.to_lowercase().contains("/g/")) // TODO: Add Website URL.
		.unwrap_or_default();

	let ip_addr = req
		.connection_info()
		.remote_addr()
		.map_or_else(String::new, |c| c.to_string());

	// TODO: Properly stream.
	// Make a class to ensure both fields (image, uid) are there and proper.

	let mut content_type = None;
	let mut image_data = None;
	let mut uid = None;
	let mut file_type = None;

	while let Some(field) = multipart.try_next().await? {
		let disp = field.content_disposition().unwrap();

		if disp.is_form_data() {
			match disp.get_name() {
				Some("image") => {
					content_type = Some(field.content_type().to_string());
					image_data = Some(get_file(field).await?);
				}

				Some("uid") => {
					uid = Some(get_uid(field).await?);
				}

				Some("type") => {
					file_type = get_file_type(field).await?;
				}

				_ => (),
			}
		}
	}

	// Gallery File Type
	if is_gallery_upload {
		file_type = Some(UploadImageType::Alphabetical32);
	}

	// Process File

	let content_type = match content_type {
		Some(v) => v,
		None => {
			let base_url = config.read()?.website.http_base_host.clone();

			return Ok(HttpResponse::NotAcceptable()
				.append_header((
					header::LOCATION,
					base_url + "error?type=Missing+Unique+image+content+type",
				))
				.body("Missing Image Content-Type"));
		}
	};

	let file_data = match image_data {
		Some(v) => v,
		None => {
			let base_url = config.read()?.website.http_base_host.clone();

			return Ok(HttpResponse::NotAcceptable()
				.append_header((header::LOCATION, base_url + "error?type=Missing+Image+Data"))
				.body("Missing Image Data"));
		}
	};

	let user = match uid {
		Some(user_id) => match find_user_by_id(user_id, &get_users_collection()).await? {
			Some(v) => v,
			None => {
				let base_url = config.read()?.website.http_base_host.clone();

				return Ok(HttpResponse::NotAcceptable()
					.append_header((
						header::LOCATION,
						base_url + "error?type=Incorrect+Unique+ID",
					))
					.body("Incorrect Unique ID"));
			}
		},

		None => match get_slim_user_identity(identity) {
			Some(u) => u.upgrade().await?.unwrap(),
			None => {
				let base_url = config.read()?.website.http_base_host.clone();

				return Ok(HttpResponse::NotAcceptable()
					.append_header((header::LOCATION, base_url + "error?type=Missing+Unique+ID"))
					.body("Missing Unique ID"));
			}
		},
	};

	let slim_image = service
		.process_files(
			UploadProcessData {
				user,
				file_type,
				file_data,
				content_type,
				ip_addr
			},
			&config,
			&words,
		)
		.await?;

	if is_gallery_upload {
		Ok(HttpResponse::Ok().json(slim_image))
	} else {
		let path = format!(
			"{}/{}",
			config.read()?.website.image_host_with_proto(),
			slim_image.full_file_name()
		);

		Ok(HttpResponse::Found()
			.append_header((header::LOCATION, path.clone()))
			.body(format!("302 Found. Redirecting to {}", path)))
	}
}

pub async fn get_file(mut field: Field) -> Result<Vec<u8>> {
	let mut value = Vec::new();

	while let Some(bytes) = field.try_next().await? {
		value.extend(bytes);

		// 10 MB MAX
		if value.len() > 10 * 1048576 {
			return Err(InternalError::UploadSizeTooLarge.into());
		}
	}

	Ok(value)
}

pub async fn get_uid(mut field: Field) -> Result<String> {
	let bytes = {
		let mut value = Vec::new();

		while let Some(bytes) = field.try_next().await? {
			value.extend(bytes);

			if value.len() > 100 {
				return Err(InternalError::UidSizeTooLarge.into());
			}
		}

		value
	};

	Ok(String::from_utf8_lossy(&bytes).to_string())
}

pub async fn get_file_type(mut field: Field) -> Result<Option<UploadImageType>> {
	let bytes = {
		let mut value = Vec::new();

		while let Some(bytes) = field.try_next().await? {
			value.extend(bytes);

			if value.len() > 100 {
				return Err(InternalError::FileTypeTooLarge.into());
			}
		}

		value
	};

	Ok(UploadImageType::from_num(
		String::from_utf8_lossy(&bytes).parse()?,
	))
}

pub async fn init(config: Config, service: Service) -> Result<()> {
	let addr = format!("{}:{}", config.website.base_url, config.website.port);

	println!(
		"Prefixes: {}\nSuffixes: {}\nCombinations: {}",
		words::PREFIXES.len(),
		words::SUFFIXES.len(),
		words::PREFIXES.len() * words::SUFFIXES.len()
	);

	// Handlebars
	let mut handlebars = Handlebars::new();
	handlebars.set_dev_mode(config.debug);
	handlebars
		.register_templates_directory(".hbs", "./app/frontend/views")
		.unwrap();
	let handlebars_ref = web::Data::new(handlebars);

	let service = web::Data::new(service);
	let config = web::Data::new(RwLock::new(config));

	println!("Starting website.");

	HttpServer::new(move || {
		let config = config.clone();
		let config_read = config.read().unwrap();

		let session_key = config_read.session_secret.clone();

		let base_url_with_www =
			header::HeaderValue::from_str(&format!("www.{}", config_read.website.http_base_host)).unwrap();
		let base_url_non_www = header::HeaderValue::from_str(&config_read.website.http_base_host).unwrap();
		let base_url_non_www_2 = base_url_non_www.clone(); // TODO: Remove.

		let image_url = config_read.website.http_image_host.clone();
		let icon_url = config_read.website.http_icon_host.clone();

		let app = App::new()
			// enable logger
			.wrap(Logger::default())
			// cookie session middleware
			.wrap(IdentityService::new(
				CookieIdentityPolicy::new(session_key.as_bytes())
					.name("auth")
					.max_age(
						chrono::Duration::days(365)
							.to_std()
							.unwrap()
							.try_into()
							.unwrap(),
					)
					.secure(false),
			))
			.app_data(Data::new(Mutex::new(WordManager::default())))
			.app_data(Data::new(JsonConfig::default().limit(4096)))
			.app_data(service.clone())
			.app_data(config.clone())
			.app_data(handlebars_ref.clone());

		let app = media::create_services(app, image_url, icon_url, &*config_read);

		// Redirect off www
		app.service(
			web::scope("")
				.guard(guard::fn_guard(move |req| {
					(|| -> Option<bool> {
						let host = req.headers().get(header::HOST)?;
						Some(host == base_url_with_www)
					})()
					.unwrap_or_default()
				}))
				.route(
					"",
					web::to(move || {
						HttpResponse::PermanentRedirect()
							.append_header((header::LOCATION, &base_url_non_www_2))
							.finish()
					}),
				),
		)
		// Base Virtual Host
		.service({
			let scope = web::scope("")
				.guard(guard::fn_guard(move |req| {
					(|| -> Option<bool> {
						let host = req.headers().get(header::HOST)?;
						Some(host == base_url_non_www)
					})()
					.unwrap_or_default()
				}))
				.service(upload)
				.service(index)
				.service(logout)
				.service(profile::profile)
				.service(profile::update_settings)
				.service(profile::get_images)
				.service(profile::get_settings)
				.service(get_image_info)
				.service(update_image)
				.service(remove_image);

			let scope = crate::feature::gallery::register(scope, &*config_read);
			let scope = crate::auth::twitter::register(scope, &*config_read);
			let scope = crate::auth::passwordless::register(scope, &*config_read);

			scope.service(actix_files::Files::new("/", "./app/frontend/public/www"))
		})
	})
	.bind(addr)?
	.run()
	.await?;

	Ok(())
}
