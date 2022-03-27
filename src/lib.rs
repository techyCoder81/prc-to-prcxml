#![feature(proc_macro_hygiene)]
#![feature(allocator_api)]
use std::fs::File;
use std::path::Path;
use std::io::{BufWriter, Write};
use std::thread;
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
      let version_str = format!("{}\nprc_to_xml.nro loaded!", original_str);
      call_original!(arg, skyline::c_str(&version_str))
  } else {
      call_original!(arg, string)
  }
}

fn diff_prc_files(log_writer: &mut BufWriter<File>) {
  log(log_writer, "starting diff of files".to_string());
  let mut paths = vec![]; 
  let mut deletions = vec![];
  let mut failures = vec![];
  log(log_writer, "made paths vec".to_string());
  let dir = walkdir::WalkDir::new("sd:/ultimate/mods/");
  log(log_writer, "made walkable dir".to_string());
  for entry in dir {
    if entry.is_err() {
      log(log_writer, "error!".to_string());
      continue;
    }
    let path_entry = entry.expect("no entry found during walk!");
    match path_entry.path().extension() {
      Some(extension) => {
        let path_name = path_entry.path().as_os_str().to_str().expect("error while turning path into string!");
        log(log_writer, format!("checking path: {}", path_name));
        if extension == "prc" {
          log(log_writer, format!("adding path: {}", path_name));
          paths.push(format!("{}", path_name));
        } else {
          log(log_writer,  "ignoring path.".to_string());
        }
      },
      None => {}
    }
  }
  for path in paths {
    log(log_writer, format!("\noriginal path: {}", path));
    let cleaned_path = path.replace("sd:/ultimate/mods/", "");
    log(log_writer, format!("cleaned path: {}", cleaned_path));
    let arc_path = format!("arc:{}",    cleaned_path.chars().skip(cleaned_path.find("/").unwrap()).take(cleaned_path.len()).collect::<String>()   );// &cleaned_path.as_str()[cleaned_path.find("/")..]);
    log(log_writer, format!("arc path: {}", arc_path));
    

    // load arc_path      
    let arc_file = prcx::open(arc_path.clone());      

    // load mod_path
    let mod_file = prcx::open(path.clone());

    log(log_writer, "diffing!".to_string());
    // diff files

    // ignore modded files that are not actually in the arc
    let arc_params = match arc_file{
      Ok(value) => value,
      Err(_) => {
        log(log_writer, format!("ignoring file which is not in the arc: {}", arc_path));
        continue
      }
    };

    let mod_params = mod_file.unwrap();
    let diff = match prcx::generate_patch(&arc_params, &mod_params) {
      Ok(value) => match value {
        Some(inner_value) => inner_value,
        None => {
          log(log_writer, format!("found no values: {}", path));
          failures.push(format!("{}", path));
          continue;
        }
      }
      Err(e) => {
        log(log_writer, format!("could not handle file: {}\nError:{:?}", path, e));
        failures.push(format!("{}", path));
        continue;
      }
    };


    let output_pathname = format!("{}{}xml", "sd:/xml/", cleaned_path);
    log(log_writer, format!("output path: {}", output_pathname));
    let output_path = std::path::Path::new(&output_pathname).parent().unwrap();
    
    std::fs::create_dir_all(output_path).unwrap();
    let f = File::create(output_pathname.clone()).expect("Unable to create file");
    let mut writer = BufWriter::new(f);
    match prcx::write_xml(&diff, &mut writer) {
      Ok(_) => log(log_writer, format!("wrote prcxml to: {}", output_pathname)),
      Err(e) => log(log_writer, format!("could not write prcxml path: {}\nError: {:?}", path, e))
    }
    writer.flush().expect("could not flush???");
    deletions.push(format!("{}", path));


    log(log_writer, format!("completed diffing and writing file: {}", path.clone()));
  }

  
  let should_delete = skyline_web::Dialog::no_yes("would you like to also delete the original .prc files on SD for each generated .prcxml file?");
  if should_delete {
    log(log_writer, "begin deleting originals".to_string());
    for path in deletions {
      match std::fs::remove_file(path.clone()) {
        Ok(_) => {},
        Err(e) => log(log_writer, format!("could not delete file: {}\nError: {:?}", path.clone(), e))
      }
    }
    log(log_writer, "completed deleting originals".to_string());
  } else {
    log(log_writer, "not deleting originals".to_string());
  }
  

  let should_move = skyline_web::Dialog::no_yes("would you like to move the newly generated xml files into the /ultimate/mods directory structure in place of the originals?");
  if should_move {
    log(log_writer, "begin moving xml over".to_string());
    let options = fs_extra::dir::CopyOptions::new();
    let generated_paths = std::fs::read_dir("sd:/xml/").unwrap();
    for path in generated_paths {
      let dir_entry = path.unwrap();
      let source_path = dir_entry.path();
      let source_str = source_path.as_os_str().to_str().unwrap();
      let target_str = "sd:/ultimate/mods/";
      log(log_writer, format!("moving from {} to {}", source_str, target_str));
      match fs_extra::dir::move_dir(source_str, target_str, &options) {
        Ok(_) => {},
        Err(e) => log(log_writer, format!("could not copy generated files from {} to {}\nError: {:?}", source_str,  target_str, e))
      }
    }
    log(log_writer, "completed moving xml over".to_string());
  } else {
    log(log_writer, "not moving generated xml over".to_string());
  }


  let mut result_str: String = "PRC conversion to XML is complete. Output will be in /xml/ on root of sd. Failed files:".to_owned();
  for path in failures {
    result_str.push_str("\n");
    result_str.push_str(&path);
  }
  skyline_web::DialogOk::ok(result_str.clone());
  log(log_writer, result_str);
}


