use std::{fs, thread};
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use bytes::Buf;
use chrono::{DateTime, Local};

use futures::future::join_all;
use handlebars::Handlebars;
use image::{EncodableLayout, ImageFormat};
use image::imageops::FilterType;
use once_cell::sync::Lazy;
use serde::Serialize;

use tokio::runtime::{Runtime};
use url::Url;

const WEBSITE_ROOT: &str = "built_website/";
const WEBSITE_RESOURCES_FOLDER_NAME: &str = "resources/";
const WEBSERVER_PYTHON_SRC: &str = include_str!("resources/webserver.py");
const STYLES_CSS_SRC: &str = include_str!("resources/styles.css");
const GALLERY_JS_SRC: &str = include_str!("resources/gallery.js");
const GALLERY_HTML_TEMPLATE: &str = include_str!("resources/gallery_template.html");

static HANDLEBARS: Lazy<Handlebars> = Lazy::new(|| {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("html_template", GALLERY_HTML_TEMPLATE).expect("Error registering gallery html template");
    handlebars
});

static THUMBNAIL_DOWNLOAD_MANAGER: Lazy<Mutex<ThumbnailDownloadManager>> = Lazy::new(|| {
    let tdm = ThumbnailDownloadManager::new();
    Mutex::new(tdm)
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
    pub(crate) page_build_info: PageBuildInfo,
    pub(crate) galleries: Vec<GalleryInfo>,
}

#[derive(Serialize)]
pub struct PageBuildInfo {
    pub(crate) guild_built_from: Box<str>,
    pub(crate) channel_built_from: Box<str>,
    pub(crate) user_built_by: Box<str>,
    pub(crate) built_time: Box<str>,
}

struct ThumbnailDownloadManager {
    queue: Arc<Mutex<Vec<(Url, String)>>>,
}

impl ThumbnailDownloadManager {
    pub fn new() -> Self {
        let queue = Arc::new(Mutex::new(Vec::new()));
        Self {
            queue,
        }
    }

    pub fn download_all(&mut self) {
        let queue_arc = self.queue.clone();

        thread::spawn(move || {
            let mut queue = queue_arc.lock().unwrap();
            let download_runtime = Runtime::new().unwrap();
            let mut download_futures = Vec::new();

            while let Some((url, save_as)) = queue.pop() {
                download_futures.push(
                    download_runtime.spawn(async move {
                        let thumbnail_image = {
                            let response = reqwest::get(url.clone()).await.unwrap();
                            let image_bytes = response.bytes().await.unwrap();
                            let image = image::load_from_memory(&image_bytes).unwrap();

                            // image.thumbnail(200, 200)
                            image.resize(250, 250, FilterType::Triangle)
                        };


                        match thumbnail_image.save_with_format(&save_as, ImageFormat::Jpeg) {
                            Ok(_) => {
                                println!("Successfully saved thumbnail `{save_as}`")
                            }
                            Err(err) => {
                                eprintln!("Error saving thumbnail `{save_as}`: {err}")
                            }
                        }
                    })
                );
            }
            drop(queue);

            download_runtime.block_on(join_all(download_futures));
        }).join().unwrap();
    }

    pub fn add_to_queue(&mut self, to_download: (Url, String)) {
        self.queue.lock().unwrap().push(to_download);
    }
}

pub fn save_thumbnail(url_str: &str) -> Box<str> {
    static THUMBNAIL_ABSOLUTE_FOLDER: Lazy<&Path> = Lazy::new(|| {
        static INNER_PATH: Lazy<String> = Lazy::new(|| { WEBSITE_ROOT.to_owned() + "thumbnails/" });
        Path::new(INNER_PATH.as_str())
    });
    if !THUMBNAIL_ABSOLUTE_FOLDER.exists() {
        fs::create_dir_all(*THUMBNAIL_ABSOLUTE_FOLDER).expect("Unable to create thumbnails folder!");
    }

    let url = Url::parse(url_str).unwrap();

    // let file_name = Path::new(&url.path()).file_name().unwrap().to_owned().into_string().unwrap(); // This might have file name collisions
    let file_name = url.path().to_owned()
        .replacen('/', "", 1)// Replace the leading `/`
        .replace('/', "_");
    let save_as = format!("{}{}", THUMBNAIL_ABSOLUTE_FOLDER.to_str().unwrap(), file_name);

    let return_path: Box<str> = Box::from(save_as.strip_prefix(WEBSITE_ROOT).unwrap());

    if !Path::new(&*save_as).exists() {
        THUMBNAIL_DOWNLOAD_MANAGER.lock().unwrap().add_to_queue((url, save_as));
    } else {
        println!("Already have thumbnail `{return_path}`... skipping");
    }

    return_path
}


pub fn clean_website_folder() {
    if Path::new(WEBSITE_ROOT).exists() {
        fs::remove_dir_all(WEBSITE_ROOT).expect("Could not clean the website folder");
    }
    fs::create_dir(WEBSITE_ROOT).expect("Failed to create website folder");
}

fn create_website_folders() {
    let website_root_path = Path::new(WEBSITE_ROOT);
    if !website_root_path.exists() {
        fs::create_dir_all(WEBSITE_ROOT).expect("Failed to create website root folder");
    }
    let mut webserver_python_path = PathBuf::from(website_root_path);
    webserver_python_path.push("webserver.py");
    fs::write(webserver_python_path, WEBSERVER_PYTHON_SRC).expect("Failed to write webserver python file");

    let website_resources_path = [WEBSITE_ROOT, WEBSITE_RESOURCES_FOLDER_NAME].into_iter().collect::<PathBuf>();
    if !website_resources_path.exists() {
        fs::create_dir_all(&website_resources_path).expect("Failed to create website resources folder");
    }
    {
        let mut resource_stylescss_path = website_resources_path.clone();
        resource_stylescss_path.push("styles.css");
        fs::write(resource_stylescss_path, STYLES_CSS_SRC).expect("Failed to write styles.css");

        let mut resource_galleryjs_path = website_resources_path.clone();
        resource_galleryjs_path.push("gallery.js");
        fs::write(resource_galleryjs_path, GALLERY_JS_SRC).expect("Unable to write file");
    }
}

pub fn build_website(page_info: PageInfo) {
    create_website_folders();
    {
        println!("Downloading thumbnails...");
        THUMBNAIL_DOWNLOAD_MANAGER.lock().unwrap().download_all();
        println!("Done downloading thumbnails");
    }
    println!("Building website...");

    let built_html = HANDLEBARS.render("html_template", &page_info).unwrap();

    let mut subsite_path = [WEBSITE_ROOT, &page_info.page_build_info.guild_built_from, &page_info.page_build_info.channel_built_from].into_iter().collect::<PathBuf>();
    fs::create_dir_all(&subsite_path).expect("Failed to make subsite folder");
    subsite_path.push("index.html");
    fs::write(subsite_path, built_html).expect("Unable to subsite index file");


    println!("Website built!")
}