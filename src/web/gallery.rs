use actix_identity::Identity;

use actix_web::{HttpResponse, delete, get, http::header, post, put, web};

use crate::{Result, db::{get_gallery_collection, get_images_collection, model}, error::InternalError};

use super::{ConfigDataService, HandlebarsDataService, WordDataService, get_slim_user_identity};




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

		Ok(HttpResponse::Unauthorized().append_header((header::LOCATION, location)).finish())
	}
}


#[get("/g/{id}")]
async fn item(identity: Identity, _path: web::Path<String>, hb: HandlebarsDataService<'_>, config: ConfigDataService) -> Result<HttpResponse> {
	let is_logged_in = identity.identity().is_some();

	if is_logged_in {
		Ok(HttpResponse::Ok().body(
			hb.render(
				"gallery/item",
				&json!({ "title": config.read()?.website.title })
			)?
		))
	} else {
		let location = config.read()?.get_base_url();

		Ok(HttpResponse::Unauthorized().append_header((header::LOCATION, location)).finish())
	}
}


#[post("/g/new")]
async fn gallery_new(identity: Identity, config: ConfigDataService, words: WordDataService) -> Result<HttpResponse> {
	if let Some(user) = get_slim_user_identity(identity) {
		let collection = get_gallery_collection();

		let gallery_count = model::gallery_count(&user.id, &collection).await?;

		if gallery_count < 100 {
			let mut lock = words.lock()?;

			let gallery_name = model::create_empty_gallery(user.id, &mut lock.rng, &collection).await?;

			Ok(HttpResponse::Ok().body(gallery_name))
		} else {
			Err(InternalError::MaxGalleries.into())
		}
	} else {
		let location = config.read()?.get_base_url();

		Ok(HttpResponse::Unauthorized().append_header((header::LOCATION, location)).finish())
	}
}


#[post("/g/{id}")]
async fn gallery_arrange(gallery_id: web::Path<String>, arrange: web::Form<Vec<i64>>, identity: Identity, config: ConfigDataService) -> Result<HttpResponse> {
	if let Some(user) = get_slim_user_identity(identity) {
		let gallery_collection = get_gallery_collection();

		let mut gallery = match model::find_gallery_by_name(&gallery_id, &gallery_collection).await? {
			Some(v) => v,
			None => return Err(InternalError::GalleryDoesNotExist.into())
		};

		if &user.id != gallery.id.as_ref().unwrap() {
			return Ok(HttpResponse::Unauthorized().finish());
		}

		let mut images = std::mem::take(&mut gallery.images);

		for image_index in arrange.into_inner() {
			if let Some(index) = images.iter().position(|v| v.index == image_index) {
				gallery.images.push(images.remove(index));
			}
		}

		// Place remaining images into Gallery.
		gallery.images.append(&mut images);

		Ok(HttpResponse::Ok().finish())
	} else {
		let location = config.read()?.get_base_url();

		Ok(HttpResponse::Unauthorized().append_header((header::LOCATION, location)).finish())
	}
}


#[delete("/g/{id}")]
async fn gallery_delete(gallery_id: web::Path<String>, identity: Identity, config: ConfigDataService) -> Result<HttpResponse> {
	if let Some(user) = get_slim_user_identity(identity) {
		let gallery_collection = get_gallery_collection();

		let gallery = match model::find_gallery_by_name(&gallery_id, &gallery_collection).await? {
			Some(v) => v,
			None => return Err(InternalError::GalleryDoesNotExist.into())
		};

		if &user.id != gallery.id.as_ref().unwrap() {
			return Ok(HttpResponse::Unauthorized().finish());
		}

		gallery.delete(&gallery_collection).await?;

		Ok(HttpResponse::Ok().finish())
	} else {
		let location = config.read()?.get_base_url();

		Ok(HttpResponse::Unauthorized().append_header((header::LOCATION, location)).finish())
	}
}


#[put("/g/{id}/{imageName}")]
async fn gallery_add_image(path: web::Path<(String, String)>, identity: Identity, config: ConfigDataService) -> Result<HttpResponse> {
	if let Some(user) = get_slim_user_identity(identity) {
		let (gallery_id, image_name) = path.into_inner();

		let (gallery_collection, images_collection) = (get_gallery_collection(), get_images_collection());


		let mut gallery = match model::find_gallery_by_name(&gallery_id, &gallery_collection).await? {
			Some(v) => v,
			None => return Err(InternalError::GalleryDoesNotExist.into())
		};

		if &user.id != gallery.id.as_ref().unwrap() {
			return Ok(HttpResponse::Unauthorized().finish());
		}


		let image = match model::find_image_by_name(&image_name, &images_collection).await? {
			Some(v) => v,
			None => return Err(InternalError::ImageDoesNotExist.into())
		};

		// TODO: Error
		if !gallery.images.iter().any(|v| &v.id == image.id.as_ref().unwrap()) {
			gallery.add_image(image);
		}

		gallery.update(&gallery_collection).await?;

		Ok(HttpResponse::Ok().finish())
	} else {
		let location = config.read()?.get_base_url();

		Ok(HttpResponse::Unauthorized().append_header((header::LOCATION, location)).finish())
	}
}


#[delete("/g/{id}/{imageName}")]
async fn gallery_remove_image(path: web::Path<(String, String)>, identity: Identity, config: ConfigDataService) -> Result<HttpResponse> {
	if let Some(user) = get_slim_user_identity(identity) {
		let (gallery_id, image_name) = path.into_inner();

		let (gallery_collection, images_collection) = (get_gallery_collection(), get_images_collection());


		let mut gallery = match model::find_gallery_by_name(&gallery_id, &gallery_collection).await? {
			Some(v) => v,
			None => return Err(InternalError::GalleryDoesNotExist.into())
		};

		if &user.id != gallery.id.as_ref().unwrap() {
			return Ok(HttpResponse::Unauthorized().finish());
		}


		let image = match model::find_image_by_name(&image_name, &images_collection).await? {
			Some(v) => v,
			None => return Err(InternalError::ImageDoesNotExist.into())
		};


		if let Some(index) = gallery.images.iter().position(|v| &v.id == image.id.as_ref().unwrap()) {
			gallery.images.remove(index);
		}

		gallery.update(&gallery_collection).await?;

		Ok(HttpResponse::Ok().finish())
	} else {
		let location = config.read()?.get_base_url();

		Ok(HttpResponse::Unauthorized().append_header((header::LOCATION, location)).finish())
	}
}