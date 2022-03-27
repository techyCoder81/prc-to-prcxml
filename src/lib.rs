#![feature(proc_macro_hygiene)]
#![feature(allocator_api)]
use skyline::{hook, install_hook};
use prcx::*;
use walkdir::WalkDir;
use std::fs::File;
use std::path::Path;
use std::io::{BufWriter, Write};
use std::thread;
use std::time::Duration;
use skyline_web::*;
use skyline::libc::c_char;

#[global_allocator]
static SMASH_ALLOCATOR: skyline::unix_alloc::UnixAllocator = skyline::unix_alloc::UnixAllocator;

extern "C" {
  fn change_version_string(arg: u64, string: *const c_char);
}

#[skyline::hook(replace = change_version_string)]
fn change_version_string_hook(arg: u64, string: *const c_char) {
  let original_str = unsafe { skyline::from_c_str(string) };
  if original_str.contains("Ver.") {
      spawn_thread();
      call_original!(arg, string)
  } else {
      call_original!(arg, string)
  }
}

fn diff_prc_files() {
  println!("starting diff of files");
  let mut paths = vec![]; 
  let mut deletions = vec![];
  let mut failures = vec![];
  println!("made paths vec");
  let dir = walkdir::WalkDir::new("sd:/ultimate/mods/");
  println!("made walkable dir");
  for entry in dir {
    if entry.is_err() {
      println!("error!");
      continue;
    }
    let path_entry = entry.expect("no entry found during walk!");
    match path_entry.path().extension() {
      Some(extension) => {
        let path_name = path_entry.path().as_os_str().to_str().expect("error while turning path into string!");
        println!("checking path: {}", path_name);
        let path_str = path_name;
        if extension == "prc" {
          println!("adding path: {}", path_name);
          paths.push(format!("{}", path_name));
        } else {
          println!("ignoring path.");
        }
      },
      None => {}
    }
  }
  for path in paths {
    println!("\noriginal path: {}", path);
    let cleaned_path = path.replace("sd:/ultimate/mods/", "");
    println!("cleaned path: {}", cleaned_path);
    let arc_path = format!("arc:{}",    cleaned_path.chars().skip(cleaned_path.find("/").unwrap()).take(cleaned_path.len()).collect::<String>()   );// &cleaned_path.as_str()[cleaned_path.find("/")..]);
    println!("arc path: {}", arc_path);
    

    // load arc_path      
    let arc_file = prcx::open(arc_path.clone());      

    // load mod_path
    let mod_file = prcx::open(path.clone());

    println!("diffing!");
    // diff files

    // ignore modded files that are not actually in the arc
    let arc_params = match arc_file{
      Ok(value) => value,
      Err(e) => continue
    };

    let mod_params = mod_file.unwrap();
    let diff = match prcx::generate_patch(&arc_params, &mod_params) {
      Ok(value) => match value {
        Some(inner_value) => inner_value,
        None => {
          println!("found no values: {}", path);
          failures.push(format!("{}", path));
          continue;
        }
      }
      Err(e) => {
        println!("could not handle file: {}", path);
        failures.push(format!("{}", path));
        continue;
      }
    };


    let output_pathname = format!("{}{}xml", "sd:/xml/", cleaned_path);
    println!("output path: {}", output_pathname);
    let output_path = std::path::Path::new(&output_pathname).parent().unwrap();
    
    std::fs::create_dir_all(output_path).unwrap();
    let f = File::create(output_pathname).expect("Unable to create file");
    let mut writer = BufWriter::new(f);
    prcx::write_xml(&diff, &mut writer);
    writer.flush();
    deletions.push(format!("{}", path));

  }

  
  let should_delete = skyline_web::Dialog::no_yes("would you like to also delete the original .prc files on SD for each generated .prcxml file?");
  if should_delete {
    for path in deletions {
      std::fs::remove_file(path);
    }
  }
  let mut result_str: String = "PRC conversion to XML is complete. Output will be in /xml/ on root of sd. Failed files:".to_owned();
  for path in failures {
    result_str.push_str("\n");
    result_str.push_str(&path);
  }
  skyline_web::DialogOk::ok(result_str);
}


#[skyline::main(name = "prc_to_xml")]
pub fn main() {
  
  skyline::install_hooks!(change_version_string_hook);
}

pub fn log(log_writer: &mut BufWriter<File>, string: &str) {
  println!("{}", string);
  log_writer.write(string.as_bytes());
}

pub fn spawn_thread() {
  thread::spawn(|| {
    let f = File::create("sd:/prc_to_prcxml.log").expect("Unable to create log file");
    let mut log_writer = BufWriter::new(f);
    
    println!("prc_to_xml main!");

    if Path::new("sd:/xml/").exists() {
      let should_delete = skyline_web::Dialog::no_yes("would you like to delete old xml diffs first?");
      if should_delete {
        println!("deleting /xml/");
        std::fs::remove_dir_all("sd:/xml/");
      }
    }

    let param_labels = include_str!("../resources/ParamLabels.csv");
    
    println!("making params file");
    if !Path::new("sd:/ParamLabels.csv").exists() {
      std::fs::write("sd:/ParamLabels.csv", param_labels).expect("Unable to write params file");
    }

    // load the param labels into prcx
    println!("reading the param labels");
    let labels = match prcx::hash40::read_custom_labels("sd:/ParamLabels.csv"){
      Ok(ok_labels) => ok_labels,
      Err(e) => {
        println!("{:?}", e);
        return;
      }
    };
    println!("setting custom labels");
    prcx::hash40::set_custom_labels(labels.into_iter());

    diff_prc_files();

    println!("end prc_to_xml main."); // 3
    log_writer.flush();
  });
}