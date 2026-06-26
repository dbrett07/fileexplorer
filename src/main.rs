use crate::storage::storage::list_dir_files;

mod storage; 

fn main() {
    println!("Hello, world!");
    let dir : String = String::from("C:\\Users\\dylan\\OneDrive\\Documents\\rust"); 
    let files = list_dir_files(&dir);
    for f in files { 
        println!("{f}");
    }
    println!("done");
}
