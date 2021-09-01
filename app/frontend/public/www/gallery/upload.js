const IS_NEW_GALLERY = window.location.pathname.toLowerCase().includes('/new');


const FILE_UPLOAD_PROMPTS = document.getElementsByClassName('file-upload-prompt');
const EDITING_CONTAINER = document.getElementById('editing-container');


const MEDIA_FILES = [];
const REMOVING_FILE_IDS = {};


function updateEditingContainer() {
	while (EDITING_CONTAINER.firstChild != null) EDITING_CONTAINER.firstChild.remove();

	let uploading_files_length = MEDIA_FILES.filter(v => v.isNeedingUpload()).length;;
	let removing_files_length = Object.keys(REMOVING_FILE_IDS).length;

	if (uploading_files_length != 0 || removing_files_length != 0) {
		if (IS_NEW_GALLERY) {
			let save_button = document.createElement('button');
			save_button.className = 'button success';
			save_button.innerText = 'Create';
			EDITING_CONTAINER.appendChild(save_button);

			save_button.addEventListener('click', () => {
				let files = MEDIA_FILES.filter(v => v.isNeedingUpload());

				console.log(files);

				for(let file of files) {
					file.upload()
					.then(console.log, console.log);
				}
			});
		} else {
			let save_button = document.createElement('button');
			save_button.className = 'button success';
			save_button.innerText = 'Save';
			EDITING_CONTAINER.appendChild(save_button);
		}
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
		img.src = this.is_data ? this.file_data : `//i.thick.at/i${this.media_info.name}.png`;
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
		img.src = this.is_data ? this.file_data : `//i.thick.at/${this.media_info.name}.${this.media_info.file_type}`;
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
				url: '/upload',
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

// [
// 	{
// 		"name": "AccompanyingZorse777",
// 		"file_type": "jpeg",
// 		"file_size": 27422,
// 		"is_edited": false,
// 		"is_favorite": false,
// 		"view_count": 0,
// 		"upload_date": "2021-08-26T04:03:28.548Z"
// 	},
// 	{
// 		"name": "DeificDjangoDjango019",
// 		"file_type": "png",
// 		"file_size": 27422,
// 		"is_edited": false,
// 		"is_favorite": false,
// 		"view_count": 0,
// 		"upload_date": "2021-08-26T04:09:36.498Z"
// 	}
// ].forEach(displayImage);



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
