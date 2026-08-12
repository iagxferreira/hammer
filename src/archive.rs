use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

pub fn zip_directory(source_dir: impl AsRef<Path>, destination: impl AsRef<Path>) -> Result<()> {
    let source_dir = source_dir.as_ref();
    let destination = destination.as_ref();

    let file = File::create(destination)
        .with_context(|| format!("failed to create archive {}", destination.display()))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        let path = entry.path();
        let relative = path
            .strip_prefix(source_dir)
            .with_context(|| format!("failed to build relative path for {}", path.display()))?;

        if relative.as_os_str().is_empty() {
            continue;
        }

        let name = relative.to_string_lossy().replace('\\', "/");

        if entry.file_type().is_dir() {
            zip.add_directory(format!("{name}/"), options)
                .with_context(|| format!("failed to add directory {name} to zip"))?;
            continue;
        }

        zip.start_file(name.clone(), options)
            .with_context(|| format!("failed to add file {name} to zip"))?;
        let mut input =
            File::open(path).with_context(|| format!("failed to open file {}", path.display()))?;
        io::copy(&mut input, &mut zip)
            .with_context(|| format!("failed to copy file {} into zip", path.display()))?;
    }

    zip.finish().context("failed to finish zip archive")?;
    Ok(())
}

pub fn unzip_archive(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> Result<PathBuf> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;

    let file = File::open(source)
        .with_context(|| format!("failed to open archive {}", source.display()))?;
    let mut archive = zip::ZipArchive::new(file).context("failed to read zip archive")?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .context("failed to read zip entry")?;
        let Some(safe_name) = entry.enclosed_name() else {
            anyhow::bail!("archive contains an unsafe path: {}", entry.name());
        };
        let outpath = destination.join(safe_name);

        if entry.is_dir() {
            fs::create_dir_all(&outpath)
                .with_context(|| format!("failed to create {}", outpath.display()))?;
            continue;
        }

        if let Some(parent) = outpath.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let mut outfile = File::create(&outpath)
            .with_context(|| format!("failed to create {}", outpath.display()))?;
        io::copy(&mut entry, &mut outfile)
            .with_context(|| format!("failed to extract {}", outpath.display()))?;
    }

    Ok(destination.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::{unzip_archive, zip_directory};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn zip_and_unzip_round_trip_directory_contents() {
        let source_dir = tempdir().unwrap();
        let source = source_dir.path();
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("users.json"), r#"[{"name":"Ada"}]"#).unwrap();
        fs::write(
            source.join("nested").join("roles.json"),
            r#"[{"role":"admin"}]"#,
        )
        .unwrap();

        let zip_path = source.join("backup.zip");
        zip_directory(source, &zip_path).unwrap();

        let extract_dir = tempdir().unwrap();
        unzip_archive(&zip_path, extract_dir.path()).unwrap();

        let users = fs::read_to_string(extract_dir.path().join("users.json")).unwrap();
        let roles =
            fs::read_to_string(extract_dir.path().join("nested").join("roles.json")).unwrap();

        assert!(users.contains("Ada"));
        assert!(roles.contains("admin"));
    }

    #[test]
    fn unzip_archive_rejects_missing_file() {
        let extract_dir = tempdir().unwrap();
        let err = unzip_archive("/tmp/does-not-exist.zip", extract_dir.path()).unwrap_err();
        assert!(err.to_string().contains("failed to open archive"));
    }
}
