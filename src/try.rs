use std::fs;
use std::io;

struct FileEntry {
    name: String,
    is_directory: bool,
}

enum EntryKind {
    File,
    Directory,
}

fn main() -> io::Result<()>{
    let name = "igh";
    let version = 1;

    println!("{} v{}", name, version);


    let total = add(10, 5);
    println!("Total: {}", total);


    let mut files: Vec<&str> = vec![
        "Downloads",
        "Documents",
        "motes.txt",
        "image.png",
    ];

    files.push("readme.md");

    for file in &files{
        println!("{}", file);
    }


    let entry = FileEntry{
        name: String::from("Downloads"),
        is_directory: true,
    };

    if entry.is_directory {
        println!("[DIR] {}", entry.name);
    }
    else{
        println!("[FILE] {}", entry.name);
    }


    let kind = EntryKind::Directory;
    match kind{
        EntryKind::File => println!("[FILE]"),
        EntryKind::Directory => println!("[DIR]"),
    }



    let entries = fs::read_dir(".")?;

    for entry in entries{
        let entry = entry?;
        println!("{}", entry.file_name().to_string_lossy());
    }

    Ok(())
}

fn add(a: i32, b: i32) -> i32{
    a+b
}