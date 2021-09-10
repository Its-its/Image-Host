use std::fs::File;
use std::io::{BufRead, BufReader};

use mime::{Mime, GIF, PNG, JPEG};
use rand::distributions::Alphanumeric;
use rand::prelude::{ThreadRng, Rng};

use crate::Result;
use crate::db::ImagesCollection;
use crate::model;

pub static APP_PATH: &str = "./app";


lazy_static! {
	pub static ref PREFIXES: Vec<String> = {
		let mut items: Vec<String> = Vec::new();

		let prefix_dir = format!("{}/prefixes", APP_PATH);

		let files = std::fs::read_dir(prefix_dir).expect("reading prefix dir");

		for entry in files {
			let file = File::open(entry.unwrap().path()).unwrap();
			let reader = BufReader::new(file);

			items.extend(reader.lines().map(|l| l.unwrap()));
		}

		items
	};

	pub static ref SUFFIXES: Vec<String> = {
		let mut items: Vec<String> = Vec::new();

		let prefix_dir = format!("{}/suffixes", APP_PATH);

		let files = std::fs::read_dir(prefix_dir).expect("reading prefix dir");

		for entry in files {
			let file = File::open(entry.unwrap().path()).unwrap();
			let reader = BufReader::new(file);

			items.extend(reader.lines().map(|l| l.unwrap()));
		}

		items
	};
}


#[derive(Debug, Clone)]
pub struct Filename {
	name: String,
	format: Option<String>
}

impl Filename {
	pub fn new(name: String) -> Self {
		Self { name, format: None }
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn into_name(self) -> String {
		self.name
	}

	pub fn is_accepted(&self) -> bool {
		self.format.is_some() && self.format() != "error"
	}

	pub fn format(&self) -> &str {
		let format: Option<Mime> = self.format.as_ref().and_then(|i| i.parse().ok());

		match format.as_ref().map(|f| f.subtype()) {
			Some(GIF) => "gif",
			Some(PNG) => "png",
			Some(JPEG) => "jpeg",
			_ => "error"
		}
	}

	pub fn set_format(mut self, format: String) -> Self {
		self.format = Some(format);
		self
	}

	pub fn as_filename(&self) -> String {
		format!("{}.{}", self.name(), self.format())
	}
}

// TODO: Change to Parse
impl From<&str> for Filename {
	fn from(file: &str) -> Self {
		let mut split = file.rsplitn(2, '.');

		let mut format = split.next();

		let name = split.next().or_else(|| format.take()).unwrap();

		Self {
			name: name.to_string(),
			format: format.map(|v| format!("image/{}", v))
		}
	}
}


impl From<String> for Filename {
	fn from(file: String) -> Self {
		let mut split = file.rsplitn(2, '.');

		let mut format = split.next();

		let name = split.next().or_else(|| format.take()).unwrap();

		Self {
			name: name.to_string(),
			format: format.map(|s| format!("image/{}", s))
		}
	}
}



#[derive(Debug, Clone)]
pub struct WordManager {
	pub rng: ThreadRng
}


impl WordManager {
	pub async fn get_next_filename_prefix_suffix(
		&mut self,
		collection: &ImagesCollection
	) -> Result<Filename> {
		self.loop_and_check_model_db(|rng| get_next_filename_unchecked(rng), collection).await
	}

	pub async fn get_next_filename_sized_8(
		&mut self,
		collection: &ImagesCollection
	) -> Result<Filename> {
		self.loop_and_check_model_db(|rng| Filename::new(gen_sample_alphanumeric(8, rng)), collection).await
	}

	pub async fn get_next_filename_sized_32(
		&mut self,
		collection: &ImagesCollection
	) -> Result<Filename> {
		self.loop_and_check_model_db(|rng| Filename::new(gen_sample_alphanumeric(32, rng)), collection).await
	}

	async fn loop_and_check_model_db(
		&mut self,
		func: impl Fn(&mut ThreadRng) -> Filename,
		collection: &ImagesCollection
	) -> Result<Filename> {
		loop {
			let file_name = func(&mut self.rng);

			if !model::does_image_name_exist(file_name.name(), collection).await? {
				break Ok(file_name);
			}
		}
	}
}

impl Default for WordManager {
	fn default() -> Self {
		Self {
			rng: ThreadRng::default()
		}
	}
}

pub fn get_next_filename_unchecked(rng: &mut ThreadRng) -> Filename {
	let prefix_pos = rng.gen_range(0..PREFIXES.len());
	let suffix_pos = rng.gen_range(0..SUFFIXES.len());

	let filename = format!("{}{}{}", &PREFIXES[prefix_pos], &SUFFIXES[suffix_pos], gen_three_numbers(rng));

	Filename::new(filename)
}

pub fn gen_three_numbers(rng: &mut ThreadRng) -> String {
	(0..3)
	.fold(
        String::new(),
        |mut v, _| {
            v.push(char::from_u32(rng.gen_range(48..=57)).unwrap());
            v
        }
	)
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