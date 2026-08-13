use webkit6::{WebView, prelude::WebViewExt};

pub struct BrowserEngine;

impl BrowserEngine {
    pub fn configure(view: &WebView) {
        if let Some(settings) = webkit6::prelude::WebViewExt::settings(view) {
            settings.set_enable_developer_extras(true);

            settings.set_enable_javascript(true);

            settings.set_enable_html5_local_storage(true);

            settings.set_enable_html5_database(true);

            settings.set_enable_media(true);

            settings.set_enable_mediasource(true);

            settings.set_enable_media_capabilities(true);

            settings.set_enable_media_stream(true);

            settings.set_enable_webrtc(true);

            settings.set_enable_encrypted_media(true);

            settings.set_enable_fullscreen(true);
        }
    }

    pub fn load(view: &WebView, uri: &str) {
        view.load_uri(uri);
    }

    pub fn back(view: &WebView) {
        if view.can_go_back() {
            view.go_back();
        }
    }

    pub fn forward(view: &WebView) {
        if view.can_go_forward() {
            view.go_forward();
        }
    }

    pub fn reload(view: &WebView) {
        view.reload();
    }
}
