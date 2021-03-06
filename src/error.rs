use std::{num::ParseIntError, sync::PoisonError};

use thiserror::Error as ThisError;

use lettre::error::Error as LettreError;
use lettre::address::AddressError;
use lettre::transport::smtp::Error as SmtpError;
use handlebars::RenderError;
use image::ImageError;
use mongodb::error::Error as MongodbError;
use reqwest::Error as HttpError;
use serde_json::Error as JsonError;
use serde::de::value::Error as SerdeValueError;
use twapi::TwapiError;
use std::io::Error as IoError;
use mongodb::bson::ser::Error as BsonError;
use url::ParseError as UrlParseError;

use actix_multipart::MultipartError;
use actix_web::Error as ActixError;
use actix_web::ResponseError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, ThisError)]
pub enum Error {
	#[error("Internal Error: {0}")]
	Internal(#[from] InternalError),

	#[error("Internal DateTime Error: {0}")]
	DateTime(#[from] DateTimeError),

	#[error("Poison Error")]
	Poisoned,

	#[error("Json Error: {0}")]
	Json(#[from] JsonError),
	#[error("Serde Value Error: {0}")]
	SerdeValue(#[from] SerdeValueError),

	#[error("IO Error: {0}")]
	Io(#[from] IoError),
	#[error("HTTP Error: {0}")]
	Http(#[from] HttpError),
	#[error("Parse Int Error: {0}")]
	ParseInt(#[from] ParseIntError),
	#[error("Url Parse Error: {0}")]
	UrlParse(#[from] UrlParseError),

	#[error("Actix Invalid Header Error: {0}")]
	ActixInvalidHeaderValue(String),
	#[error("ActixWeb Error: {0}")]
	Actix(#[from] ActixError),
	#[error("Multipart Error: {0}")]
	Multipart(#[from] MultipartError),
	#[error("MongoDB Error: {0}")]
	Mongodb(#[from] MongodbError),
	#[error("Bson Error: {0}")]
	Bson(#[from] BsonError),
	#[error("Image Error: {0}")]
	Image(#[from] ImageError),
	#[error("Handlebars Error: {0}")]
	Render(#[from] RenderError),
	#[error("Lettre Error: {0}")]
	Lettre(#[from] LettreError),
	#[error("SMTP Error: {0}")]
	Smtp(#[from] SmtpError),
	#[error("Address Error: {0}")]
	Address(#[from] AddressError),
	#[error("TwApi Error: {0}")]
	TwApi(String),

	#[error("Backblaze Error: {0}")]
	B2(#[from] crate::upload::service::b2::JsonErrorStruct),

	#[error("PNG Optimize Error: {0}")]
	Oxipng(#[from] oxipng::PngError),
}

#[derive(Debug, ThisError)]
pub enum InternalError {
	// Backblaze

	#[error("Backblaze B2 Authorization Error.")]
	B2Authorization,
	#[error("Backblaze B2 Get Upload Url Error.")]
	B2GetUploadUrl,
	#[error("Backblaze B2 Upload File Error.")]
	B2UploadFile,

	// Actix

	#[error("The Upload Size is Too Large")]
	UploadSizeTooLarge,

	#[error("The UID is Too Large")]
	UidSizeTooLarge,

	#[error("The user does not exist")]
	UserMissing,

	#[error("The File Type is Too Large")]
	FileTypeTooLarge,

	// Image / Gallery

	#[error("Max Galleries")]
	MaxGalleries,

	#[error("Max Images in Gallery")]
	MaxImagesInGallery,

	#[error("Image Does Not Exist")]
	ImageDoesNotExist,

	#[error("Gallery Does Not Exist")]
	GalleryDoesNotExist,

	// JPEG

	#[error("An Error Occured while trying to Optimize JPEG Image")]
	MozJpegError,

	#[error("An Error Occured while trying retrieve JPEG Data")]
	MozJpegDataRetrive,

	#[error("An Error Occured while trying read scan lines for JPEG")]
	MozJpegScanLines,

	#[error("The Specific format is unimplemented for this JPEG")]
	MozJpegUnimplementedFormat,

	// Other

	#[error("Unable to convert PathBuf to String")]
	ConvertPathBufToString,

	#[error("Missing Wanted ObjectId")]
	MissingObjectId,
}

impl ResponseError for Error {}

impl<V> From<PoisonError<V>> for Error {
	fn from(_: PoisonError<V>) -> Self {
		Self::Poisoned
	}
}

impl From<TwapiError> for Error {
	fn from(v: TwapiError) -> Self {
		Self::TwApi(format!("{:?}", v))
	}
}


#[derive(Debug, ThisError)]
pub enum DateTimeError {
	#[error("Invalid Year {0}")]
	InvalidYear(u32),
	#[error("Invalid Month {0}")]
	InvalidMonth(u32),
	#[error("Invalid Day {0}")]
	InvalidDay(u32),
	#[error("Invalid Hour {0}")]
	InvalidHour(u32),
	#[error("Invalid Minute {0}")]
	InvalidMinute(u32),
	#[error("Invalid Second {0}")]
	InvalidSecond(u32),
}