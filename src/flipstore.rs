use std::{cell::UnsafeCell, sync::{Mutex, Arc, MutexGuard}, ops::{Deref, DerefMut}};

use crate::Result;

// Allows reading and writing simultaneously.
pub struct FlipStore<D: Clone> {
	/// The current one which can be read from.
	reader: UnsafeCell<Arc<StoreData<D>>>,
	/// The current one which is being written to.
	writer: Mutex<StoreData<D>>,
}

unsafe impl<D: ?Sized + Send + Clone> Send for FlipStore<D> {}
unsafe impl<D: ?Sized + Send + Sync + Clone> Sync for FlipStore<D> {}

impl<D: Clone> FlipStore<D> {
	pub fn new(value: D) -> Self {
		Self {
			reader: UnsafeCell::new(Arc::new(StoreData::new(value.clone()))),
			writer: Mutex::new(StoreData::new(value)),
		}
	}

	pub fn read(&self) -> FlipReader<D> {
		unsafe {
			FlipReader::new((&*self.reader.get()).clone())
		}
	}

	pub fn write(&self) -> Result<FlipWriter<'_, D>> {
		Ok(FlipWriter::new(self, self.writer.lock()?))
	}
}


#[derive(Clone)]
struct StoreData<D: Clone> {
	data: D,
}

impl<D: Clone> StoreData<D> {
	pub fn new(value: D) -> Self {
		Self {
			data: value,
		}
	}
}


pub struct FlipReader<D: Clone> {
	store: Arc<StoreData<D>>,
}

impl<D: Clone> FlipReader<D> {
	fn new(store: Arc<StoreData<D>>) -> Self {
		Self {
			store
		}
	}
}

impl<D: Clone> Deref for FlipReader<D> {
	type Target = D;

	fn deref(&self) -> &Self::Target {
		&self.store.data
	}
}


pub struct FlipWriter<'a, D: Clone> {
	store: &'a FlipStore<D>,
	guard: MutexGuard<'a, StoreData<D>>,
}

impl<'a, D: Clone> FlipWriter<'a, D> {
	fn new(
		store: &'a FlipStore<D>,
		guard: MutexGuard<'a, StoreData<D>>
	) -> Self {
		Self {
			store,
			guard
		}
	}
}

impl<'a, D: Clone> Deref for FlipWriter<'a, D> {
	type Target = D;

	fn deref(&self) -> &Self::Target {
		&self.guard.data
	}
}

impl<'a, D: Clone> DerefMut for FlipWriter<'a, D> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.guard.data
	}
}

impl<'a, D: Clone> Drop for FlipWriter<'a, D> {
	fn drop(&mut self) {
		let clone = (*self.guard).clone();

		let readable = std::mem::replace(&mut *self.guard, clone);

		unsafe {
			let v = &mut *self.store.reader.get();
			*v = Arc::new(readable);
		}
	}
}

