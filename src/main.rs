
use std::io::{copy, Write};
use std::io::Read;
use std::fs::File;
use std::collections::HashMap;
use chrono::{Datelike, Utc};
use tempfile::Builder;
use walkdir::WalkDir;

#[tokio::main]
async fn main() {
    
    let url =  get_picture_of_the_day_url().await.unwrap();
  
    let file = download_file(url,"data/pictures").await;

    match file {
        Ok(_) => {
           println!("File downloaded successfully");
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

}
fn get_date()-> String {
    let today = Utc::today();
    let year = today.year();
    let month = today.month();
    let day = today.day();
    let date = format!("{}-{}-{}", year, month, day);
    date
}

async fn get_picture_of_the_day_url()->Option<String> {
    let endpoint = "https://bing.biturl.top";
    let resp = reqwest::get(endpoint).await.unwrap();
    let body = resp.json::<HashMap<String, String>>().await.unwrap();
    
    match body.get("url") {
        Some(url) => {
            println!("{}", url);
            Some(url.to_string())
        },
        None => None
    }
}

async fn download_file(target: String,destination:&str)->Result<File,reqwest::Error> {
    let response = reqwest::get(target).await?;
    if response.status().is_success() {
        let file_name = get_date();
        let ext = response.headers().get("content-type").unwrap().to_str().unwrap();
        let ext = ext.split("/").last().unwrap();
        let mut file = File::create(format!("{}/{}.{}",destination,file_name,ext)).unwrap();
        let mut bytes = response.bytes().await?;
        file.write_all(&mut bytes);
        Ok(file)
    } else {
        panic!("Error downloading file: {}", response.status());
    }
}

fn is_not_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
         .file_name()
         .to_str()
         .map(|s| entry.depth() == 0 || !s.starts_with("."))
         .unwrap_or(false)
}

fn move_older_to_archive(){
    
    // move any content inside the today folder to the archive folder.
    let archive_path = "data/archive/".to_string();
    //read the directory of the today folder.
    let today_folder = "data/pictures/";

    WalkDir::new(today_folder)
        .into_iter()
        .filter_entry(|e| is_not_hidden(e))
        .filter_map(|v| v.ok())
        .for_each(|x| {
            // rename the file to the archive folder.
            let mut new_path = archive_path.clone();
            new_path.push_str(x.file_name().to_str().unwrap());
            std::fs::rename(x.path(), new_path).unwrap();
        } );

}