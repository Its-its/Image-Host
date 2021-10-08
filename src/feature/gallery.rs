use actix_web::Scope;

use crate::config::Config;
use crate::web::gallery;




pub fn register(scope: Scope, config: &Config) -> Scope {
	if config.features.gallery.enabled {
		scope
		.service(gallery::home)
		.service(gallery::item)
		.service(gallery::gallery_new)
		.service(gallery::gallery_delete)
		.service(gallery::gallery_update)
		.service(gallery::gallery_image_list)
	} else {
		scope
	}
}