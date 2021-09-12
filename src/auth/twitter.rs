use actix_identity::Identity;
use actix_web::{Scope, web};
use actix_web::{HttpResponse, http::header};
use mongodb::bson::doc;
// TODO: Remove.
use twapi::{Twapi, oauth1::request_token};

use crate::config::Config;
use crate::db::{CollectionType, get_auth_collection, get_collection, get_users_collection};
use crate::db::model::{NewUser, UserData, UserTwitter, create_auth_verify, find_and_remove_auth_verify};
use crate::upload::image::UploadImageType;
use crate::web::ConfigDataService;
use crate::words::gen_uuid;
use crate::Result;



pub fn register(scope: Scope, config: &Config) -> Scope {
	if config.passport.twitter.enabled {
		scope
			.route(&config.passport.twitter.auth_path, web::get().to(get_twitter_oauth))
			.route(&config.passport.twitter.callback_path, web::get().to(get_twitter_oauth_callback))
	} else {
		scope
	}
}




pub async fn get_twitter_oauth(identity: Identity, config: ConfigDataService) -> Result<HttpResponse> {
	if identity.identity().is_some() {
		return Ok(HttpResponse::Found().append_header((header::LOCATION, "/")).finish())
	}

	let config = config.read().unwrap();

	let (oauth_token, oauth_token_secret, url) = request_token(
		&config.passport.twitter.consumer_key,
		&config.passport.twitter.consumer_secret,
		// TODO
		&format!("{}://{}{}", &config.website.url_protocol, &config.website.http_base_host, &config.passport.twitter.callback_path),
		None
	).await.unwrap();

	create_auth_verify(oauth_token, oauth_token_secret, &get_auth_collection()).await?;

	Ok(HttpResponse::Found().append_header((header::LOCATION, url)).finish())
}




#[derive(Serialize, Deserialize)]
pub struct QueryCallback {
	pub oauth_token: String,
	pub oauth_verifier: String
}


pub async fn get_twitter_oauth_callback(query: web::Query<QueryCallback>, identity: Identity, config: ConfigDataService) -> Result<HttpResponse> {
	if identity.identity().is_some() {
		return Ok(HttpResponse::Found().append_header((header::LOCATION, "/")).finish())
	}

	let config = config.read().unwrap();

	let QueryCallback { oauth_token , oauth_verifier } = query.into_inner();

	let auth_collection = get_auth_collection();
	let user_collection = get_users_collection();

	if let Some(auth_verify) = find_and_remove_auth_verify(&oauth_token, &auth_collection).await? {
		let (oauth_token, oauth_token_secret, _user_id, _screen_name) = twapi::oauth1::access_token(
			&config.passport.twitter.consumer_key,
			&config.passport.twitter.consumer_secret,
			&auth_verify.oauth_token,
			&auth_verify.oauth_token_secret,
			&oauth_verifier
		).await.expect("access_token");

		let user = twapi::UserAuth::new(
			&config.passport.twitter.consumer_key,
			&config.passport.twitter.consumer_secret,
			&oauth_token,
			&oauth_token_secret
		);

		let resp = user.get_verify_credentials(
			&Vec::new()
		).await.expect("get_verify_credentials");

		if resp.status_code == 200 {
			let profile: VerifyCredentials = serde_json::from_value(resp.json.unwrap())?;

			// Create or Update User.

			let user = if let Some(user) = user_collection.find_one(doc!{ "twitter.id": profile.id }, None).await? {
				user
			} else {
				let new_user = NewUser {
					twitter: UserTwitter {
						id: profile.id,
						token: oauth_token,
						username: profile.screen_name,
						display_name: profile.name
					},
					data: UserData {
						upload_type: UploadImageType::PrefixAndSuffix,
						is_banned: false,
						join_date: mongodb::bson::DateTime::now(),
						unique_id: gen_uuid(),
						image_count: 0,
						deletion_count: 0,
					}
				};

				let inserted = get_collection(CollectionType::Users)
					.insert_one(mongodb::bson::to_document(&new_user).unwrap(), None)
					.await?;

				new_user.into_user(inserted.inserted_id.as_object_id().unwrap())
			};

			identity.remember(serde_json::to_string(&user.into_slim()).unwrap());
		} else {
			println!("{:#?}", resp);
		}
	}

	Ok(HttpResponse::Found().append_header((header::LOCATION, "/")).finish())
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