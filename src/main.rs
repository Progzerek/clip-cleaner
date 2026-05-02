use walkdir::WalkDir;
#[derive(Debug)]
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
    let total_mb: f64 = (total_size) as f64 / 1024.0 / 1024.0;
    let total_gb = (total_size) as f64 / 1024.0 / 1024.0 / 1024.0;
    println!("Total: {:.1} MB", total_mb);
    println!("Total: {:.1} GB", total_gb);
    total_mb
}

fn delete_clips(clips: &mut Vec<Clip>, target: f64){
    let i = 0;
    while i < clips.len() {
        let size = get_size(clips);
        if size > target {
            std::fs::remove_file(&clips[i].path).expect("Failed to delete");
            clips.remove(i);
        } else {
            break;
        }
}
}

fn main() {
    let mut clips = read_clips("D:/unknown");
    //println!("{:?}", &clips);
    clips.sort_by(|a, b| a.date_modified.cmp(&b.date_modified));
    //get_size(&clips);
    delete_clips(&mut clips, (3000) as f64);
}
