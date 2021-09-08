let IS_NEW_GALLERY = window.location.pathname.toLowerCase().includes('/new');


const FILE_UPLOAD_PROMPTS = document.getElementsByClassName('file-upload-prompt');
const EDITING_CONTAINER = document.getElementById('editing-container');


const MEDIA_FILES = [];
const REMOVING_FILE_IDS = {};



if (!IS_NEW_GALLERY) {
	(async function() {
		let update_resp = await fetch(`${window.location.pathname}/list`, {
			url: `${window.location.pathname}/list`,
			method: 'GET'
		});

		let images = await update_resp.json();

		images.forEach(file => {
			let media_file = new MediaFile(file);
			media_file.display();
			MEDIA_FILES.push(media_file);
		});
	}())
	.catch(console.log);
}


function updateEditingContainer() {
	while (EDITING_CONTAINER.firstChild != null) EDITING_CONTAINER.firstChild.remove();

	let uploading_files_length = MEDIA_FILES.filter(v => v.isNeedingUpload()).length;;
	let removing_files_length = Object.keys(REMOVING_FILE_IDS).length;

	if (uploading_files_length != 0 || removing_files_length != 0) {
		let save_button;

		if (IS_NEW_GALLERY) {
			save_button = document.createElement('button');
			save_button.className = 'button success';
			save_button.innerText = 'Create';
			EDITING_CONTAINER.appendChild(save_button);
		} else {
			save_button = document.createElement('button');
			save_button.className = 'button success';
			save_button.innerText = 'Save';
			EDITING_CONTAINER.appendChild(save_button);
		}

		save_button.addEventListener('click', () => {
			(async function() {
				if (IS_NEW_GALLERY) {
					// Create Gallery
					let new_resp = await fetch('/g/new', {
						url: '/g/new',
						method: 'POST'
					});

					let gallery_id = await new_resp.text();

					// Change URL.
					window.history.replaceState(null, '', `/g/${gallery_id}`);
				}


				// Upload Files.

				let files = MEDIA_FILES.filter(v => v.isNeedingUpload());

				console.log(files);

				for(let file of files) {
					await file.upload();
				}

				console.log(files);

				// Update Gallery
				let update_resp = await fetch(`${window.location.pathname}`, {
					url: `${window.location.pathname}`,
					method: 'POST',
					body: JSON.stringify({
						add: files.map(v => v.media_info.name),
						remove: Object.keys(REMOVING_FILE_IDS)
					}),
					headers: {
						'Content-Type': 'application/json'
					}
				});

				console.log(await update_resp.text());


				// Refresh Sidebar
				IS_NEW_GALLERY = false;
				updateEditingContainer();
			}())
			.then(console.log, console.log);
		});
	}


	if (uploading_files_length != 0) {
		let upload_count = document.createElement('h5');
		upload_count.innerHTML = `Uploading <b>${uploading_files_length}</b> image(s)`;
		EDITING_CONTAINER.appendChild(upload_count);
	}

	if (removing_files_length != 0) {
		let remove_count = document.createElement('h5');
		remove_count.innerHTML = `Removing <b>${removing_files_length}</b> image(s)`;
		EDITING_CONTAINER.appendChild(remove_count);
	}
}


class MediaFile {
	constructor(file, is_data = false) {
		this.is_data = is_data; // TODO: Change name.

		if (is_data) {
			this.file = file;
			// SlimImage
			this.file_data = null;
		} else {
			this.media_info = file;
		}

		this.container = document.createElement('div');
		this.container.className = 'image-container';
	}

	display() {
		if (this.is_data && this.file_data == null) {
			const file_reader = new FileReader();

			file_reader.onload = () => {
				this.file_data = file_reader.result;
				this.display();
			}

			file_reader.readAsDataURL(this.file);

			return;
		}

		this.showImage();

		if (this.container.parentElement == null) {
			document.getElementById('images-list').appendChild(this.container);
		}
	}

