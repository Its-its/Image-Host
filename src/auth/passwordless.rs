// You supply an email. We email the link to authenticate with.

// TODO: Better security. Simple Proof of Concept.


use actix_identity::Identity;
use actix_web::{http::header, HttpResponse};
use actix_web::{web, Scope};

use lettre::message::header::ContentType;
use lettre::message::{MultiPart, SinglePart};
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use mongodb::bson::doc;
use mongodb::options::{Collation, CollationStrength, FindOneOptions};

use crate::config::{Config, ConfigEmail};
use crate::db::model::{NewUser, UserPasswordless, create_auth_verify, find_and_remove_auth_verify};
use crate::db::{get_auth_collection, get_collection, get_users_collection, CollectionType};
use crate::error::{InternalError, Error};
use crate::upload::image::UploadImageType;
use crate::web::{ConfigDataService, HandlebarsDataService, remember_identity};
use crate::words::{gen_sample_alphanumeric, gen_uuid};
use crate::Result;


pub fn register(scope: Scope, config: &Config) -> Scope {
	if config.auth.passwordless.enabled {
		scope
			.route(
				&config.auth.passwordless.auth_path,
				web::get().to(get_passwordless_oauth),
			)
			.route(
				&config.auth.passwordless.auth_path,
				web::post().to(post_passwordless_oauth),
			)
			.route(
				&config.auth.passwordless.callback_path,
				web::get().to(get_passwordless_oauth_callback),
			)
	} else {
		scope
	}
}


pub async fn get_passwordless_oauth(
	identity: Identity,
	hb: HandlebarsDataService<'_>,
	config: ConfigDataService,
) -> Result<HttpResponse> {
	if identity.identity().is_some() {
		return Ok(HttpResponse::Found()
			.append_header((header::LOCATION, "/"))
			.finish());
	}

	let config = config.read()?;

	let body = hb.render(
		"auth/passwordless",
		&json!({
			"title": config.website.title,
			"auth_path": config.auth.passwordless.auth_path
		}),
	)?;

	Ok(HttpResponse::Ok().body(body))
}

#[derive(Serialize, Deserialize)]
pub struct TwitterPostCallback {
	pub email: String,
}

pub async fn post_passwordless_oauth(
	query: web::Query<TwitterPostCallback>,
	hb: HandlebarsDataService<'_>,
	identity: Identity,
	config: ConfigDataService,
) -> Result<HttpResponse> {
	if identity.identity().is_some() {
		return Ok(HttpResponse::MethodNotAllowed().finish()); // TODO: What's the proper status?
	}

	let config = config.read()?;
	let oauth_token = gen_sample_alphanumeric(config.auth.passwordless.secret_key_length, &mut rand::thread_rng());

	let auth_url = format!(
		"{}://{}{}?oauth_token={}&email={}",
		&config.website.url_protocol,
		&config.website.http_base_host,
		&config.auth.passwordless.callback_path,
		serde_urlencoded::from_str::<String>(&oauth_token)?,
		serde_urlencoded::from_str::<String>(&query.email)?
	);

	let main_html = hb.render(
		"auth/email_sign_in",
		&json!({
			"website_title": config.website.title,
			"website_url_protocol": config.website.url_protocol,
			"website_http_base_host": config.website.http_base_host,
			"website_http_image_host": config.website.http_image_host,
			"website_http_icon_host": config.website.http_icon_host,

			"email_display_name": config.email.display_name,
			"email_sending_email": config.email.sending_email,
			"email_contact_email": config.email.contact_email,

			"email_callback_url": "https://image.host/auth/nopass",
		}),
	)?;

	create_auth_verify(oauth_token, String::new(), &get_auth_collection()).await?;

	send_auth_email(query.0.email, auth_url, main_html, &config.email)?;

	Ok(HttpResponse::Ok().finish())
}

#[derive(Serialize, Deserialize)]
pub struct QueryCallback {
	pub oauth_token: String,
	pub email: String,
}

pub async fn get_passwordless_oauth_callback(
	query: web::Query<QueryCallback>,
	identity: Identity,
) -> Result<HttpResponse> {
	if identity.identity().is_some() {
		return Ok(HttpResponse::Found()
			.append_header((header::LOCATION, "/"))
			.finish());
	}

	let QueryCallback {
		oauth_token,
		email,
	} = query.into_inner();

	let auth_collection = get_auth_collection();
	let user_collection = get_users_collection();

	if let Some(_auth_verify) = find_and_remove_auth_verify(&oauth_token, &auth_collection).await? {
		// Create or Update User.
		let user = if let Some(user) = user_collection
			.find_one(
				doc! { "passwordless.email": email.to_lowercase() },
				FindOneOptions::builder()
					.collation(
						Collation::builder()
							.locale("en")
							.strength(CollationStrength::Secondary)
							.build()
						)
					.build()
			)
			.await?
		{
			user
		} else {
			let new_user = NewUser {
				twitter: None,
				passwordless: Some(UserPasswordless {
					email
				}),
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

			new_user.into_user(
				inserted.inserted_id.as_object_id().ok_or_else(|| Error::from(InternalError::MissingObjectId))?
			)
		};

		remember_identity(&identity, user)?;
	}

	Ok(HttpResponse::Found()
		.append_header((header::LOCATION, "/"))
		.finish())
}



pub fn send_auth_email(sending_to_email: String, alt_text: String, main_html: String, email_config: &ConfigEmail) -> Result<()> {
	let email = Message::builder()
		.from(format!("{} <{}>", email_config.display_name, email_config.sending_email).parse()?)
		.reply_to(email_config.sending_email.parse()?)
		.to(sending_to_email.parse()?)
		.subject(&email_config.subject_line)
		.multipart(
            MultiPart::alternative() // This is composed of two parts.
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(alt_text),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(main_html),
                ),
        )?;

	let creds = Credentials::new(email_config.smtp_username.clone(), email_config.smtp_password.clone());

	// Open a remote connection to gmail
	let mailer = SmtpTransport::relay(&email_config.smtp_relay)?
		.credentials(creds)
		.build();

	// Send the email
	mailer.send(&email)?;

	Ok(())
}