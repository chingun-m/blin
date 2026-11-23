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

pub fn add_to_zip<W: io::Write + io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    path: &PathBuf,
    name_in_zip: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let options = zip::write::SimpleFileOptions::default();

    if path.is_dir() {
        zip.add_directory(format!("{}/", name_in_zip), options)?;
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let child = format!("{}/{}", name_in_zip, entry.file_name().to_string_lossy());
            add_to_zip(zip, &entry.path(), &child)?;
        }
    } else {
        zip.start_file(name_in_zip, options)?;
        let mut f = fs::File::open(path)?;
        io::copy(&mut f, zip)?;
    }
    Ok(())
}

pub fn create_zip(targets: &[PathBuf], dest: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::create(dest)?;
    let mut zip = zip::ZipWriter::new(file);

    for path in targets {
        let name = match path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };
        add_to_zip(&mut zip, path, &name)?;
    }

    zip.finish()?;
    Ok(())
}

pub fn extract_zip(zip_path: &PathBuf, dest: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    fs::create_dir_all(dest)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;

        // enclosed_name rejects malicious paths like "../../evil.exe"
        let relative = match entry.enclosed_name() {
            Some(p) => p,
            None => continue,
        };
        let out_path = dest.join(relative);

        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = fs::File::create(&out_path)?;
            io::copy(&mut entry, &mut out_file)?;
        }
    }
    Ok(())
}
