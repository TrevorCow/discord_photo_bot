use std::fs;
use handlebars::Handlebars;
use once_cell::sync::Lazy;
use serde::Serialize;

const STYLES_CSS: &str = include_str!("resources/gallery_styles.css");
const GALLERY_JS: &str = include_str!("resources/gallery.js");
const GALLERY_HTML_TEMPLATE: &str = include_str!("resources/gallery_template.html");

static HANDLEBARS: Lazy<Handlebars> = Lazy::new(|| {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("html_template", GALLERY_HTML_TEMPLATE).expect("Error registering gallery html template");
    handlebars
});

#[derive(Serialize)]
pub struct PhotoInfo {
    pub(crate) url: String,
    pub(crate) picture_description: Option<String>,
}

#[derive(Serialize)]
pub struct GalleryInfo {
    pub(crate) title: String,
    pub(crate) picture_infos: Vec<PhotoInfo>,
}

#[derive(Serialize)]
pub struct PageInfo {
    pub(crate) page_title: String,
    pub(crate) page_build_info: String,
    pub(crate) galleries: Vec<GalleryInfo>,
}

pub fn build_website(page_info: PageInfo) {
    let built_html = HANDLEBARS.render("html_template", &page_info).unwrap();

    fs::write("styles.css", STYLES_CSS).expect("Unable to write file");
    fs::write("gallery.js", GALLERY_JS).expect("Unable to write file");
    fs::write("index.html", &built_html).expect("Unable to write file");
}