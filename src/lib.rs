#![feature(proc_macro_hygiene)]

use skyline::{hook, install_hook};
use prc::*;
use prcx::*;
use walkdir::WalkDir;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::thread;
use std::time::Duration;

fn diff_prc_files() {
  println!("starting diff of files");
    let mut paths = vec![]; 
    println!("made paths vec");
    let dir = walkdir::WalkDir::new("mods:/");
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
      println!("original path: {}", path);
      let arc_path = path.replace("mods:/", "arc:/");
      println!("arc path: {}", arc_path);
      

      // load arc_path
      let arc_file = prcx::open(arc_path.clone());

      // load mod_path
      let mod_file = prcx::open(path.clone());

      println!("diffing!");
      // diff files
      let arc_params = arc_file.unwrap();
      let mod_params = mod_file.unwrap();
      let diff = prcx::generate_patch(&arc_params, &mod_params).unwrap().unwrap();


      let output_pathname = arc_path.replace("arc:/", "sd:/xml/");
      println!("output path: {}", output_pathname);
      let output_path = std::path::Path::new(&output_pathname).parent().unwrap();
      
      std::fs::create_dir_all(output_pathname.clone()).unwrap();
      let f = File::create(output_pathname).expect("Unable to create file");
      let mut writer = BufWriter::new(f);
      prcx::write_xml(&diff, &mut writer);

    }
  }


#[skyline::main(name = "prc_to_xml")]
pub fn main() {
  thread::spawn(|| {
    for i in 1..100 {
        println!("xml wait: {}%", i);
        thread::sleep(Duration::from_millis(1000));
    }

    println!("prc_to_xml main!");

    diff_prc_files();

    println!("end prc_to_xml main."); // 3
  });
  
}
