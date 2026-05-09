use std::io;
use walkdir::WalkDir;
use rayon::prelude::*;
use std::time::Instant;

use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use ratatui::prelude::*;
use ratatui::{DefaultTerminal, Frame};
use ratatui::widgets::*;
use ratatui::layout::Flex;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
//
// Cleaner logic
//
#[derive(Debug)]

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

fn calculate_deleted(clip_path: &str, target: f64)-> Vec<Clip>{
    let lengh_treshold = 120.0;
    let mut to_delete_clips = read_clips(clip_path);
    to_delete_clips.sort_by(|a, b| a.date_modified.cmp(&b.date_modified));
    let mut current_size = get_size(&to_delete_clips);
    to_delete_clips.retain(|clip| {
        if current_size > target{
            if clip.length < lengh_treshold{false}
            else {
                current_size -= ((clip.size) as f64)/1024.0/1024.0;
                true
            }
        }
        else {
            false
        }
    });
    to_delete_clips

}

fn delete_clips(clips: &mut Vec<Clip>){
    for file in clips{
        match std::fs::remove_file(&file.path) {
            Ok(_)=>continue,
            Err(_)=>continue
        }
        
    }
}

/* 
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
        let start = Instant::now();
        let to_delete = calculate_deleted(&target_path, target_size_gb*1024.0);
        delete_clips(to_delete);

        let current_size = (get_size(&read_clips(&target_path)))/1024.0;
        println!("\nRemaining space: {:.1}GB", current_size);
        let duration = start.elapsed();
        println!("Time elapsed: {:?}", duration);
    }
    else {
        println!("Failed confirmaion");
    }
}*/


//
// TUI
//

enum Screen {
    AskPath,
    AskSize,
    MainSrc,
    ProcessDelete,
    Quit
}

struct App{
    items: Vec<Clip>,
    list_state: ListState,

    to_delete: Vec<Clip>,
    to_delete_state: ListState,

    path_input: Input,
    path: String,

    current_size: Option<f64>,
    target_size: Option<f64>,
    target_size_input: Input,
    
