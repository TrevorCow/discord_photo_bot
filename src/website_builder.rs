use std::{fs, thread};
use std::path::Path;
use std::sync::{Arc, Mutex};
use futures::executor::block_on;
use futures::future::join_all;
use handlebars::Handlebars;
use image::ImageFormat;
use image::imageops::FilterType;
use once_cell::sync::Lazy;
use serde::Serialize;
use tokio::join;
use tokio::runtime::{Handle, Runtime};
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
    pub(crate) page_build_info: Box<str>,
    pub(crate) galleries: Vec<GalleryInfo>,
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
                            let bytes = response.bytes().await.unwrap();
                            let image = image::load_from_memory(&bytes).unwrap();
                            // image.thumbnail(200, 200)
                            image.resize(250, 250, FilterType::Triangle)
                        };


                        match thumbnail_image.save_with_format(&save_as, ImageFormat::Jpeg) {
                            Ok(_) => {
                                println!("Successfully saved thumbnail `{}`", save_as)
                            }
                            Err(err) => {
                                eprintln!("Error saving thumbnail `{}`: {}", save_as, err)
                            }
                        }
                    })
                );
            }

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
        fs::create_dir(*THUMBNAIL_ABSOLUTE_FOLDER).expect("Unable to create thumbnails folder!");
    }

    let url = Url::parse(url_str).unwrap();

    // let file_name = Path::new(&url.path()).file_name().unwrap().to_owned().into_string().unwrap(); // This might have file name collisions
    let file_name = url.path().to_owned().replace('/', "_");
    let save_as = format!("{}{}", THUMBNAIL_ABSOLUTE_FOLDER.to_str().unwrap(), file_name);

    let return_path = Box::from(save_as.strip_prefix(WEBSITE_ROOT).unwrap());

    println!("Trying to save thumbnail: {:?}", return_path);
    {
        THUMBNAIL_DOWNLOAD_MANAGER.lock().unwrap().add_to_queue((url, save_as));
    }
    // thread::spawn(move || {
    //     let response = reqwest::blocking::get(url.clone()).unwrap();
    //     let bytes = response.bytes().unwrap();
    //     let image = image::load_from_memory(&bytes).unwrap();
    //
    //     let thumbnail_image = image.thumbnail(200, 200);
    //
    //     match thumbnail_image.save(&save_as) {
    //         Ok(_) => {
    //             println!("Successfully saved thumbnail `{}`", file_name)
    //         }
    //         Err(err) => {
    //             eprintln!("Error saving thumbnail `{}`: {}", save_as, err)
    //         }
    //     }
    // });

    return_path
}


pub fn clean_website_folder() {
    if Path::new(WEBSITE_ROOT).exists() {
        fs::remove_dir_all(WEBSITE_ROOT).expect("Could not clean the website folder");
    }
    fs::create_dir(WEBSITE_ROOT).expect("Failed to create website folder");
}

pub fn build_website(page_info: PageInfo) {
    {
        println!("Downloading thumbnails...");
        THUMBNAIL_DOWNLOAD_MANAGER.lock().unwrap().download_all();
        println!("Done downloading thumbnails");
    }
    println!("Building website...");
    let built_html = HANDLEBARS.render("html_template", &page_info).unwrap();

    fs::write(WEBSITE_ROOT.to_owned() + "styles.css", STYLES_CSS).expect("Unable to write file");
    fs::write(WEBSITE_ROOT.to_owned() + "gallery.js", GALLERY_JS).expect("Unable to write file");
    fs::write(WEBSITE_ROOT.to_owned() + "index.html", &built_html).expect("Unable to write file");
    println!("Website built!")
}