#[skyline::main(name = "prc_to_xml")]
pub fn main() {
  
  skyline::install_hooks!(change_version_string_hook);
}

/// log the given string using println! and also to the given buffer
pub fn log(log_writer: &mut BufWriter<File>, string: String) {
  println!("{}", string);
  match log_writer.write(format!("{}\n", string).as_bytes()) {
    Ok(_) => {},
    Err(e) => println!("logger failed to write to buffer with string: {}\nError:{:?}", string, e)
  }
}

/// this is functionally the "main" of the application, procced by the hook that gets the version string.
pub fn spawn_thread() {
  thread::spawn(|| {
    let f = File::create("sd:/prc_to_prcxml.log").expect("Unable to create log file");
    let mut log_writer = BufWriter::new(f);
    
    log(&mut log_writer, "prc_to_xml main!".to_string());

    if Path::new("sd:/xml/").exists() {
      let should_delete = skyline_web::Dialog::no_yes("would you like to delete old xml diffs first?");
      if should_delete {
        log(&mut log_writer, "deleting /xml/".to_string());
        match std::fs::remove_dir_all("sd:/xml/") {
          Ok(_) => {},
          Err(e) => {
            log(&mut log_writer, format!("could not delete /xml/ dir!\nError: {:?}\n", e))
          }
        }
      }
    }

    let param_labels = include_str!("../resources/ParamLabels.csv");
    
    log(&mut log_writer, "making params file".to_string());
    if !Path::new("sd:/ParamLabels.csv").exists() {
      std::fs::write("sd:/ParamLabels.csv", param_labels).expect("Unable to write params file");
    }

    // load the param labels into prcx
    log(&mut log_writer, "reading the param labels".to_string());
    let labels = match prcx::hash40::read_custom_labels("sd:/ParamLabels.csv"){
      Ok(ok_labels) => ok_labels,
      Err(e) => {
        log(&mut log_writer, format!("{:?}", e));
        return;
      }
    };
    log(&mut log_writer, "setting custom labels".to_string());
    prcx::hash40::set_custom_labels(labels.into_iter());

    diff_prc_files(&mut log_writer);

    log(&mut log_writer, "end prc_to_xml main.".to_string()); // 3
    match log_writer.flush() {
      Ok(_) => println!("flushed log file."),
      Err(e) => println!("error while flushing log file: {:?}", e)
    }
  });
}