    state: Screen,
    
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut clip_app=App{
        items: Vec::new(),
        to_delete: Vec::new(),
        to_delete_state: ListState::default(),
        path_input: Input::default(),
        path: String::new(),
        current_size: None,
        target_size_input: Input::default(),
        target_size: None,
        list_state: ListState::default(),
        state: Screen::AskPath,
        
        
    };
    loop {
        terminal.draw(|frame| {render(frame, &mut clip_app);})?;

        if let Event::Key(key) = event::read()? && key.kind == KeyEventKind::Press{
            match clip_app.state {
                Screen::AskPath=>{
                    match key.code {
                        KeyCode::Enter => {
                            if std::path::Path::new(&clip_app.path).exists() {
                                clip_app.items = read_clips(&clip_app.path);
                                clip_app.current_size = Some(get_size(&clip_app.items));
                                clip_app.state = Screen::MainSrc;
                            }
                        }
                        _ => {
                            clip_app.path_input.handle_event(&Event::Key(key));
                            clip_app.path = clip_app.path_input.value().to_string();
                        }
                    }
                }
                Screen::MainSrc=> match key.code {
                    KeyCode::Down => {
                        let current = match clip_app.list_state.selected(){
                            None=> 0,
                            Some(value)=>value
                        };
                        let last_index = clip_app.items.len();
                        let new = (current+1).min(last_index-1);
                        clip_app.list_state.select(Some(new))
                    }
                    KeyCode::Up   => {
                        let current = match clip_app.list_state.selected(){
                            None=> 0,
                            Some(value)=>value
                        };
                        let new = current.saturating_sub(1);
                        clip_app.list_state.select(Some(new))
                    }
                    KeyCode::F(1)=>clip_app.state=Screen::AskSize,
                    KeyCode::F(10)=>{
                        if clip_app.target_size.is_none(){
                            clip_app.state = Screen::AskSize;
                        }
                        else{
                            clip_app.to_delete = calculate_deleted(&clip_app.path, clip_app.target_size.unwrap());
                            clip_app.state=Screen::ProcessDelete
                        }
                    },
                    KeyCode::Char('q') => clip_app.state=Screen::Quit,
                    _ => {}
                }
                Screen::AskSize=>{
                    match key.code {
                        KeyCode::Enter => {
                            if clip_app.target_size.is_some(){
                                clip_app.state = Screen::MainSrc;
                            }
                            }
                        _ => {
                            clip_app.target_size_input.handle_event(&Event::Key(key));
                            clip_app.target_size = clip_app.target_size_input.value().parse::<f64>().ok();
                        }
                    }
                }
                Screen::ProcessDelete=>{
                    match key.code {
                        KeyCode::Down => {
                            let i = match clip_app.to_delete_state.selected() {
                                None => 0,
                                Some(v) => (v + 1).min(clip_app.to_delete.len().saturating_sub(1)),
                            };
                            clip_app.to_delete_state.select(Some(i));
                        }
                        KeyCode::Up => {
                            let i = match clip_app.to_delete_state.selected() {
                                None => 0,
                                Some(v) => v.saturating_sub(1),
                            };
                            clip_app.to_delete_state.select(Some(i));
                        }
                        KeyCode::Char('y') => {delete_clips(&mut clip_app.to_delete)},    // handle delete better pls
                        KeyCode::Char('n') => clip_app.state = Screen::MainSrc,
                        _ => {}
                    }
                }
                Screen::Quit=>{
                    match key.code {
                        KeyCode::Char('y')=> break Ok(()),
                        KeyCode::Char('n')=> clip_app.state=Screen::MainSrc,
                        _ => {} // ignore everything else
                    }
                }
            }
        }
    }
}
fn render(frame: &mut Frame, app: &mut App) {
    match app.state {
        Screen::AskPath => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),  // label
                    Constraint::Length(1),  // input
                ])
                .split(frame.area());

            let label = Line::from("Path to delete from:");
            frame.render_widget(label.centered(), chunks[0]);

            frame.set_cursor_position((
                chunks[1].x + app.path_input.visual_cursor() as u16,
                chunks[1].y,
            ));

            let input_widget = Paragraph::new(&*app.path);
            frame.render_widget(input_widget, chunks[1]);
        }
        Screen::MainSrc=>{
            let highlight = Style::default()
                .bg(Color::Blue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD);
            let items: Vec<ListItem> = app.items
                .iter()
                .map(|s| {
                    let row = format!(
                        "{:<50} {:>8.1} MB  {:>6.0}s",
                        s.name,
                        s.size as f64 / 1024.0 / 1024.0,
                        s.length
                    );
                    ListItem::new(row)
                })
                .collect();
            let list = List::new(items)
                .highlight_style(highlight);

            let [header, body, footer] = Layout::vertical([
                Constraint::Length(1
                ),
                Constraint::Fill(1),
                Constraint::Length(1),
            ]).areas(frame.area());

            //header render
            let [name_label, current_path] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]).areas(header);
            frame.render_widget(Paragraph::new("Clip cleaner"), name_label);
            frame.render_widget(Paragraph::new(format!("Current path: {}", app.path)).right_aligned(), current_path);
            //body render
            frame.render_stateful_widget(list, body, &mut app.list_state);
            //feet render
            let [add_target, delete, current_size] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]).flex(Flex::SpaceBetween).areas(footer);
            
            frame.render_widget(Paragraph::new(format!("F1 - add target: {}GB", match app.target_size{Some(value)=>value.to_string(), None=>String::from("")})).centered(), add_target);
            frame.render_widget(Paragraph::new("F10 - procede delete").centered(), delete);

            frame.render_widget(Paragraph::new(format!("Current size: {:.1}GB", &app.current_size.unwrap()/1024.0)).centered(), current_size);
        }
        Screen::Quit=>{
            let quit_text = Line::from("Are you sure you want to quit? (y/n)");
            frame.render_widget(quit_text.centered(), frame.area());
        }
        Screen::AskSize=>{
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),  // label
                    Constraint::Length(1),  // input
                ])
                .split(frame.area());

            let label = Line::from("Target size to delete (in GB): ");
            frame.render_widget(label, chunks[0]);

            frame.set_cursor_position((
                chunks[1].x + app.target_size_input.visual_cursor() as u16,
                chunks[1].y,
            ));
            let input_widget = Paragraph::new(format!("{}", app.target_size_input.value()));
            frame.render_widget(input_widget, chunks[1]);
        }
        Screen::ProcessDelete=>{
            let [overview, delete_listing] = Layout::vertical([
                Constraint::Length(7),
                Constraint::Fill(1),
            ]).areas(frame.area());

            frame.render_widget(Paragraph::new(format!("Delete overview
        - Current Path: {}
        - Current Size: {:.1} GB
        - Target size: {:.1} GB\n
    WARNING! THIS ACTION IS IRREVERSABLE! MAKE SURE YOU HAVE THE RIGHT SETTINGS BEFORE YOU CONTINUE!
    Are you sure you want continue? (y/n)", app.path, app.current_size.unwrap()/1024.0, app.target_size.unwrap())), overview);

            let highlight = Style::default()
                .bg(Color::Blue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD);
            let items: Vec<ListItem> = app.to_delete
                .iter()
                .map(|s| {
                    let row = format!(
                        "{:<50} {:>8.1} MB  {:>6.0}s",
                        s.name,
                        s.size as f64 / 1024.0 / 1024.0,
                        s.length
                    );
                    ListItem::new(row)
                })
                .collect();
            let list = List::new(items)
                .highlight_style(highlight);


            frame.render_stateful_widget(list, delete_listing, &mut app.to_delete_state);

        }
    }
    
}