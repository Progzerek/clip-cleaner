use std::io;
use walkdir::WalkDir;
use rayon::prelude::*;

struct Clip{
    name: String,
    path: std::path::PathBuf,
    size: u64,
    date_modified: std::time::SystemTime,
    length: f64,
    
}

fn read_clips(clip_path: &str) -> Vec<Clip> {
    
    let paths: Vec<_> = WalkDir::new(clip_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("mp4"))
        .collect();

    paths.par_iter()
        .filter_map(|clip| {
            let path = clip.path().to_path_buf();
            let name = clip.file_name().to_string_lossy().to_string();
            let metadata = clip.metadata().ok()?;
            let size = metadata.len();
            let date_modified = metadata.modified().ok()?;
            let length = ffprobe::ffprobe(&path)
                .ok()
                .and_then(|info| info.streams.into_iter().find(|s| s.duration.is_some()))
                .and_then(|s| s.duration)
                .and_then(|d| d.parse::<f64>().ok())
                .unwrap_or(0.0);
            Some(Clip { name, path, size, date_modified, length })
        })
        .collect()
}
fn get_size(clips: &Vec<Clip>)-> f64{
    let total_size: u64 = clips.iter().map(|c| c.size).sum();
    (total_size) as f64 / 1024.0 / 1024.0
}

fn delete_clips(clips: &mut Vec<Clip>, target: f64){
    let mut i = 0;
    while i < clips.len() {
        let size = get_size(clips);
        if size > target {
            if clips[i].length < (120) as f64{
                i += 1;
                continue;
            }
            std::fs::remove_file(&clips[i].path).expect("Failed to delete");
            println!("Deleted file: {}", &clips[i].name);
            clips.remove(i);
        } else {
            break;
        }
}
}

fn main() {
    println!("||Clip space manager||");

    let target_path = loop{
        let mut ask_target_path=String::new();

        println!("Folder to delete from: ");
        
        io::stdin().read_line(&mut ask_target_path).expect("Failed to read line");
        let target_path = ask_target_path.trim().to_string();
        if std::path::Path::new(&target_path).exists() {
            break target_path;
        }
        println!("Invalid path");
    };
    
    let target_size_gb = loop{
        println!("Target size (GB):");
        let mut target_size = String::new();
        io::stdin().read_line(&mut target_size).expect("Failed to read line");
        match target_size.trim().parse::<f64>(){
            Ok(value)=>break value,
            Err(_)=>{
                println!("Not a valid number");
                continue;
            }
        };
    };
    
    
    println!("Your setting: \n - Target folder: {}\n - Target size: {}GB", &target_path, &target_size_gb);
    println!("Are you sure you want to continue? (y/n)");
    let mut safety_check = String::new();
    io::stdin().read_line(&mut safety_check).expect("Failed to read line");
    if safety_check.trim() == "y"{
        let mut clips = read_clips(&target_path);
        clips.sort_by(|a, b| a.date_modified.cmp(&b.date_modified));
        delete_clips(&mut clips, target_size_gb*1024.0);
        let current_size = (get_size(&clips))/1024.0;
        println!("\nRemaining space: {:.1}", current_size);
    }
    else {
        println!("Failed confirmaion");
    }
}