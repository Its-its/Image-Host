use std::{num::ParseIntError, sync::PoisonError};

use thiserror::Error as ThisError;

use std::io::Error as IoError;
use handlebars::RenderError;
use image::ImageError;
use reqwest::Error as HttpError;
use serde_json::Error as JsonError;
use mongodb::error::Error as MongodbError;

use actix_web::Error as ActixError;
use actix_multipart::MultipartError;
use actix_web::ResponseError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, ThisError)]
pub enum Error {
	#[error("Internal Error: {0}")]
	Internal(InternalError),

	#[error("Poison Error")]
	Poisoned,

	#[error("Json Error: {0}")]
	Json(JsonError),
	#[error("IO Error: {0}")]
	Io(IoError),
	#[error("HTTP Error: {0}")]
	Http(HttpError),
	#[error("Parse Int: {0}")]
	ParseInt(ParseIntError),

	#[error("ActixWeb Error: {0}")]
	Actix(ActixError),
	#[error("Multipart Error: {0}")]
	Multipart(MultipartError),
	#[error("MongoDB Error: {0}")]
	Mongodb(MongodbError),
	#[error("Image Error: {0}")]
	Image(ImageError),
	#[error("Handlebars Error: {0}")]
	Render(RenderError),

	#[error("Backblaze Error: {0}")]
	B2(crate::upload::service::b2::JsonErrorStruct)
}


#[derive(Debug, ThisError)]
pub enum InternalError {
	#[error("Backblaze B2 Authorization Error.")]
	B2Authorization,
	#[error("Backblaze B2 Get Upload Url Error.")]
	B2GetUploadUrl,
	#[error("Backblaze B2 Upload File Error.")]
	B2UploadFile,

	#[error("The Upload Size is Too Large")]
	UploadSizeTooLarge,

	#[error("The UID is Too Large")]
	UidSizeTooLarge,

	#[error("The File Type is Too Large")]
	FileTypeTooLarge,

	#[error("Max Galleries")]
	MaxGalleries,

	#[error("Max Images in Gallery")]
	MaxImagesInGallery,

	#[error("Image Does Not Exist")]
	ImageDoesNotExist,

	#[error("Gallery Does Not Exist")]
	GalleryDoesNotExist
}


impl ResponseError for Error {}


impl From<Error> for actix_web::body::Body {
	fn from(val: Error) -> Self {
		actix_web::body::Body::from_message(format!("{}", val))
	}
}

impl From<InternalError> for Error {
	fn from(value: InternalError) -> Self {
		Self::Internal(value)
	}
}


impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self::ParseInt(value)
    }
}

impl From<IoError> for Error {
	fn from(value: IoError) -> Self {
		Self::Io(value)
	}
}

impl From<HttpError> for Error {
	fn from(value: HttpError) -> Self {
		Self::Http(value)
	}
}

impl From<JsonError> for Error {
	fn from(value: JsonError) -> Self {
		Self::Json(value)
	}
}

impl From<ActixError> for Error {
	fn from(value: ActixError) -> Self {
		Self::Actix(value)
	}
}

impl From<MultipartError> for Error {
	fn from(value: MultipartError) -> Self {
		Self::Multipart(value)
	}
}

impl From<MongodbError> for Error {
	fn from(value: MongodbError) -> Self {
		Self::Mongodb(value)
	}
}

impl From<ImageError> for Error {
	fn from(value: ImageError) -> Self {
		Self::Image(value)
	}
}

impl From<RenderError> for Error {
	fn from(value: RenderError) -> Self {
		Self::Render(value)
	}
}

impl From<crate::upload::service::b2::JsonErrorStruct> for Error {
    fn from(value: crate::upload::service::b2::JsonErrorStruct) -> Self {
        Self::B2(value)
    }
}

impl<V> From<PoisonError<V>> for Error {
	fn from(_: PoisonError<V>) -> Self {
		Self::Poisoned
	}
}