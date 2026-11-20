use std::io::{self, Write};
use std::path::PathBuf;
use std::{default, fs, string};

pub fn unique_dest(dest_dir: &PathBuf, name: &std::ffi::OsStr) -> PathBuf {
    let mut dest = dest_dir.join(name);
    if !dest.exists() {
        return dest;
    }
    let original = PathBuf::from(name);
    let stem = original
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let ext = original
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    let mut n = 1;
    while dest.exists() {
        dest = dest_dir.join(format!("{}({}){}", stem, n, ext));
        n += 1;
    }
    dest
}

pub fn copy_dir_recursive(src: &PathBuf, dest: &PathBuf) -> io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
