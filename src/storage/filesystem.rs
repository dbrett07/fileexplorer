use chrono::{DateTime, Utc}; 
use std::{collections::HashMap, fmt, str}; 

#[derive(Debug, Clone)]
pub struct FileNode { 
    name : String, 
    read_only : bool, 
    modified : DateTime<Utc>
}

impl fmt::Display for FileNode { 
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileNode<name={}, read={}, modified={}>", self.name, self.read_only, self.modified.to_string())
    }
}

pub struct DirectoryNode { 
    id : usize,
    name : String, 
    parent : Option<usize>, 
    files : Vec<FileNode>, 
    subdirs : HashMap<String, usize>
} 

impl DirectoryNode { 
    pub fn new(id : usize, name : String, parent : Option<usize>) -> DirectoryNode { 
        DirectoryNode {
            id : id,
            name : name,
            parent : parent, 
            files : Vec::new(), 
            subdirs : HashMap::new()
        } 
    }

    pub fn add_file(&mut self, name : String, read_only : bool, modified: DateTime<Utc>) { 
        self.files.push(FileNode { 
            name, 
            read_only, 
            modified 
        })
    }

    pub fn remove_file(&mut self, name : String) -> Option<FileNode> { 
        let mut tmp : usize = usize::MAX; 
        for i in 0..self.files.len() { 
            if self.files.get(i).expect("No file").name == name { 
                tmp = i; 
                break; 
            }
        }

        if tmp != usize::MAX {
            Some(self.files.remove(tmp));
        } 

        None
    }

    pub fn add_subdir(&mut self, name : String, id : usize) { 
        let pair = self.subdirs.iter().find(|&(_, v)| *v == id);

        match pair { 
            Some((k, _)) => {self.subdirs.remove(&k.clone()); self.subdirs.insert(name, id); }
            None => {self.subdirs.insert(name, id);}
        }
    }

    pub fn remove_subdir_by_id(&mut self, id : &usize) { 
        self.subdirs.retain(|_, v| v != id);
    }

    pub fn remove_subdir_by_name(&mut self, name : &String) { 
        self.subdirs.retain(|k, _| k != name); 
    }

    pub fn get_subdir_by_name(&self, name : &str) -> Option<&usize> { 
        self.subdirs.get(name)
    }

    pub fn get_file_by_name(&self, name : &str) -> Option<&FileNode> { 
        for file in &self.files { 
            if file.name == name { 
                return Some(file); 
            }
        }

        None 
    }

    pub fn set_parent(&mut self, id : usize) { 
        self.parent = Some(id)
    }

    ///     Return the flat size of the directory (not recursively including the size of subdirectories).
    pub fn size(&self) -> usize { 
        return self.files.len() + self.subdirs.len(); 
    }

}

/// Hold information about a filesystem on disk.
pub struct FileSystem { 
    dirs : Vec<DirectoryNode>, 
}

impl FileSystem { 
    pub fn new(root_dir : DirectoryNode) -> FileSystem { 
        FileSystem { 
            dirs : vec![root_dir]
        }
    }

    pub fn numdirs(&self) -> usize { 
        return self.dirs.len();  
    }

    // pub fn numfiles(&self) -> usize { 

    // }

    fn descend_to_dir(&self, path : &str) -> Option<&DirectoryNode> { 
        let path_parts = path.split("/").collect::<Vec<&str>>();
        let mut current_dir : &DirectoryNode = self.dirs.get(0).expect("Filesystem has no root");

        for i in 0..path_parts.len() { 
            let file_idx = *path_parts.get(i) .expect("Path out of boudns access"); 

            match current_dir.get_subdir_by_name(file_idx) { 
                Some(u) => {
                    current_dir = self.dirs.get(*u).expect("Subdir {tmp.name} held a reference to /{part}, but it is not in FS array"); 
                },
                None => { 
                    return None;
                }
            }
        }

       Some(current_dir)
    }

    /// Search for a file or directory at path `path` in the file system.
    /// Directory paths are expected to end in `/`. 
    pub fn search(&self, path : &str) -> Option<Result<&FileNode, &DirectoryNode>> { 
        let (target_dir_path, target_file)  = path.rsplit_once("/").unwrap_or(("", ""));

        let target = match self.descend_to_dir(&(target_dir_path.to_owned() + "/")) { 
            Some(dir) => dir, 
            None => {return None; }
        };

        if target_file == "" { 
            return Some(Err(target)); 
        }

        match target.get_file_by_name(target_file) { 
            Some(f) => Some(Ok(f)), 
            None => None 
        }
    }

    fn get_dir_id(&self, path : &str) -> Option<usize> { 
        if !path.ends_with("/") { 
            return None; // Not a directory -> No such directory 
        }

        match self.descend_to_dir(path) { 
            Some(dir) => { 
                Some(dir.id)
            }
            None => None 
        }
    }
    
    pub fn add_dir(&mut self, name : &str, path : &str) { 
        self.dirs.push(DirectoryNode::new(self.dirs.len(), name.to_string(), self.get_dir_id(path)));
    }
}