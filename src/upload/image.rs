use serde::{Serialize, Serializer, Deserialize, Deserializer};

use crate::{Filename, Result, WordManager, db::get_images_collection};


// 0: Random Name	HyperSnowmobile123
//
// 1: simple-uid	08 bytes | 0348cd3f
//
// 2: extended-uid	32 bytes | ba0cc6361b4a28b8c08d1a36afa2a9a1
//
// 3: Encrypted		88 bytes | #c08d1a30348cd3f6afabaf0cc63b0348cd3a0cc6361b4a28!2jkh34v523gfpsd0fwasmxnvczcvn5435dman34r
// 							 | # unique Identifier (48 bytes)                   ! Deobfuscation Code (40 bytes)


#[derive(Debug, Clone, Copy)]
pub enum UploadImageType {
	PrefixAndSuffix = 0,
	Alphabetical8,
	Alphabetical32,
	// Crypto
}

impl UploadImageType {
	pub fn from_num(value: u8) -> Option<Self> {
		Some(match value {
			0 => Self::PrefixAndSuffix,
			1 => Self::Alphabetical8,
			2 => Self::Alphabetical32,
			_ => return None
		})
	}

	pub fn to_num(self) -> u8 {
		self as u8
	}

	pub async fn get_link_name(self, words: &mut WordManager) -> Result<Filename> {
		let collection = get_images_collection();

		match self {
			Self::PrefixAndSuffix => words.get_next_filename_prefix_suffix(&collection).await,
			Self::Alphabetical8 => words.get_next_filename_sized_8(&collection).await,
			Self::Alphabetical32 => words.get_next_filename_sized_32(&collection).await,
			// Self::Crypto =>
		}
	}
}

impl From<UploadImageType> for u8 {
    fn from(val: UploadImageType) -> Self {
        val.to_num()
    }
}


impl Serialize for UploadImageType {
	fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
		serializer.serialize_u8(self.to_num())
	}
}

impl<'de> Deserialize<'de> for UploadImageType {
	fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: Deserializer<'de> {
		Ok(Self::from_num(u8::deserialize(deserializer)?).unwrap())
	}
}