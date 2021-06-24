use std::sync::RwLock;
use std::{convert::TryInto, sync::Mutex};

use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use futures::TryStreamExt;

use handlebars::Handlebars;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, http::header, middleware::Logger, get, post, delete, web::{self, JsonConfig}};
use actix_multipart::{Field, Multipart};

use mongodb::bson::doc;

use crate::Filename;
use crate::config::Config;
use crate::upload::service::Service;
use crate::{Result, WordManager, db::{get_images_collection, model}, error::InternalError, words};


mod profile;


// Services
type UploadDataService = web::Data<Mutex<Service>>;
type ConfigDataService = web::Data<RwLock<Config>>;
type WordDataService = web::Data<Mutex<WordManager>>;
type HandlebarsDataService<'a> = web::Data<Handlebars<'a>>;


pub fn get_slim_user_identity(identity: Identity) -> Option<model::SlimUser> {
	let id = identity.identity()?;
	serde_json::from_str(&id).ok()
}


#[get("/")]
async fn index(identity: Identity, hb: HandlebarsDataService<'_>, config: ConfigDataService) -> Result<HttpResponse> {
	let is_logged_in = identity.identity().is_some();

	let body = hb.render(
		"home",
		&json!({
			"title": config.read()?.website.title,
			"is_logged_in": is_logged_in
		})
	)?;

	Ok(HttpResponse::Ok().body(body))
}


#[get("/logout")]
async fn logout(identity: Identity, _hb: HandlebarsDataService<'_>) -> HttpResponse {
	identity.forget();
	HttpResponse::Ok().finish()
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

	let image = collection.find_one(
		doc! {
			"name": path.as_ref()
		},
		None
	).await?;

	// TODO: Check if user is the one who uploaded it.

	Ok(HttpResponse::Found().json(image))
}

#[derive(Serialize, Deserialize)]
struct UpdateImage {
	favorite: Option<bool>,
	custom_name: Option<String>,
	tags: Option<Vec<String>>
}

#[post("/image/{name}")]
async fn update_image(identity: Identity, path: web::Path<String>, form: web::Form<UpdateImage>) -> Result<HttpResponse> {
	let collection = get_images_collection();

	let user = match get_slim_user_identity(identity) {
		Some(u) => u,
		None => {
			return Ok(HttpResponse::Unauthorized().body("Not Logged in."));
		}
	};

	let res = collection.update_one(
		doc! {
			"name": path.as_ref(),
			"uploader_id": user.id
		},
		doc! {
			"$set": {
				// TODO
			}
		},
		None
	).await?;

	// TODO: Check if user is the one who uploaded it.

	Ok(HttpResponse::Found().json(res))
}


#[delete("/image/{name}")]
async fn remove_image(identity: Identity, file_name: web::Path<String>, service: UploadDataService) -> Result<HttpResponse> {
	let collection = get_images_collection();

	// let user = match get_slim_user_identity(identity) {
	// 	Some(u) => u,
	// 	None => {
	// 		return Ok(HttpResponse::Unauthorized().body("Not Logged in."));
	// 	}
	// };

	let res = collection.find_one(
		doc! {
			"name": file_name.as_ref()
		},
		None
	).await?;

	if let Some(image) = res {
		// if image.uploader_id != user.id { Ok(HttpResponse::Unauthorized().finish()) }

		let file_name = Filename::from(image.full_file_name());

		service.lock()?.hide_file(file_name).await?;

		let res = image.delete_request(&collection).await?;

		// TODO: Check if user is the one who uploaded it.

		if res.modified_count == 0 {
			Ok(HttpResponse::Unauthorized().finish())
		} else {
			Ok(HttpResponse::Ok().finish())
		}
	} else {
		Ok(HttpResponse::NotFound().finish())
	}
}






