// Data Storage
var Portion = function(data) {
	this.data = data;
}

// Populate template with data
Portion.prototype.populate = function() {
	var copyOf = this.data;

	for (var i = 0; i < arguments.length; i++) {
		copyOf = copyOf.replace(new RegExp("%" + i, "g"), arguments[i]);
	}

	return copyOf;
}


// Central
var Template = function() {
	this._v = 0.1;
	this.templates = [];
}

// Add template to storage
Template.prototype.add = function(name, template) {
	this.templates[name.toLowerCase()] = new Portion(template);
}

// Get Template Portion from storage
Template.prototype.get = function(name) {
	return this.templates[name.toLowerCase()];
}


// MAKE ZIS GLOBAL!
window.templates = new Template();
