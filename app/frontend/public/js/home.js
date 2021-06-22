'use strict';

(function(document, window) {
	var forms = document.querySelectorAll('.box');

	var uploadAdvanced = 'FormData' in window && 'FileReader' in window;

	if (!uploadAdvanced) return;//TODO: Temp.

	// file.type.indexOf('image')
	forms.forEach(function(form) {
		var input = form.querySelector('input[type="file"]');
		var label = form.querySelector('label');
		var filesPending = [];

		[ 'drag', 'dragstart', 'dragend', 'dragover', 'dragenter', 'dragleave', 'drop' ]
		.forEach(function(eventName) {
			form.addEventListener(eventName, function(event) {
				event.preventDefault();
				event.stopPropagation();
			});
		});


		form.addEventListener('drop', function(event) {
			var reader = new FileReader();

			var file = event.dataTransfer.files[0];

			reader.onload = function(event) {
				uploadImage(file, file.type);
			}

			reader.readAsDataURL(file);
		});
	});

	function convertToBase64(url, imagetype, callback) {
		var img = document.createElement('img');
		var canvas = document.createElement('canvas');
		var ctx = canvas.getContext('2d');
		var data = '';

		img.crossOrigin = 'Anonymous'

		img.onload = function() {
			canvas.height = this.height;
			canvas.width = this.width;
			ctx.drawImage(this, 0, 0);
			data = canvas.toDataURL(imagetype);
			callback(data);
		};

		img.src = url;
	}
	
	function sendBase64ToServer(base64) {
		var formData = new FormData();
		formData.append('image', base64);

		var httpPost = new XMLHttpRequest();

		httpPost.onreadystatechange = function(err) {
			if (httpPost.readyState == 4 && httpPost.status == 200) console.log(httpPost.responseText);
			else console.log(err);
		};

		httpPost.open('POST', '/upload', true);
		httpPost.send(formData);
	}

	function uploadImage(src, type) {
		// convertToBase64(src, type, function(data) {
			sendBase64ToServer(src);
		// });
	}
})(document, window);