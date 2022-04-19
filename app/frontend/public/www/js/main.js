$(document).ready(function() {
	const MONTHS = [
		'January', 'February', 'March',
		'April', 'May', 'June',
		'July', 'August', 'September',
		'October', 'November', 'December'
	];

	class ViewOptions {
		constructor() {
			this.container = $('.years-months');
			this.years = this.container.find('.years');
			this.months = this.container.find('.months');

			this.activeYear = null;
			this.activeMonth = null;

			var self = this;

			this.years.on('click', 'span', function() {
				var year = parseInt(this.innerText);

				if (self.activeYear != year) {
					var backwards = self.activeYear > year;
					self.viewYear(year);

					var months = uploader.storage.years[self.activeYear];
					self.viewMonth((backwards ? months[months.length - 1] : months[0]) + 1);
				}
			});

			this.months.on('click', 'span', function() {
				var month = 1 + [ 'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec' ].indexOf(this.innerText);

				if (self.activeMonth != month) {
					self.viewMonth(month);
				}
			});
		}

		setYears() {
			var joinDate = new Date(uploader.joinDate);
			var currDate = new Date();

			while(joinDate.getFullYear() <= currDate.getFullYear()) {
				var currentYear = joinDate.getFullYear();

				var span = document.createElement('span');
				span.classList.add('valid');
				span.innerText = currentYear;
				this.years.append(span);

				var months = [];

				if (currentYear != currDate.getFullYear()) {
					var remaining = joinDate.getMonth();

					while(remaining < 12) months.push(remaining++);

					joinDate.setMonth(0);
					joinDate.setYear(currentYear + 1);
				} else {
					var remaining = 0;

					while(remaining <= currDate.getMonth()) months.push(remaining++);

					joinDate.setYear(currentYear + 1);
				}

				uploader.storage.years[currentYear] = months;
			}
		}

		viewYear(year) {
			if (this.activeYear == year) return;

			if (this.activeYear != null) this.years.children()[this.activeYear - uploader.joinDate.getFullYear()].classList.remove('active');
			this.years.children()[year - uploader.joinDate.getFullYear()].classList.add('active');

			this.unsetMonths();

			var yearMonths = uploader.storage.years[year];

			var self = this;

			// var months = [ 'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec' ];

			yearMonths.forEach(function(month) {
				self.months.children()[month].classList.add('valid');
			});

			this.activeYear = year;
		}

		viewMonth(month) {
			if (this.activeMonth == month || !this.months.children()[month - 1].classList.contains('valid')) return;

			if (this.activeMonth != null) this.months.children()[this.activeMonth - 1].classList.remove('active');
			this.months.children()[month - 1].classList.add('active');

			uploader.storage.getImages(this.activeYear, month, function(data) {
				if (data.err != null) return console.error(data.err);

				var year = data.response.year;
				/** @type any[] */
				var images = data.response.images;
				console.log(images);
				var images = images.filter(img => !img.deleted);
				console.log(images);

				$('#images').empty();
				$('#images').append(uploader.createMonthContainer(month, images));

				setTimeout(function() { $('img').unveil(); }, 300);
			});

			this.activeMonth = month;
		}

		unsetMonths() {
			this.months.children()
			.each(function(_, item) {
				item.classList.remove('active', 'valid');
			});
			this.activeMonth = null;
		}

		isValid(month) {
			//
		}
	}


	class Storage {
		constructor() {
			// [year][month] = [images];
			this.items = {};
			this.years = {};
		}

		getImages(year, month, callback) {
			if (year == null || month == null) return callback([]);

			if (this.isCached(year, month)) {
				return callback(this.items[year][month]);
			}

			var self = this;

			$.get('/user/images', {
				year: year,
				month: month
			}, function(images) {
				self.cacheImages(year, month, images);
				callback(images);
			});
		}

		isCached(year, month) {
			return this.items[year] != null && this.items[year][month] != null;
		}

		cacheImages(year, month, images) {
			if (this.items[year] == null)
				this.items[year] = {};
			this.items[year][month] = images;
		}
	}


	$(document).foundation();
	new Clipboard('.copy');

	var uploader = {
		uploadType: null,
		uniqueID: null,
		joinDate: null,
		chart: null,
		storage: new Storage(),
		viewOptions: new ViewOptions(),
		hover: {
			timeout: null,
			image: null
		},
		createImage: function(name, views, favorited, type) {
			let image = document.createElement('div');
			image.classList.add('img-info', 'large-2');


			let clickable = document.createElement('a');
			clickable.classList.add('thumbnail');

			clickable.setAttribute('data-target', name);
			clickable.setAttribute('data-type', type);

			// Image

			let img = document.createElement('img');
			img.classList.add('img');
			img.setAttribute('data-src', `//${window.ICON_HOST}/i` + name + '.png');
			img.setAttribute('align', 'center');
			img.setAttribute('alt', 'Loading.');

			img.addEventListener('click', event => {
				if (event.shiftKey) {
					let win = window.open(`https://${window.IMAGE_HOST}/${name}.png`, '_blank');
					win.focus();
				}
			});

			clickable.appendChild(img);

			// Hover Left
			let spanTopLeft = document.createElement('span');
			spanTopLeft.classList.add('hover', 'left');

			let iTopLeft = document.createElement('i');
			iTopLeft.classList.add('fa', 'fa-eye');
			iTopLeft.setAttribute('aria-hidden', 'true');
			spanTopLeft.appendChild(iTopLeft);

			let numb = document.createElement('span');
			numb.innerText = views;
			spanTopLeft.appendChild(numb);

			clickable.appendChild(spanTopLeft);

			// Hover Right
			let spanTopRight = document.createElement('span');
			spanTopRight.classList.add('hover', 'right', 'favorite');

			let iTopRight = document.createElement('i');
			iTopRight.classList.add('fa', 'fa-star');
			iTopRight.setAttribute('aria-hidden', 'true');
			iTopRight.style.color = favorited ? 'yellow' : 'white';
			spanTopRight.appendChild(iTopRight);

			clickable.appendChild(spanTopRight);

			// Hover Bottom Right
			let spanBotRight = document.createElement('span');
			spanBotRight.classList.add('hover', 'bottom-right');

			let iBotRight = document.createElement('i');
			iBotRight.classList.add('fa', 'fa-clipboard', 'copy');
			iBotRight.setAttribute('data-clipboard-text', `https://${window.IMAGE_HOST}/` + name + '.' + type);
			iBotRight.setAttribute('aria-hidden', 'true');
			spanBotRight.appendChild(iBotRight);

			clickable.appendChild(spanBotRight);


			// Hover Bottom Left
			let spanBotLeft = document.createElement('span');
			spanBotLeft.classList.add('hover', 'bottom-left');

			let iBotLeft = document.createElement('i');
			iBotLeft.classList.add('fa', 'fa-times', 'delete');
			iBotLeft.setAttribute('aria-hidden', 'true');
			spanBotLeft.appendChild(iBotLeft);


			spanBotLeft.addEventListener('click', () => {
				if (window.event.shiftKey) {
					remove();
				} else if (window.confirm('Are you use you would like to delete this file?')) {
					remove();
				}
			});

			function remove() {
				if (image.parentElement) image.parentElement.removeChild(image);

				const oReq = new XMLHttpRequest();
				oReq.open('DELETE', `/image/${name}`);
				oReq.send();
			}

			clickable.appendChild(spanBotLeft);

			image.appendChild(clickable);

			return image;
		},
		createMonthContainer: function(month, images) {
			var container = document.createElement('div');

			var title = document.createElement('h3');
			title.innerText = MONTHS[month - 1];
			container.appendChild(title);

			var imageContainer = document.createElement('div');
			imageContainer.className = 'row large-12';

			images.forEach(image => imageContainer.appendChild(uploader.createImage(image.name, image.view_count, image.is_favorite, image.file_type)));

			container.appendChild(imageContainer);

			return container;
		}
	};

	// On hover image
	$('#images', '.img-info').hover(function() {
		console.log('hover');
		if (uploader.hover.image == this) return;

		if (uploader.hover.image != null) {
			// Remove hover overlay.
		}

		console.log(this);
		// setTimeout(function() {}, 1000);
	}, function() {
		if (uploader.hover.timeout != null) clearTimeout(uploader.hover.timeout);
		uploader.hover.timeout = null;
	});

	// Get Options
	$.get('/user/settings', function(data) {
		window.ICON_HOST = data.icon_host;
		window.IMAGE_HOST = data.image_host;

		uploader.joinDate = new Date(data.join_date);
		uploader.uniqueID = data.unique_id;
		uploader.uploadType = data.upload_type;

		uploader.viewOptions.setYears();
		uploader.viewOptions.viewYear(new Date().getFullYear());
		uploader.viewOptions.viewMonth(new Date().getMonth() + 1);

		$(`#urlTypeForm input[value="${data.uploadType}"]`).attr('checked', '');

		$('#urlTypeForm').submit(function() {
			var data = $(this).serialize();
			$.post('user/settings', data);
			return false;
		});
	});

	// Show Settings
	$('#showSettings').on('click', function() {
		if ($('#settings').css('display') != 'block') {
			$('#settings').css('display', 'block');
			document.getElementById('uniqueID').innerText = uploader.uniqueID;
		} else {
			$('#settings').css('display', 'none');
			document.getElementById('uniqueID').innerText = '';
		}
	});

	$(window).resize(function() { uploader.chart.highcharts().reflow(); });

	uploader.chart = $('#chart').highcharts({
		chart: { type: 'spline' },
		title: { text: 'Image Information' },
		subtitle: { text: 'Dates the picture has been clicked' },
		xAxis: {
			type: 'datetime',
			minRange: 3600000,
			title: {
				text: 'Date'
			}
		},
		yAxis: {
			title: { text: 'Amount' },
			min: 0
		},
		tooltip: {
			headerFormat: '<b>{series.name}</b><br>',
			pointFormat: '{point.x: %I%p, %e. %b} | {point.y} view(s)'
		},
		plotOptions: {
			spline: {
				marker: {
					enabled: true
				}
			}
		},
		series: []
	});
});
