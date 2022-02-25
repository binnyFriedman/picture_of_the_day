use std::io::{Write};
use std::fs::File;
use std::collections::HashMap;
use std::{fs};
use std::path::Path;
use std::env;
use chrono::{Datelike, Utc};
use walkdir::{DirEntry, WalkDir};

#[tokio::main]
async fn main() {
    // get the path to the directory
    let pictures_path = env::args().nth(1).expect("Please provide a path to a directory");
    let archive_path = env::args().nth(2).expect("Please provide a path to an archive");

    picture_of_the_day(&archive_path,&pictures_path).await;
    println!("Downloaded picture of the day {}", Utc::today().format("%Y-%m-%d"));
}



async fn picture_of_the_day(archive_dir: &str, download_dir: &str) {
    let url =  get_picture_of_the_day_url().await.expect("Could not get picture of the day url");
    move_older_to_archive(download_dir,archive_dir).expect("Could not move older files to archive");
    download_file(&url,download_dir).await.expect("Could not download file");
}

fn get_date()-> String {
    let today = Utc::today();
    let year = today.year();
    let month = today.month();
    let day = today.day();
    let date = format!("{}-{}-{}", year, month, day);
    date
}

async fn get_picture_of_the_day_url()->Result<String,String> {
    let endpoint = "https://bing.biturl.top";
    let resp = reqwest::get(endpoint).await;
    if resp.is_err() {
        return Err(format!("Could not get response from {}", endpoint));
    }
    let resp = resp.unwrap();
    let body = resp.json::<HashMap<String, String>>().await;
    if body.is_err() {
        return Err(format!("Could not get json from {}", endpoint));
    }
    let body = body.unwrap();

   let url =  body.get("url");
   match url {
       Some(url) => Ok(url.to_string()),
       None => Err("No url found".to_string())
   }
}

fn get_file_extension(response: &reqwest::Response)-> Result<String, String> {
    let header = response.headers().get("content-type");
    if header.is_none() {
        return Err("No content-type header found".to_string());
    }
    let header = header.unwrap();

    let ext = header.to_str();
    let ext = match ext {
        Ok(ext) => ext,
        Err(_) => return Err("Could not get file extension".to_string())
    };
    let ext = ext.split("/").collect::<Vec<&str>>();
    let ext = ext.last();
    if ext.is_none() {
        return Err("Could not get file extension".to_string());
    }
    let ext = ext.unwrap();
 
    Ok(ext.to_string())
}

async fn download_file(target: &str,destination:&str)->Result<File,String> {
    let response = reqwest::get(target).await;
    if response.is_err() {
        return Err(format!("Could not get response from {}", target));
    }
    let response = response.unwrap();
    let file_name = get_date();
    let ext = match get_file_extension(&response){
        Ok(ext) => ext,
        Err(_) => "txt".to_string()
    };
    
    let path = Path::new(destination).join(format!("{}.{}", file_name, ext));
    let mut file = File::create(path).expect(format!("Could not create file in destination {}",destination).as_str());
    let bytes = response.bytes().await;
    if bytes.is_err() {
        return Err(format!("Could not get bytes from {}", target));
    }
    let mut bytes = bytes.unwrap();
    let result = file.write_all(&mut bytes);
    match result {
        Ok(_) => Ok(file),
        Err(e) => Err(format!("Could not write to file {}", e))
    }
}

fn is_not_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
         .file_name()
         .to_str()
         .map(|s| entry.depth() == 0 || !s.starts_with("."))
         .unwrap_or(false)
}

fn create_path_if_not_exists(path:&str)->Result<(),String> {
    if !Path::new(path).exists() {
       if fs::create_dir_all(path).is_err() {
             return Err("Could not create path".to_string());
       }
    }

    Ok(())
}

fn loop_over_dir<F:FnMut(walkdir::DirEntry)>(dir:&str, callback:F){
    WalkDir::new(dir).into_iter()
        .filter_entry(|e| is_not_hidden(e))
        .filter_map(|v| v.ok())
        .for_each(callback);

}

fn move_older_to_archive(from:&str, to:&str)->Result<(),String>{

    create_path_if_not_exists(from)?;
    create_path_if_not_exists(to)?;

   let  move_to_archive = |entry:DirEntry|{
        if !entry.metadata().unwrap().is_dir() {
            let new_path = Path::new(to).join(entry.file_name());
            std::fs::rename(entry.path(), new_path).unwrap();
        }
    };
    loop_over_dir(from, &move_to_archive);
    Ok(())
}

pub mod test {
    use super::*;
    use std::path::Path;
    use std::fs::{ File};
    #[test]
    fn _older_to_archive(){
       //create 2 files in a temp dir
       let temp_dir = "data/temp/";
       let archive_dir = "data/temp_archive/";

        create_path_if_not_exists(temp_dir).unwrap();
        create_path_if_not_exists(archive_dir).unwrap();

        let file1_temp = format!("{}/file1.txt",temp_dir);
        let file2_temp = format!("{}/file2.txt",temp_dir);
        //create 2 files in a temp dir
        File::create(file1_temp.as_str()).unwrap();
        File::create(file2_temp.as_str()).unwrap();

        move_older_to_archive(temp_dir,archive_dir).unwrap();

        let file1_archive = format!("{}/file1.txt",archive_dir);
        let file2_archive = format!("{}/file2.txt",archive_dir);
        //check if the files are in the archive dir
        //try opening the files in the temp dir
        let file_1_path = Path::new(file1_archive.as_str());
        let file_2_path = Path::new(file2_archive.as_str());
        assert!(file_1_path.exists());
        assert!(file_2_path.exists());

        //check if the files are in the temp dir
        let file_1_path = Path::new(file1_temp.as_str());
        let file_2_path = Path::new(file2_temp.as_str());
        assert!(!file_1_path.exists());
        assert!(!file_2_path.exists());

        //cleanup
        std::fs::remove_dir_all(temp_dir).unwrap();
        std::fs::remove_dir_all(archive_dir).unwrap();
    }

    #[test]
    fn create_if_not_exists(){
        let path = "temp/archive/";
        create_path_if_not_exists(path).unwrap();
        assert!(Path::new(path).exists());
        //clean up
        std::fs::remove_dir_all(path).unwrap();
        assert!(!Path::new(path).exists());
    }

    #[test]
    fn _is_not_hidden(){
        let entry = WalkDir::new(".").into_iter().next().unwrap().unwrap();
        assert!(is_not_hidden(&entry));
    }

    #[test]
    fn _loop_over_dir(){
        let mut count = 0;
        let callback =  |_|{
            count += 1;
        };
        crate::loop_over_dir(".", callback);
        assert!(count > 0);
    }

}