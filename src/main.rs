use std::io::{Write};
use std::fs::File;
use std::collections::HashMap;
use std::{fs, io};
use std::path::Path;
use chrono::{Datelike, Utc};
use clokwerk::{AsyncScheduler, Job, TimeUnits};
use walkdir::{DirEntry, WalkDir};

#[tokio::main]
async fn main() {
    picture_of_the_day_to_data_folder().await;
    schedule_downloads().await;
}

async fn picture_of_the_day_to_data_folder(){
    picture_of_the_day("data/archive/","data/pictures").await;
}

 async fn schedule_downloads() {
    //use clockwork to run the file download every day at 9:00 am
    let mut scheduler = AsyncScheduler::new();
    scheduler.every(1.day()).at("09:00").run(picture_of_the_day_to_data_folder);


     //keep the main thread alive by looping forever
     //put the thread to sleep for 10 minutes
     loop {
         scheduler.run_pending().await;
         std::thread::sleep(std::time::Duration::from_secs(600));
     }
}

async fn picture_of_the_day(archive_dir: &str, download_dir: &str) {
    let url =  get_picture_of_the_day_url() .await.expect("Could not get picture of the day url");
    move_older_to_archive(download_dir,archive_dir).expect("Could not move older files to archive");
    download_file(url,download_dir).await.expect("Could not download file");
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
    let resp = reqwest::get(endpoint).await.unwrap();
    let body = resp.json::<HashMap<String, String>>().await.unwrap();

   let url =  body.get("url");
   match url {
       Some(url) => Ok(url.to_string()),
       None => Err("No url found".to_string())
   }
}

async fn download_file(target: String,destination:&str)->Result<File,reqwest::Error> {
    let response = reqwest::get(target).await?;
    let file_name = get_date();
    let ext = response.headers().get("content-type").unwrap().to_str().unwrap();
    let ext = ext.split("/").last().unwrap();
    let mut file = File::create(format!("{}/{}.{}",destination,file_name,ext))
        .expect(format!("Could not create file in destination {}",destination).as_str());
    let mut bytes = response.bytes().await?;
    file.write_all(&mut bytes).expect("Could not write file the downloaded file");
    Ok(file)
}

fn is_not_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
         .file_name()
         .to_str()
         .map(|s| entry.depth() == 0 || !s.starts_with("."))
         .unwrap_or(false)
}

fn create_path_if_not_exists(path:&str){
    if !Path::new(path).exists() {
        fs::create_dir_all(path).unwrap();
    }
}

fn loop_over_dir<F:FnMut(walkdir::DirEntry)>(dir:&str, callback:F){
    WalkDir::new(dir).into_iter()
        .filter_entry(|e| is_not_hidden(e))
        .filter_map(|v| v.ok())
        .for_each(callback);

}

fn move_older_to_archive(from:&str, to:&str)->Result<(),io::Error>{

    create_path_if_not_exists(from);
    create_path_if_not_exists(to);

   let  move_to_archive = |entry:DirEntry|{
        if !entry.metadata().unwrap().is_dir() {
            let mut new_path = to.to_string();
            new_path.push_str(entry.file_name().to_str().unwrap());
            std::fs::rename(entry.path(), new_path).unwrap();
        }
    };
    loop_over_dir(from, &move_to_archive);
    Ok(())
}

pub mod test {
    use std::fs::File;
    use std::path::Path;
    use crate::{create_path_if_not_exists, move_older_to_archive};
    use walkdir::{WalkDir};

    #[test]
    fn _older_to_archive(){
       //create 2 files in a temp dir
       let temp_dir = "data/temp/";
       let archive_dir = "data/temp_archive/";

        create_path_if_not_exists(temp_dir);
        create_path_if_not_exists(archive_dir);

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
        crate::create_path_if_not_exists(path);
        assert!(Path::new(path).exists());
        //clean up
        std::fs::remove_dir_all(path).unwrap();
        assert!(!Path::new(path).exists());
    }

    #[test]
    fn _is_not_hidden(){
        let entry = WalkDir::new(".").into_iter().next().unwrap().unwrap();
        assert!(crate::is_not_hidden(&entry));
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