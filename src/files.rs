// use rocket::fs::TempFile;
// use rocket::form::Form;
// // use rocket::http::ContentType;
// use rocket::response::Redirect;
// use rocket::fs::NamedFile;
// use std::path::Path;
// use rocket::fs;
// use std::fs::read_dir;
//
// use crate::schema::files;
//
//
// #[derive(FromForm)]
// struct FileUpload<'r> {
//     file: TempFile<'r>,
// }
//
// // Implement a file_list handler to list all the files in the "uploads" directory.
// #[get("/files")]
// async fn file_list() -> String {
//     let mut files = Vec::new();
//     if let Ok(mut dir_entries) = read_dir("uploads/") {
//         while let Some(Ok(entry)) = dir_entries.next() {
//             if let Ok(file_name) = entry.file_name().into_string() {
//                 files.push(file_name);
//             }
//         }
//     }
//     let mut html = String::from("<h1>Uploaded Files</h1><ul>");
//     for file in files {
//         html.push_str(&format!(
//             "<li><a href='/uploads/{}'>{}</a></li>",
//             file, file
//         ));
//     }
//     html.push_str("</ul>");
//     html
// }
//
// #[post("/upload", data = "<file_upload>")]
// async fn upload(mut file_upload: Form<FileUpload<'_>>) -> Redirect {
//     let file = &mut file_upload.file;
//     file.persist_to("uploads/").await.unwrap();
//     Redirect::to(uri!(file_list))
// }
//
// #[get("/uploads/<file_name>")]
// async fn get_file(file_name: String) -> Option<NamedFile> {
//     NamedFile::open(Path::new("uploads/").join(file_name)).await.ok()
// }
