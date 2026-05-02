use std::io;

use walkdir::WalkDir;
//#[derive(Debug)]
struct Clip{
    name: String,
    path: std::path::PathBuf,
    size: u64,
    date_modified: std::time::SystemTime,
    length: f64,
    
}

fn read_clips(clip_path: &str)-> Vec<Clip>{
    let mut clips: Vec<Clip> = Vec::new();
    //reading 1 file
    for entry in WalkDir::new(clip_path){
        let clip = entry.expect("Failed to get file");
        let name = clip.file_name().to_string_lossy().to_string();
        let path = clip.path();
        if path.extension().and_then(|e| e.to_str()) != Some("mp4") {
            continue;
        }
        let length = ffprobe::ffprobe(&path)
            .ok()
            .and_then(|info| info.streams.into_iter().find(|s| s.duration.is_some()))
            .and_then(|s| s.duration)
            .and_then(|d| d.parse::<f64>().ok())
            .unwrap_or(0.0);
        let metadata = clip.metadata().expect("Failed to get metadata");
        let size = metadata.len();
        let date_modified = metadata.modified().expect("Failed to get modified time");
        clips.push(Clip { name, path: path.to_path_buf(), size, date_modified, length});
    }
    clips
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
    //let mut clips = read_clips("D:/unknown");
    //println!("{:?}", &clips);
    //clips.sort_by(|a, b| a.date_modified.cmp(&b.date_modified));
    //get_size(&clips);
    //delete_clips(&mut clips, (3000) as f64);


    println!("||Clip space manager||");

    println!("Folder to delete from: ");
    let mut target_path = String::new();
    io::stdin().read_line(&mut target_path).expect("Failed to read line");
    let target_path = target_path.trim();

    println!("Target size (GB):");
    let mut target_size = String::new();
    io::stdin().read_line(&mut target_size).expect("Failed to read line");
    let target_size_gb: f64 = target_size.trim().parse().expect("Failed to parse");

    println!("Your setting: \n - Target folder: {}\n - Target size: {}GB", &target_path, &target_size_gb);
    println!("Are you sure you want to continue? (y/n)");
    let mut safety_check = String::new();
    io::stdin().read_line(&mut safety_check).expect("Failed to read line");
    if safety_check.trim() == "y"{
        let mut clips = read_clips(&target_path);
        clips.sort_by(|a, b| a.date_modified.cmp(&b.date_modified));
        //get_size(&clips);
        delete_clips(&mut clips, target_size_gb*1024.0);
        let current_size = (get_size(&read_clips(target_path)))/1024.0;
        println!("\nRemaining space: {:.1}", current_size);
    }
    else {
        println!("Failed confirmaion");
    }
}
