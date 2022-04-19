use actix_identity::Identity;
use actix_web::{get, http::header, post, web, HttpResponse};
use futures::TryStreamExt;
use mongodb::bson::doc;

use crate::{
	db::{get_images_collection, get_users_collection, model},
	upload::image::UploadImageType,
	web::get_slim_user_identity,
	Result,
};

use super::{ConfigDataService, HandlebarsDataService};

#[get("/profile")]
async fn profile(
	identity: Identity,
	hb: HandlebarsDataService<'_>,
	config: ConfigDataService,
) -> Result<HttpResponse> {
	let is_logged_in = identity.identity().is_some();

	if is_logged_in {
		let body = hb.render(
			"profile",
			&json!({
				"title": config.read()?.website.title
			}),
		)?;

		Ok(HttpResponse::Ok().body(body))
	} else {
		let location = config.read()?.website.http_base_host.clone();

		Ok(HttpResponse::TemporaryRedirect()
			.append_header((header::LOCATION, location))
			.finish())
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
	upload_type: Option<u8>,
	unique_id: Option<String>,
	join_date: Option<i64>,

	icon_host: String,
	image_host: String,
}

#[post("/user/settings")]
async fn update_settings(identity: Identity, data: web::Form<Settings>) -> Result<HttpResponse> {
	let user = match get_slim_user_identity(identity) {
		Some(u) => u,
		None => {
			return Ok(HttpResponse::Unauthorized().body("Not Logged in."));
		}
	};

	if let Some(upload_id) = data.upload_type.and_then(UploadImageType::from_num) {
		get_users_collection()
			.update_one(
				doc! {
					"data.unique_id": user.unique_id
				},
				doc! {
					"$set": {
						"data.upload_type": upload_id as i32
					}
				},
				None,
			)
			.await?;
	}

	Ok(HttpResponse::Ok().json("{}".to_string()))
}

#[get("/user/settings")]
async fn get_settings(identity: Identity, _hb: HandlebarsDataService<'_>, config: ConfigDataService) -> Result<HttpResponse> {
	let slim_user = match get_slim_user_identity(identity) {
		Some(v) => v,
		None => {
			return Ok(HttpResponse::Unauthorized().body("Not Logged in."));
		}
	};

	let user = match model::find_user_by_id(slim_user.id, &get_users_collection()).await? {
		Some(u) => u,
		None => {
			return Ok(HttpResponse::InternalServerError().body("Unable to find User"));
		}
	};

	let config = config.read()?;

	Ok(HttpResponse::Ok().json(Settings {
		upload_type: Some(user.upload_type.to_num()),
		unique_id: Some(user.unique_id),
		join_date: Some(user.join_date.timestamp_millis()),

		icon_host: config.website.http_icon_host.clone(),
		image_host: config.website.http_image_host.clone(),
	}))
}

#[derive(Serialize, Deserialize)]
struct ImageQuery {
	year: u32,
	month: u32,
}

#[get("/user/images")]
async fn get_images(identity: Identity, query: web::Query<ImageQuery>) -> Result<HttpResponse> {
	let collection = get_images_collection();

	let slim_user = match get_slim_user_identity(identity) {
		Some(u) => u,
		None => {
			return Ok(HttpResponse::Unauthorized().body("Not Logged in."));
		}
	};

	let user = slim_user.upgrade().await?.unwrap();

	let mut images =
		model::find_images_by_date(user.unique_id, query.year, query.month, &collection)
			.await?;

	let images = {
		let mut values = Vec::new();

		while let Some(image) = images.try_next().await? {
			values.push(image);
		}

		values
	};

	Ok(HttpResponse::Ok().json(serde_json::json!({
		"response": {
			"year": query.year,
			"month": query.month,
			"images": images
		}
	})))
}