#[post("/upload")]
async fn upload(req: HttpRequest, mut multipart: Multipart, service: UploadDataService, words: WordDataService, config: ConfigDataService) -> Result<HttpResponse> {
	let ip_addr: String = req.connection_info().remote_addr().map_or(String::new(), |c| c.to_string());

	// TODO: Properly stream.
	// Make a class to ensure both fields (image, uid) are there and proper.

	let mut image_content_type = None;
	let mut image_data = None;
	let mut uid = None;

	while let Some(field) = multipart.try_next().await? {
		let disp = field.content_disposition().unwrap();

		if disp.is_form_data() {
			match disp.get_name() {
				Some("image") => {
					image_content_type.insert(field.content_type().to_string());
					image_data.insert(get_file(field).await?);
				}

				Some("uid") => {
					uid.insert(get_uid(field).await?);
				}

				_ => ()
			}
		}
	}

	// Process File

	let uid = match uid {
		Some(v) => v,
		None => {
			let base_url = config.read()?.get_base_url();

			return Ok(
				HttpResponse::NotAcceptable()
					.append_header((header::LOCATION, base_url + "error?type=Missing+Unique+ID"))
					.body("Missing Unique ID")
				);
		}
	};

	let image_content_type = match image_content_type {
		Some(v) => v,
		None => {
			let base_url = config.read()?.get_base_url();

			return Ok(
				HttpResponse::NotAcceptable()
					.append_header((header::LOCATION, base_url + "error?type=Missing+Unique+image+content+type"))
					.body("Missing Image Content-Type")
			);
		}
	};

	let file_data = match image_data {
		Some(v) => v,
		None => {
			let base_url = config.read()?.get_base_url();

			return Ok(
				HttpResponse::NotAcceptable()
					.append_header((header::LOCATION, base_url + "error?type=Missing+Image+Data"))
					.body("Missing Image Data")
			);
		}
	};


	service.lock()?.process_files(uid, file_data, image_content_type, ip_addr, &mut *words.lock()?).await?;

	Ok(HttpResponse::Ok().finish())
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
				return Err(InternalError::UploadSizeTooLarge.into());
			}
		}

		value
	};

	Ok(String::from_utf8_lossy(&bytes).to_string())
}



pub async fn init(config: Config, service: Service) -> Result<()> {
	println!(
		"Prefixes: {}\nSuffixes: {}\nCombinations: {}",
		words::PREFIXES.len(),
		words::SUFFIXES.len(),
		words::PREFIXES.len() * words::SUFFIXES.len()
	);

	// Handlebars
	let mut handlebars = Handlebars::new();
	handlebars.register_templates_directory(".hbs", "./app/frontend/views").unwrap();
	let handlebars_ref = web::Data::new(handlebars);

	let service = web::Data::new(Mutex::new(service));
	let config = web::Data::new(RwLock::new(config));

	println!("Starting website.");

	HttpServer::new(move || {
		App::new()
			// enable logger
			.wrap(Logger::default())

			// cookie session middleware
			.wrap(IdentityService::new(
				CookieIdentityPolicy::new("super secret key of my life why.".as_bytes())
					.name("auth")
					.max_age(chrono::Duration::days(365).to_std().unwrap().try_into().unwrap())
					.secure(false)
			))

			.data(Mutex::new(WordManager::new()))
			.data(JsonConfig::default().limit(4096))

			.app_data(service.clone())
			.app_data(config.clone())
			.app_data(handlebars_ref.clone())

			.service(upload)

			.service(index)
			.service(logout)

			.service(profile::profile)
			.service(profile::update_settings)
			.service(profile::get_images)
			.service(profile::get_settings)

			.service(get_image_info)
			.service(update_image)
			.service(remove_image)

			.service(
				web::resource("/auth/twitter")
					.route(web::get().to(crate::auth::twitter::get_twitter))
					.route(web::post().to(crate::auth::twitter::post_twitter))
			)
			.service(actix_files::Files::new("/", "./app/frontend/public/www"))
	})
	.bind("127.0.0.1:8080")?
	.run()
	.await?;

	Ok(())
}