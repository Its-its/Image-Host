use std::{
	ops::{Deref, DerefMut},
	path::PathBuf,
};

use serde::{de::DeserializeOwned, Serialize};

use crate::Result;

pub type Config = ConfigHelper<ConfigInner>;

#[derive(Default)]
pub struct ConfigHelper<C: DeserializeOwned + Serialize + Default> {
	path: PathBuf,
	config: C,
}

impl<C: DeserializeOwned + Serialize + Default> ConfigHelper<C> {
	pub fn create_with_defaults<P: Into<PathBuf>>(path: P) -> Self {
		Self {
			path: path.into(),
			config: C::default(),
		}
	}

	pub async fn create_and_load<P: Into<PathBuf>>(path: P) -> Result<Self> {
		let mut this = Self::create_with_defaults(path);
		this.load().await?;

		Ok(this)
	}

	pub async fn load(&mut self) -> Result<()> {
		// File doesn't exist?
		if tokio::fs::metadata(&self.path).await.is_err() {
			tokio::fs::write(&self.path, serde_json::to_string_pretty(&self.config)?).await?;
			panic!("Config file was missing. I created it.\nEnsure the config is correct. Located in app/config.json");
		}

		// Error occured while reading the file?
		let file = match tokio::fs::File::open(&self.path).await {
			Ok(v) => v,
			Err(e) => {
				panic!("{}\n\nAn Error Occured while trying to open the config file!\nPlease ensure you have permissions to access it.", e);
			}
		};

		self.config = serde_json::from_reader(file.into_std().await)?;

		Ok(())
	}

	pub async fn save(&self) -> Result<()> {
		tokio::fs::write(&self.path, serde_json::to_string_pretty(&self.config)?).await?;

		Ok(())
	}
}

impl<C: DeserializeOwned + Serialize + Default> AsRef<C> for ConfigHelper<C> {
	fn as_ref(&self) -> &C {
		&self.config
	}
}

impl<C: DeserializeOwned + Serialize + Default> AsMut<C> for ConfigHelper<C> {
	fn as_mut(&mut self) -> &mut C {
		&mut self.config
	}
}

impl<C: DeserializeOwned + Serialize + Default> Deref for ConfigHelper<C> {
	type Target = C;

	fn deref(&self) -> &Self::Target {
		&self.config
	}
}

impl<C: DeserializeOwned + Serialize + Default> DerefMut for ConfigHelper<C> {
	fn deref_mut(&mut self) -> &mut C {
		&mut self.config
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigInner {
	pub debug: bool,

	pub session_secret: String,

	pub email: ConfigEmail,
	pub database: ConfigDatabase,

	pub website: ConfigWebsite,
	pub auth: ConfigAuth,

	pub services: ConfigServices,
	pub features: ConfigFeatures,
}

impl Default for ConfigInner {
	fn default() -> Self {
		Self {
			debug: false,

			session_secret: "secret key goes here".into(),

			email: ConfigEmail::default(),
			database: ConfigDatabase::default(),

			website: ConfigWebsite::default(),
			auth: ConfigAuth::default(),
			services: ConfigServices::default(),
			features: ConfigFeatures::default(),
		}
	}
}



// Email

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ConfigEmail {
	pub display_name: String,
	pub sending_email: String,
	pub contact_email: String,

	pub subject_line: String,

	pub smtp_username: String,
	pub smtp_password: String,
	pub smtp_relay: String,
}

// Database

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigDatabase {
	pub url: String,
	pub database: String,
}

impl Default for ConfigDatabase {
	fn default() -> Self {
		Self {
			url: "mongodb://127.0.0.1:27017".into(),
			database: "image_host".into(),
		}
	}
}



// Website

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigWebsite {
	pub title: String,
	pub port: usize,
	pub url_protocol: String,
	pub base_url: String,
	pub http_base_host: String,
	pub http_image_host: String,
	pub http_icon_host: String,
}

impl ConfigWebsite {
	pub fn base_host_with_proto(&self) -> String {
		format!("{}://{}", self.url_protocol, self.http_base_host)
	}

	pub fn image_host_with_proto(&self) -> String {
		format!("{}://{}", self.url_protocol, self.http_image_host)
	}

	pub fn icon_host_with_proto(&self) -> String {
		format!("{}://{}", self.url_protocol, self.http_icon_host)
	}
}

impl Default for ConfigWebsite {
	fn default() -> Self {
		Self {
			title: "Image Host".into(),

			port: 8080,

			url_protocol: "https".into(),
			base_url: "127.0.0.1".into(),

			http_base_host: "local.host".into(),
			http_image_host: "i.local.host".into(),
			http_icon_host: "icon.local.host".into(),
		}
	}
}



// Authentication

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ConfigAuth {
	pub google: ConfigAuthGoogle,
	pub twitter: ConfigAuthTwitter,
	pub passwordless: ConfigAuthPasswordless,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigAuthGoogle {
	#[serde(default)]
	pub enabled: bool,

	pub client_id: String,
	pub client_secret: String,

	pub auth_path: String,
	pub callback_path: String,
}

impl Default for ConfigAuthGoogle {
	fn default() -> Self {
		Self {
			enabled: Default::default(),
			auth_path: String::from("/auth/google"),
			callback_path: String::from("/auth/google/callback"),
			client_id: String::new(),
			client_secret: String::new(),
		}
	}
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigAuthTwitter {
	#[serde(default)]
	pub enabled: bool,

	pub consumer_key: String,
	pub consumer_secret: String,

	pub auth_path: String,
	pub callback_path: String,
}

impl Default for ConfigAuthTwitter {
	fn default() -> Self {
		Self {
			enabled: Default::default(),
			auth_path: String::from("/auth/twitter"),
			callback_path: String::from("/auth/twitter/callback"),
			consumer_key: String::new(),
			consumer_secret: String::new(),
		}
	}
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigAuthPasswordless {
	#[serde(default)]
	pub enabled: bool,

	pub secret_key_length: usize,

	pub auth_path: String,
	pub callback_path: String,
}

impl Default for ConfigAuthPasswordless {
	fn default() -> Self {
		Self {
			secret_key_length: 30,
			enabled: Default::default(),
			auth_path: String::from("/auth/nopass"),
			callback_path: String::from("/auth/nopass/callback")
		}
	}
}



// Features

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ConfigFeatures {
	pub compression: ConfigFeatureCompression,
	pub gallery: ConfigFeatureGallery,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigFeatureCompression {
	pub enabled: bool,
	pub quality: f32,
}

impl Default for ConfigFeatureCompression {
    fn default() -> Self {
        Self {
			enabled: false,
			quality: 80.0
		}
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ConfigFeatureGallery {
	pub enabled: bool
}



// Services

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ConfigServices {
	pub logging: ConfigServiceLogging,
	pub b2: ConfigServiceB2,
	pub filesystem: ConfigServiceFileSystem,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ConfigServiceLogging {
	pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ConfigServiceB2 {
	pub enabled: bool,

	pub id: String,
	pub key: String,

	pub bucket_id: String,

	pub image_sub_directory: String,
	pub icon_sub_directory: String,

	pub public_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ConfigServiceFileSystem {
	pub enabled: bool,

	pub upload_directory: String,

	pub image_sub_directory: String,
	pub icon_sub_directory: String,
}
