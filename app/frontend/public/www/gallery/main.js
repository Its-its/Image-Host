/**
 * @type {HTMLInputElement}
 */
const input = document.getElementById('exampleFileUpload');

input.addEventListener('change', () => {
	for (const file of input.files) {
		console.log(file);
	}
});