use std::borrow::Borrow;
use std::fs::File;
use std::io::{BufRead, BufReader};

use actix_web::error::ErrorNotAcceptable;
use mime::{Mime, GIF, JPEG, PNG};
use rand::distributions::Alphanumeric;
use rand::prelude::{Rng, ThreadRng};

use crate::db::ImagesCollection;
use crate::model;
use crate::Result;

pub static APP_PATH: &str = "./app";

fn correct_line(value: String) -> String {
	// TODO: Replace ALL invalid charizards.
	value.trim().replace(' ', "")
}

lazy_static! {
	pub static ref PREFIXES: Vec<String> = {
		let mut items: Vec<String> = Vec::new();

		let prefix_dir = format!("{}/prefixes", APP_PATH);

		#[allow(clippy::expect_used)]
		let files = std::fs::read_dir(prefix_dir).expect("reading prefix dir");

		for entry in files {
			let file = File::open(entry.unwrap().path()).unwrap();
			let reader = BufReader::new(file);

			items.extend(reader.lines().filter_map(|l| Some(correct_line(l.ok()?))));
		}

		items
	};

	pub static ref SUFFIXES: Vec<String> = {
		let mut items: Vec<String> = Vec::new();

		let suffix_dir = format!("{}/suffixes", APP_PATH);

		#[allow(clippy::expect_used)]
		let files = std::fs::read_dir(suffix_dir).expect("reading suffix dir");

		for entry in files {
			let file = File::open(entry.unwrap().path()).unwrap();
			let reader = BufReader::new(file);

			items.extend(reader.lines().filter_map(|l| Some(correct_line(l.ok()?))));
		}

		items
	};
}

#[derive(Debug, Clone)]
pub struct Filename {
	pub name: String,
	pub format: Option<Mime>,
}

impl Filename {
	pub fn new(name: String, format: Option<String>) -> Result<Self> {
		let this = Self { name, format: None };

		if let Some(format) = format {
			this.set_format(format)
		} else {
			Ok(this)
		}
	}

	pub fn format_name(&self) -> Result<&str> {
		match self.format.as_ref().map(|v| v.subtype()) {
			Some(GIF) => Ok("gif"),
			Some(PNG) => Ok("png"),
			Some(JPEG) => Ok("jpeg"),
			_ => Err(ErrorNotAcceptable("Invalid file format. Expected gif, png, or jpeg.").into())
		}
	}

	pub fn set_format(mut self, format: String) -> Result<Self> {
		if let Some(format) = format.parse().ok().filter(Self::is_valid_format) {
			self.format = Some(format);
			Ok(self)
		} else {
			Err(ErrorNotAcceptable("Invalid file format. Expected gif, png, or jpeg.").into())
		}
	}

	pub fn as_filename(&self) -> Result<String> {
		Ok(format!("{}.{}", self.name, self.format_name()?))
	}

	pub fn is_format<B: Borrow<Mime>>(&self, mime: B) -> bool {
		if let Some(format) = self.format.as_ref() {
			format == mime.borrow()
		} else {
			false
		}
	}

	fn is_valid_format(value: &Mime) -> bool {
		matches!(value.subtype(), GIF | PNG | JPEG)
	}
}


#[derive(Debug, Default, Clone)]
pub struct WordManager {
	pub rng: ThreadRng,
}

impl WordManager {
	pub async fn get_next_filename_prefix_suffix(
		&mut self,
		image_icon_same_dir: bool,
		collection: &ImagesCollection,
	) -> Result<Filename> {
		self.loop_and_check_model_db(
			get_next_filename_unchecked,
			image_icon_same_dir,
			collection
		).await
	}

	pub async fn get_next_filename_sized_8(
		&mut self,
		image_icon_same_dir: bool,
		collection: &ImagesCollection,
	) -> Result<Filename> {
		self.loop_and_check_model_db(
			|rng| Filename::new(gen_sample_alphanumeric(8, rng), None),
			image_icon_same_dir,
			collection,
		).await
	}

	pub async fn get_next_filename_sized_32(
		&mut self,
		image_icon_same_dir: bool,
		collection: &ImagesCollection,
	) -> Result<Filename> {
		self.loop_and_check_model_db(
			|rng| Filename::new(gen_sample_alphanumeric(32, rng), None),
			image_icon_same_dir,
			collection,
		).await
	}

	async fn loop_and_check_model_db(
		&mut self,
		func: impl Fn(&mut ThreadRng) -> Result<Filename>,
		image_icon_same_dir: bool,
		collection: &ImagesCollection,
	) -> Result<Filename> {
		loop {
			let mut file_name = func(&mut self.rng)?;

			// Correct lowercase "i" in image names IF they're going to be in the same directory.
			if image_icon_same_dir && file_name.name.as_bytes()[0] == b'i' {
				file_name.name.replace_range(0..1, "I");
			}

			if !model::does_image_name_exist(&file_name.name, collection).await? {
				break Ok(file_name);
			}
		}
	}
}


pub fn get_next_filename_unchecked(rng: &mut ThreadRng) -> Result<Filename> {
	let prefix_pos = rng.gen_range(0..PREFIXES.len());
	let suffix_pos = rng.gen_range(0..SUFFIXES.len());

	let filename = format!(
		"{}{}{}",
		&PREFIXES[prefix_pos],
		&SUFFIXES[suffix_pos],
		gen_three_numbers(rng)
	);

	Filename::new(filename, None)
}

pub fn gen_three_numbers(rng: &mut ThreadRng) -> String {
	(0..3).fold(String::new(), |mut v, _| {
		v.push(char::from_u32(rng.gen_range(48..=57)).unwrap());
		v
	})
}

pub fn gen_sample_alphanumeric(amount: usize, rng: &mut ThreadRng) -> String {
	rng.sample_iter(Alphanumeric)
		.take(amount)
		.map(char::from)
		.collect()
}

pub fn gen_uuid() -> String {
	uuid::Uuid::new_v4().to_hyphenated().to_string()
}
