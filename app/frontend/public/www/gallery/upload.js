const file_upload_prompts = document.getElementsByClassName('file-upload-prompt');

for (const element of file_upload_prompts) {
	element.addEventListener('change', (event) => {
		for (const file of element.files) {
			console.log(file);
		}
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
