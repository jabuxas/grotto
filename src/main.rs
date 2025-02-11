#[macro_use]
extern crate rocket;

use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::tokio::io::AsyncWriteExt;
use rocket::tokio::fs::File;
use rocket::tokio::io::AsyncReadExt;
use rocket::fs::NamedFile;
use rocket::get;
use std::path::{Path, PathBuf};

#[derive(FromForm)]
struct Upload<'r> {
    save: bool,
    file: TempFile<'r>,
}

#[get("/")]
fn index() -> &'static str {
    "<h1>hey, index is working</h1>"
}

#[get("/<file..>")]
async fn file(file: PathBuf) -> Option<NamedFile> {
    let path = Path::new("./store").join(file);

    NamedFile::open(path).await.ok()
}

#[post("/", data = "<upload>")]
async fn upload(upload: Form<Upload<'_>>) -> String {
    std::fs::create_dir_all("./store").expect("Failed to create directory");

    let upload = upload.into_inner();

    let filename: String = upload
        .file
        .name()
        .and_then(|n| std::path::Path::new(n).file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_owned())
        .unwrap_or_else( || "unknown.bin".to_owned());

    let path = std::path::Path::new("./store").join(&filename);

    let mut tmp_file = upload.file.open().await.expect("Failed to open temp file");
    let mut dest_file = File::create(&path)
        .await
        .expect("Failed to create destination file");

    let mut buffer = Vec::new();
    tmp_file
        .read_to_end(&mut buffer)
        .await
        .expect("Failed to read temporary file");
    dest_file
        .write_all(&buffer)
        .await
        .expect("Failed to write destination file");


    format!("http://localhost:8000/{}\n", filename)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, upload, file])
}
