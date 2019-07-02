extern crate dirs; //Used to get the user's home dir
extern crate args; //Used to handle cli arguments
extern crate getopts;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::{Error, ErrorKind};
use std::vec::Vec;

fn list_dir(dir: &Path) -> std::io::Result<Vec<PathBuf>> { //Get all files and dirs in given dir
    let mut vec = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        vec.push(entry.path());
    }
    Ok(vec)
}

fn get_dirs(dir: Vec<PathBuf>) -> Vec<PathBuf> { //Returns only PathBufs that are attributed to dirs, not files
    let mut v = Vec::new();
    for d in dir.iter() {
        if d.is_dir() { v.push(d.to_path_buf()); }
    }
    v
}

fn recursive_dir_grab(dir: &Path) -> std::io::Result<Vec<PathBuf>> { //Returns a list of all dirs starting from given Path 
    let mut final_list: Vec<PathBuf> = Vec::new();
    final_list.push(dir.to_path_buf());
    
    let list = list_dir(dir)?;
    let dir_list = get_dirs(list);
    
    for d in dir_list.iter() {
        let mut skip = false;
        for p in d.components() {
            if p == std::path::Component::Normal(".wine".as_ref()){
                skip = true;
            }
            if p == std::path::Component::Normal(".joe_backups".as_ref()) {
                skip = true;
            }
        }
        if skip {
            break;
        }
        let recursive_dir_list = recursive_dir_grab(d)?;
        for r_dir in recursive_dir_list.iter() { 	//When this variable, "r_dir" is only one character 
            final_list.push(r_dir.to_path_buf());	//long, joe's rust syntax highlighting stops working
        }
        final_list.push(d.to_path_buf());
    }
    Ok(final_list)
}

fn find_backup_files(ext: String) -> std::io::Result<Vec<PathBuf>> { //Get a vector of all files ending with '~' starting in home_dir
    let home_dir = dirs::home_dir();
    let home_dir = match home_dir {
        Some(path) => path,
        None  => {
            return Err(Error::new(ErrorKind::Other, "Error getting home directory."));
        },
    };
    
    //Make a vec of directories to search
    let dirs = recursive_dir_grab(&home_dir)?;
    let mut files = Vec::new();
    for d in dirs.iter() {
        let curr_dir = list_dir(d)?;
        for f in curr_dir.iter() {
            if f.is_file() { files.push(f.to_path_buf()); }
        }
    }
    
    println!("[-] Finding files ending with {}", ext);
    
    let mut found: Vec<PathBuf> = Vec::new();
    for p in files.iter(){
        let s = p.to_str();
        let f = match s{ //Find
            Some(string) => string.find(&ext), //If s was succesfully converted to str, then see if '~' is in the string
            None => None,
        };
        match f{ //See if '~' was found
            Some(_) => {
                let ext_length = ext.chars().count();
                let last_char_vec: Vec<char> = s.unwrap().chars().rev().take(ext_length).collect();
                let last_char_string: String = last_char_vec.into_iter().rev().collect();
                if ext == last_char_string {
                    found.push(p.to_path_buf());
                }
            },
            None => (),
        };
    }
    Ok(found)
}

fn main() {
    //Argument handling
    let mut arg = args::Args::new("joe-bbm", "Joe-betterbackups Manager");
    arg.flag("h", "help", "Print program usage.");
    arg.flag("f", "find", "Find Joe backup files.");
    arg.option("e", "extension", "File extension Joe uses for backups. Default: ~", "EXT", getopts::Occur::Optional, None);
    
    let input: Vec<String> = env::args().collect();
    let parse_res = arg.parse(input);
    match parse_res {
        Ok(_) => (),
        Err(e) => {
            println!("Error: {}", e);
            return;
        },
    };
    
    let help = arg.value_of("help"); //Help argument
    let help = match help {
        Ok(h) => h,
        Err(e) => {
            println!("Error: {}", e);
            return;
        },
    };
    if help {
        println!("{}", arg.full_usage());
        return;
    }
    
    let find = arg.value_of("find"); //Find argument
    let find = match find {
        Ok(f) => f,
        Err(e) => {
            println!("Error {}", e);
            return;
        },
    };
    let extension = arg.value_of("extension");
    let extension = match extension {
        Ok(e) => e,
        Err(e) => String::from("~")
    };
    
    if find {
        let backup_files = find_backup_files(extension);
        let backup_files = match backup_files {
            Ok(vec) => vec,
            Err(err) => {
                println!("Error: {:?}", err.get_ref());
                return;
            },          
        };
        for f in backup_files.iter() {
            println!("{}", f.to_str().unwrap());
        }
    }
    else {
        if extension != "~" {
            println!("Invalid argument: -e {}. Setting the extension option requires finding files with -f", extension);
            return;
        }
    }
}

