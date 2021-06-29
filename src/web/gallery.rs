use actix_identity::Identity;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, http::header, get, post, delete, web};

use crate::{Result, WordManager, db::{get_images_collection, get_users_collection, model}, error::InternalError, words};

use super::{ConfigDataService, HandlebarsDataService};




#[get("/gallery")]
async fn home(identity: Identity, hb: HandlebarsDataService<'_>, config: ConfigDataService) -> Result<HttpResponse> {
	let is_logged_in = identity.identity().is_some();

	if is_logged_in {
		let body = hb.render(
			"gallery/home",
			&json!({ "title": config.read()?.website.title })
		)?;

		Ok(HttpResponse::Ok().body(body))
	} else {
		let location = config.read()?.get_base_url();

		Ok(HttpResponse::TemporaryRedirect().append_header((header::LOCATION, location)).finish())
	}
}


#[get("/g/{id}")]
async fn item(identity: Identity, path: web::Path<String>, hb: HandlebarsDataService<'_>, config: ConfigDataService) -> Result<HttpResponse> {
	let is_logged_in = identity.identity().is_some();

	if is_logged_in {
		let body = if path.as_str() == "new" {
			hb.render(
				"gallery/upload",
				&json!({ "title": config.read()?.website.title })
			)?
		} else {
			hb.render(
				"gallery/item",
				&json!({ "title": config.read()?.website.title })
			)?
		};

		Ok(HttpResponse::Ok().body(body))
	} else {
		let location = config.read()?.get_base_url();

		Ok(HttpResponse::TemporaryRedirect().append_header((header::LOCATION, location)).finish())
	}
}