use webkit6::{WebView, prelude::WebViewExt};

pub struct BrowserEngine;

impl BrowserEngine {
    pub fn configure(view: &WebView, developer_tools: bool) {
        if let Some(settings) = webkit6::prelude::WebViewExt::settings(view) {
            settings.set_enable_developer_extras(developer_tools);
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

    pub fn reload(view: &WebView) {
        view.reload();
    }
}
