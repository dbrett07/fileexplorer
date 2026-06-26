use std::{error::Error, fmt}; 
use chrono::{DateTime, Utc}; 
use windows::core::{HSTRING};
use windows::Win32::Foundation::{ERROR_NO_MORE_FILES, FILETIME, HANDLE, INVALID_HANDLE_VALUE, MAX_PATH};
use windows::Win32::Storage::FileSystem::{FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_READONLY, FindClose, FindFirstFileW, FindNextFileW, WIN32_FIND_DATAW};

const TICKS_PER_SECOND : i64 = 10000000; // windows ticks in 100ns intervals
const EPOCH_DIFFERENCE : i64 = 116444736000000000; // windows epoch starts in 1601 but unix in 1970

pub fn list_dir_files(starting_dir : &String) -> Vec<FileInfo> { 
    let mut out : Vec<FileInfo> = Vec::new(); 
    match get_dir_files(starting_dir, &mut out) { 
        Ok(v) => v, 
        Err(e) => out
    }
} 

fn get_dir_files(directory : &String, store : & mut Vec<FileInfo>) -> Result<Vec<FileInfo>, FileError> { 
    println!("Call");
    let mut ffd : WIN32_FIND_DATAW = WIN32_FIND_DATAW::default();
    let ffdp = &mut ffd as *mut _;  

    let mut find_handle : HANDLE = INVALID_HANDLE_VALUE; 
    let sz_dir : String = directory.clone() + "\\\\*"; 

    if directory.len() as u32 > (MAX_PATH - 3) {
        return Err(FileError {});
    }

    unsafe { 
        find_handle = match FindFirstFileW(&HSTRING::from(&sz_dir), ffdp) { 
            Ok(h) => h, 
            Err(e) => {
                FindClose(find_handle); 
                return Err(FileError {})
            }
        };

        if find_handle == INVALID_HANDLE_VALUE { 
            FindClose(find_handle); 
            return Err(FileError {  });
        }

        loop {
            let file = *ffdp; 
            let fstr =  win32_to_string(&file.cFileName);
            // println!("{fstr}");
            let x = FileInfo { 
                name : fstr.clone(), 
                read_only : file.dwFileAttributes & (FILE_ATTRIBUTE_READONLY.0) > 0, 
                directory : file.dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY.0) > 0,
                modified : DateTime::from_timestamp(filetime_to_unix(file.ftLastWriteTime), 0).expect("oops")
            };
            store.push(x);
            let l = store.len();
            println!("{l}");
            if  (*ffdp).dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY.0) > 0 && fstr != "." && fstr != ".." { 
                store.extend(get_dir_files(&(directory.clone() + "\\" + &fstr), &mut Vec::new()).expect("oh no")); 
            }

            match FindNextFileW(find_handle, ffdp) { 
                Ok(_) => continue, 
                Err(e) => if e.code() != ERROR_NO_MORE_FILES.into() { 
                    FindClose(find_handle); 
                    return Err(FileError {});
                } else { 
                    break; 
                }
           }
        }


    } 

    Ok(store.to_vec())
}

fn win32_to_string(buffer : &[u16; 260]) -> String { 
    let len = buffer.iter().position(|&x| x == 0).unwrap_or(buffer.len()); 
    String::from_utf16_lossy(&buffer[..len])
}

fn filetime_to_unix(ftime : FILETIME) -> i64 { 
    let mut temp : i64 = 0; 
    temp |= (ftime.dwHighDateTime as i64) << 32;
    temp |= ftime.dwLowDateTime as i64; 

    temp -= EPOCH_DIFFERENCE;
    temp /= TICKS_PER_SECOND; 
    return temp;
}

#[derive(Debug, Clone)]
pub struct FileInfo { 
    name : String, 
    read_only : bool, 
    directory : bool, 
    modified : DateTime<Utc>
}

impl fmt::Display for FileInfo { 
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileInfo<name={}, read={}, dir={}, modified={}>", self.name, self.read_only, self.directory, self.modified.to_string())
    }
}

#[derive(Debug)]
pub struct FileError {}

impl Error for FileError {}

impl fmt::Display for FileError { 
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "There was an error while retrieving a file.")
    }
}