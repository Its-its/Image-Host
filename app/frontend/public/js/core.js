$(document).ready(function() {
	// Foundation thingy
	$(document).foundation();

	// Clipboard thingy
	new Clipboard('.copy');

	// Template thingy
	templates.add('image', `
		<div class="img-info large-2">
			<a data-target="%0" data-type="%3" class="thumbnail">
				<img align="center" class="img" src="" data-src="//i.thick.at/i%0.png" alt="Image not loading.">
				<span class="hover left">
					<i class="fa fa-eye" aria-hidden="true"></i> %1
				</span>
				<span class="hover right favorite">
					<i class="fa fa-star" aria-hidden="true" style="color: %2;"></i>
				</span>
				<span class="hover bottom-right">
					<i class="fa fa-clipboard copy" aria-hidden="true" data-clipboard-text="https://i.thick.at/%0.%3"></i>
				</span>
			</a>
		</div>`);

	// Sort thingy

	(function() {
		'use strict';
		var currButton = null;
		var direction = -1;
		var html_up = '<i class="fa fa-caret-up" aria-hidden="true"></i>';
		var html_down = '<i class="fa fa-caret-down" aria-hidden="true"></i>';

		// Sort images
		$('#sorting').on('click', 'button', function() {
			if (currButton != this) {
				// Change button
				$(currButton).children('span').empty();
				currButton = this;
			} else {
				// Flip direction
				direction *= -1;
			}

			var span = $(this).children('span');
			var target = $(this).attr('data-target');

			span.empty();
			span.append(direction != 1 ? html_up : html_down);

			storage.sort({
				$loc: target,
				$dir: direction
			});

			var output = storage.output();
			var lastMove = output[0];

			$('#images').append($('a[data-target="' + lastMove.name + '"]').closest('div'));

			for (var i = 1; i < output.length; i++) {
				$('a[data-target="' + output[i].name + '"]').closest('div').insertAfter($('a[data-target="' + lastMove.name + '"]').closest('div'));
				lastMove = output[i];
			}

			setTimeout(function() {
				$('img').unveil();
			}, 300);
		});
	}());

	// Filter Images
	(function() {
		'use strict';
		var currButton = null;
		var html_select = '<i class="fa fa-circle" aria-hidden="true"></i>';

		$('#filtering').on('click', 'button', function() {
			if (currButton != this) {
				// Change button
				$(currButton).children('span').empty();
				currButton = this;
			} else return;

			var span = $(this).children('span');
			var target = $(this).attr('data-target');

			span.empty();
			span.append(html_select);

			storage.filter(target);

			if (target == 'all') {
				$('#images div[style="display: none;"]').css('display', 'inline-block');
			} else {
				$('#images div[style="display: none;"]').css('display', 'inline-block');
				$('#images a[data-type!="' + target + '"]').closest('div').css('display', 'none');
			}

			var output = storage.output();
			var lastMove = output[0];

			$('#images').append($('a[data-target="' + lastMove.name + '"]').closest('div'));

			for (var i = 1; i < output.length; i++) {
				$('a[data-target="' + output[i].name + '"]').closest('div').insertAfter($('a[data-target="' + lastMove.name + '"]').closest('div'));
				lastMove = output[i];
			}

			setTimeout(function() {
				$('img').unveil();
			}, 300);
		});
	}());

	// Add/Remove image to favorites
	$('#images').on('click', '.img-info span[class~="favorite"]', function() {
		var _this = $(this);
		var name = _this.parent('a').attr('data-target');
		var favorite = !storage.get({ $type: 'name', name: name }).favorite;
		storage.set({ $type: 'fullname', name: name }, { $type: 'favorite', value: favorite });
		$('a[data-target="' + name + '"] i[class~="fa-star"]').css('color', (favorite ? 'yellow' : 'white'));

		$.post('image/favorite', { name: name, newFav: favorite });
	});

	// Show image information
	$('body').on('click', '.img-info a', function(to) {
		// If we clicked on one of the other elements.
		if (to.toElement.tagName != 'IMG') return;

		var target = $(this).attr('data-target');
		$.post('image/info', { name: target }, function(image) {
			if(image == null) return;

			var highChart = highASFChart.highcharts();

			for (var i = 0; i < highChart.series.length; i++) {
				highChart.series[i].destroy();
			}

			var series = [];

			// Has the possibility to be null.
			if (image.viewDates) {
				var viewDateKeys = Object.keys(image.viewDates).sort();

				for (var o = 0; o < viewDateKeys.length; o++) {
					var key = viewDateKeys[o];
					series.push([ parseInt(key), image.viewDates[key] ]);
				}
			}

			highASFChart.highcharts()
			.addSeries({
				name: 'Views',
				data: series
			});

			$('#image').attr('src', '//i.thick.at/' + image.fullname);
			$('#name').text(image.fullname);
			$('#date').text(new Date(image.date).toLocaleString());
			$('#views').text(image.views);
			$('#bytes').text((image.views * image.size) * 0.0000000009313226);
			$('#modelImage').foundation('open')

			setTimeout(function() {
				highASFChart.highcharts().reflow();
			}, 500);
		});
	});

	// Has to be a better way thingy
	$('#urlTypeForm')[0][dataType + 1].checked = true;

	// Url type submittion thingy
	$('#urlTypeForm').submit(function() {
		var data = $(this).serialize();
		$.post('user/urltype', data);
		return false;
	});

	// On click remove
	$('#remove').on('click', function() {
		var name = $('#name').text();
		if (name == '') return;

		$.post('image/delete', { name: name }, function(data) {
			if (!data.error) {
				$('#modelImage').foundation('close');
				$('a[data-target="' + $('#name').text() + '"]').closest('div').remove();

				$('#image').attr('src', '');
				$('#name').text('');
				$('#date').text('');
				$('#views').text('');
				$('#bytes').text('');
			}
		});
	});

	// Get images
	$.post('images', function(data) {
		var icons = [];

		for (var i = 0; i < data.response.length; i++) {
			var image = data.response[i];
			image.fullname = image.name + '.' + image.type;//TODO: Remove this and implement full name.
			icons.push(templates.get('image').populate(image.name, image.views, (image.favorite ? 'yellow' : 'white'), image.type));
		}

		if (storage)
			for (var o = 0; o < data.response.length; o++) {
				storage.store(data.response[o]);
			}

		$('#images').append(icons);
		$('img').unveil();
	});

	var highASFChart = $('#container').highcharts({
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

	$(window).resize(function() {
		highASFChart.highcharts().reflow();
	});
});
