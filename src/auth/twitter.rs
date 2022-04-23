use actix_identity::Identity;
use actix_web::{http::header, HttpResponse};
use actix_web::{web, Scope};
use mongodb::bson::doc;
// TODO: Remove.
use twapi::{oauth1::request_token, Twapi};

use crate::config::Config;
use crate::db::model::{create_auth_verify, find_and_remove_auth_verify, NewUser, UserTwitter};
use crate::db::{get_auth_collection, get_collection, get_users_collection, CollectionType};
use crate::error::{Error, InternalError};
use crate::upload::image::UploadImageType;
use crate::web::{ConfigDataService, remember_identity};
use crate::words::gen_uuid;
use crate::Result;

pub fn register(scope: Scope, config: &Config) -> Scope {
	if config.auth.twitter.enabled {
		scope
			.route(
				&config.auth.twitter.auth_path,
				web::get().to(get_twitter_oauth),
			)
			.route(
				&config.auth.twitter.callback_path,
				web::get().to(get_twitter_oauth_callback),
			)
	} else {
		scope
	}
}

pub async fn get_twitter_oauth(
	identity: Identity,
	config: ConfigDataService,
) -> Result<HttpResponse> {
	if identity.identity().is_some() {
		return Ok(HttpResponse::Found()
			.append_header((header::LOCATION, "/"))
			.finish());
	}

	let config = config.read()?;

	let (oauth_token, oauth_token_secret, url) = request_token(
		&config.auth.twitter.consumer_key,
		&config.auth.twitter.consumer_secret,
		&format!(
			"{}://{}{}",
			&config.website.url_protocol,
			&config.website.http_base_host,
			&config.auth.twitter.callback_path
		),
		None,
	)
	.await?;

	create_auth_verify(oauth_token, oauth_token_secret, &get_auth_collection()).await?;

	Ok(HttpResponse::Found()
		.append_header((header::LOCATION, url))
		.finish())
}

#[derive(Serialize, Deserialize)]
pub struct QueryCallback {
	pub oauth_token: String,
	pub oauth_verifier: String,
}

pub async fn get_twitter_oauth_callback(
	query: web::Query<QueryCallback>,
	identity: Identity,
	config: ConfigDataService,
) -> Result<HttpResponse> {
	if identity.identity().is_some() {
		return Ok(HttpResponse::Found()
			.append_header((header::LOCATION, "/"))
			.finish());
	}

	let config = config.read()?;

	let QueryCallback {
		oauth_token,
		oauth_verifier,
	} = query.into_inner();

	let auth_collection = get_auth_collection();
	let user_collection = get_users_collection();

	if let Some(auth_verify) = find_and_remove_auth_verify(&oauth_token, &auth_collection).await? {
		let (oauth_token, oauth_token_secret, _user_id, _screen_name) =
			twapi::oauth1::access_token(
				&config.auth.twitter.consumer_key,
				&config.auth.twitter.consumer_secret,
				&auth_verify.oauth_token,
				&auth_verify.oauth_token_secret,
				&oauth_verifier,
			)
			.await?;

		let user = twapi::UserAuth::new(
			&config.auth.twitter.consumer_key,
			&config.auth.twitter.consumer_secret,
			&oauth_token,
			&oauth_token_secret,
		);

		let resp = user
			.get_verify_credentials(&Vec::new())
			.await?;

		if resp.status_code == 200 {
			let profile = match resp.json {
				Some(v) => serde_json::from_value::<VerifyCredentials>(v)?,
				None => {
					return Ok(HttpResponse::InternalServerError().body("Unable to verify credentials. Try again in a few minutes."));
				}
			};

			// Create or Update User.

			let user = if let Some(user) = user_collection
				.find_one(doc! { "twitter.id": profile.id }, None)
				.await?
			{
				user
			} else {
				let new_user = NewUser {
					twitter: Some(UserTwitter {
						id: profile.id,
						username: profile.screen_name,
						display_name: profile.name,
					}),
					passwordless: None,
					upload_type: UploadImageType::PrefixAndSuffix,
					is_banned: false,
					join_date: mongodb::bson::DateTime::now(),
					unique_id: gen_uuid(),
					image_count: 0,
					deletion_count: 0,
				};

				let inserted = get_collection(CollectionType::Users)
					.insert_one(mongodb::bson::to_document(&new_user)?, None)
					.await?;

				new_user.into_user(inserted.inserted_id.as_object_id().ok_or_else(|| Error::from(InternalError::MissingObjectId))?)
			};

			remember_identity(&identity, user)?;
		} else {
			println!("{:#?}", resp);
		}
	}

	Ok(HttpResponse::Found()
		.append_header((header::LOCATION, "/"))
		.finish())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyCredentials {
	pub id: i64,

	pub name: String,
	pub screen_name: String,

	pub location: Option<String>,
	pub description: Option<String>,

	pub url: String,

	pub protected: bool,
	pub followers_count: i64,
	pub friends_count: i64,
	pub listed_count: i64,
	pub created_at: String,
	pub favourites_count: i64,
	pub geo_enabled: bool,
	pub verified: bool,
	pub statuses_count: i64,
	// contributors_enabled: bool,
	// is_translator: bool,
	// is_translation_enabled: bool,
	// profile_background_color: String,
	// profile_background_image_url: String,
	// profile_background_image_url_https: String,
	// profile_background_tile: bool,
	// profile_image_url: String,
	// profile_image_url_https: String,
	// profile_banner_url: String,
	// profile_link_color: String,
	// profile_sidebar_border_color: String,
	// profile_sidebar_fill_color: String,
	// profile_text_color: String,
	// profile_use_background_image: bool,
	// has_extended_profile: bool,
	// default_profile: bool,
	// default_profile_image: bool,
	// following: bool,
	// follow_request_sent: bool,
	// notifications: bool,
	// translator_type: String,
	// withheld_in_countries: Vec<>,
	// suspended: bool,
	// needs_phone_verification: bool
}
