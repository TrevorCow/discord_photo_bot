use std::{fs, thread};
use std::path::Path;
use handlebars::Handlebars;
use once_cell::sync::Lazy;
use serde::Serialize;
use url::Url;

const WEBSITE_ROOT: &str = "build_website/";
const STYLES_CSS: &str = include_str!("resources/styles.css");
const GALLERY_JS: &str = include_str!("resources/gallery.js");
const GALLERY_HTML_TEMPLATE: &str = include_str!("resources/gallery_template.html");

static HANDLEBARS: Lazy<Handlebars> = Lazy::new(|| {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("html_template", GALLERY_HTML_TEMPLATE).expect("Error registering gallery html template");
    handlebars
});

#[derive(Serialize)]
pub struct PhotoInfo {
    pub(crate) url: Box<str>,
    pub(crate) thumbnail_url: Box<str>,
    pub(crate) picture_description: Option<Box<str>>,
}

#[derive(Serialize)]
pub struct GalleryInfo {
    pub(crate) title: Box<str>,
    pub(crate) picture_infos: Vec<PhotoInfo>,
}

#[derive(Serialize)]
pub struct PageInfo {
    pub(crate) page_title: Box<str>,
    pub(crate) page_build_info: Box<str>,
    pub(crate) galleries: Vec<GalleryInfo>,
}

pub fn save_thumbnail(url_str: &str) -> Box<str> {
    static THUMBNAIL_ABSOLUTE_FOLDER: Lazy<&Path> = Lazy::new(|| {
        static INNER_PATH: Lazy<String> = Lazy::new(|| { WEBSITE_ROOT.to_owned() + "thumbnails/" });
        Path::new(INNER_PATH.as_str())
    });
    if !THUMBNAIL_ABSOLUTE_FOLDER.exists() {
        fs::create_dir(*THUMBNAIL_ABSOLUTE_FOLDER).expect("Unable to create thumbnails folder!");
    }

    let url = Url::parse(url_str).unwrap();
    let file_name = Path::new(&url.path()).file_name().unwrap().to_owned().into_string().unwrap();
    let save_as = format!("{}{}", THUMBNAIL_ABSOLUTE_FOLDER.to_str().unwrap(), file_name);

    let return_path = Box::from(save_as.strip_prefix(WEBSITE_ROOT).unwrap());

    println!("Trying to save thumbnail: {:?}", return_path);
    thread::spawn(move || {
        let response = reqwest::blocking::get(url.clone()).unwrap();
        let bytes = response.bytes().unwrap();
        let image = image::load_from_memory(&bytes).unwrap();

        let thumbnail_image = image.thumbnail(200, 200);

        match thumbnail_image.save(&save_as) {
            Ok(_) => {
                println!("Successfully saved thumbnail `{}`", file_name)
            }
            Err(err) => {
                eprintln!("Error saving thumbnail `{}`: {}", save_as, err)
            }
        }
    });

    return_path
}


pub fn clean_website_folder() {
    if Path::new(WEBSITE_ROOT).exists() {
        fs::remove_dir_all(WEBSITE_ROOT).expect("Could not clean the website folder");
    }
    fs::create_dir(WEBSITE_ROOT).expect("Failed to create website folder");
}

pub fn build_website(page_info: PageInfo) {
    let built_html = HANDLEBARS.render("html_template", &page_info).unwrap();

    fs::write(WEBSITE_ROOT.to_owned() + "styles.css", STYLES_CSS).expect("Unable to write file");
    fs::write(WEBSITE_ROOT.to_owned() + "gallery.js", GALLERY_JS).expect("Unable to write file");
    fs::write(WEBSITE_ROOT.to_owned() + "index.html", &built_html).expect("Unable to write file");
}