	showMarkForDeletion() {
		while (this.container.firstChild) this.container.firstChild.remove();
		this.container.classList.add('marked-for-deletion');

		updateEditingContainer();

		let image_cont = document.createElement('div');
		image_cont.className = 'image';
		this.container.appendChild(image_cont);

		let img = document.createElement('img');
		img.src = this.is_data ? this.file_data : `${window.DIRECT_IMAGE_URL}i${this.media_info.name}.png`;
		image_cont.appendChild(img);


		let undo_button = document.createElement('button');
		undo_button.className = 'button';
		undo_button.innerText = 'Undo Delete';
		this.container.appendChild(undo_button);

		undo_button.addEventListener('click', () => {
			delete REMOVING_FILE_IDS[this.media_info.name];
			this.showImage();
		});
	}

	showImage() {
		while (this.container.firstChild) this.container.firstChild.remove();
		this.container.classList.remove('marked-for-deletion');

		updateEditingContainer();

		let image_cont = document.createElement('div');
		image_cont.className = 'image';
		this.container.appendChild(image_cont);

		let delete_button = document.createElement('div');
		delete_button.innerText = 'X';
		delete_button.style = 'cursor: pointer; position: absolute; right: 5px; top: 5px; line-height: 1; font-weight: bold; font-size: 24px; color: red;';
		image_cont.appendChild(delete_button);

		delete_button.addEventListener('click', () => {
			if (this.is_data) {
				container.remove();
			} else {
				REMOVING_FILE_IDS[this.media_info.name] = true;
				this.showMarkForDeletion();
			}
		});

		let img = document.createElement('img');
		img.src = this.is_data ? this.file_data : `${window.DIRECT_IMAGE_URL}${this.media_info.name}.${this.media_info.file_type}`;
		image_cont.appendChild(img);
	}

	isNeedingUpload() {
		return this.is_data;
	}

	async upload() {
		let formData = new FormData();
		formData.append("image", this.file);

		let resp = await fetch(
			'/upload',
			{
				method: 'POST',
				body: formData
			}
		);

		let json = await resp.json();

		this.media_info = json;
		this.is_data = false;
		this.file = null;
		this.file_data = null;
	}
}


for (const element of FILE_UPLOAD_PROMPTS) {
	element.addEventListener('change', () => {
		for (const file of element.files) {
			console.log('FILE_UPLOAD_PROMPTS', file);

			let media_file = new MediaFile(file, true);
			media_file.display();
			MEDIA_FILES.push(media_file);

			// const file_reader = new FileReader();

			// file_reader.onload = () => displayImage(file_reader.result, file);

			// file_reader.readAsDataURL(file);
		}

		updateEditingContainer();
	});
}


// Drag and Drop Popup

const popup = document.getElementById('popup-file-drop');
let isHovering = false;

function preventEvent(event) {
	event.preventDefault();
	event.stopPropagation();
}

window.addEventListener('dragstart', preventEvent);
window.addEventListener('dragend', preventEvent);
window.addEventListener('dragover', preventEvent);
window.addEventListener('drag', preventEvent);

// Display Popup

window.addEventListener('dragenter', (event) => {
	preventEvent(event);

	if (!isHovering) {
		console.log(event.type);
		console.log(event);

		isHovering = true;
		popup.style.display = 'flex';
	}

});


// Remove Popups. One of them will emit on drag stop.

window.addEventListener('dragleave', (event) => {
	preventEvent(event);

	if (doesContainElement(event.target, popup)) {
		console.log(event.type);
		console.log(event);

		isHovering = false;
		popup.style.display = 'none';
	}

});

window.addEventListener('drop', (event) => {
	preventEvent(event);

	if (doesContainElement(event.target, popup)) {
		console.log(event.type);
		console.log(event);

		isHovering = false;
		popup.style.display = 'none';

		updateEditingContainer();
	}
});

/**
 *
 * @param {HTMLElement} element
 * @param {HTMLElement} value
 * @returns {boolean}
 */
function doesContainElement(element, value) {
	element == value || element.parentElement ? doesContainElement(element.parentElement) : false
}
