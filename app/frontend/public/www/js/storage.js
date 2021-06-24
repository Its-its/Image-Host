// Coded a long time ago when I was newer with JS. Needs a recode :/

function Storage(config) {
	this.storage = [];

	this.maxBatch = 20;
	this.batch = 0;

	// What is it storing?
	this.type = Array;
	this.filterType = null;

	//
	this.locations = {};

	if (config != null) {
		if (config.type != null) {
			this.type = config.type;
		}

		if (config.locations != null) {
			this.locations = config.locations;
		}
	}
}

Storage.prototype.resetBatch = function() {
	this.batch = 0;
}

Storage.prototype.store = function(item) {
	this.storage.push(item);
	return this;
}

Storage.prototype.filter = function(by) {
	if (by == null || by == 'all') {
		this.filterType = null;
	} else {
		this.filterType = by.toLowerCase();
	}

	return this;
}

Storage.prototype.sort = function(by) {
	this.storage.sort(this.getSortFunction(by));
	return this;
}

Storage.prototype.outputAll = function() {
	return this.storage;
}

Storage.prototype.output = function() {
	var filtered = [];

	if (this.filterType == null) {
		filtered = this.storage;
	} else if (this.filterType == 'star') {
		for(var i = 0; i < this.storage.length; i++) {
			var item = this.storage[i];

			if (item.favorite) {
				filtered.push(item);
			}
		}
	} else if (this.filterType == 'jpg') {
		for(var i = 0; i < this.storage.length; i++) {
			var item = this.storage[i];

			if (item.type.toLowerCase() == 'jpg' || item.type.toLowerCase() == 'jpeg') {
				filtered.push(item);
			}
		}
	} else {
		for(var i = 0; i < this.storage.length; i++) {
			var item = this.storage[i];
			if (item.type.toLowerCase() == this.filterType) {
				filtered.push(item);
			}
		}
	}

	if (filtered.length - 1 < this.maxBatch * this.batch) return [];

	return filtered.slice(this.batch * this.maxBatch, (this.batch++) * this.maxBatch + this.maxBatch);
}

Storage.prototype.get = function(conf) {
	for (var i = 0; i < this.storage.length; i++) {
		var item = this.storage[i];
		if (item[conf.$type] == conf.name) {
			return item;
		}
	}
	return null;
}

Storage.prototype.set = function(conf, set) {
	for (var i = 0; i < this.storage.length; i++) {
		var item = this.storage[i];
		if (item[conf.$type] == conf.name) {
			item[set.$type] = set.value;
			return item;
		}
	}
	return null;
}

Storage.prototype.getSortFunction = function(sortBy) {
	var _this = this;
	var loc = sortBy.$loc = (sortBy.$loc != null && sortBy.$loc == '') ? null : sortBy.$loc;
	var type = sortBy.$type = sortBy.$type || (this.locations[loc] != null ? this.locations[loc].type : String);
	var dir = sortBy.$dir = sortBy.$dir || 1;

	switch (type) {
		case Number:
		case Boolean:
			return function(a, b) {
				var aa = get(a);
				var bb = get(b);
				return dir == -1 ? aa - bb : bb - aa;
			}
		case Date:
			return function(a, b) {
				var aa = new Date(get(a)).getTime();
				var bb = new Date(get(b)).getTime();
				return dir == -1 ? aa - bb : bb - aa;
			}
		case String:
			return function(a, b) {
				var aa = get(a).length;
				var bb = get(b).length;
				return dir == -1 ? aa - bb : bb - aa;
			}
		default:
			return function() { return 0; }
	}

	function get(obj) {
		// If we're storing Objects or Arrays.
		if (_this.type == Object || _this.type == Array) {
			if (loc != null) {
				var locations = loc.split('.');
				var finalLoc = obj;
				for (var i = 0; i < locations.length; i++) {
					finalLoc = finalLoc[locations[i]];
				}
				return finalLoc;
			}
		}
		return obj;
	}
}

window.storage = new Storage(
	{
		type: Object,
		locations: {
			'views': {
				type: Number
			},
			'date': {
				type: Date
			},
			'favorite': {
				type: Boolean
			},
			'type': {
				type: String
			}
		}
	}
);
