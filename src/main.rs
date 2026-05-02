#[derive(Debug)]
struct Clip{
    name: String,
    path: std::path::PathBuf,
    date_made: std::time::SystemTime,
    length: u32,
    
}

fn read_clips(clip_path: &str)-> Vec<Clip>{
    let mut clips: Vec<Clip> = Vec::new();
    //reading 1 file
    for entry in std::fs::read_dir(clip_path).expect("Failed to read dir"){
        let clip = entry.expect("Failed to get file");
        let name = clip.file_name().to_string_lossy().to_string();
        let path = clip.path();
        let metadata = clip.metadata().expect("Failed to get metadata");
        let date_made = metadata.created().expect("Failed to get modified time");
        println!("name {}, path {:?}, created {:?}", &name, &path, &date_made);
        clips.push(Clip { name, path, date_made, length: 90});
    }
    clips
}

fn main() {
    let clips = read_clips("D:/AMD clipps/Until Then");
    println!("{:?}", &clips);
}
