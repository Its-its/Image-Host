use std::sync::RwLock;

use actix_identity::Identity;
use actix_web::{HttpResponse, http::header, web};
use mongodb::bson::oid::ObjectId;

use crate::{Result, config::Config, db::{UsersCollection, model::SlimUser}};


// TODO



pub async fn get_twitter(identity: Identity) -> Result<HttpResponse> {
	Ok(HttpResponse::TemporaryRedirect().append_header((header::LOCATION, "/")).finish())
}

pub async fn post_twitter() -> Result<HttpResponse> {
	Ok(HttpResponse::Ok().finish())
}