use crate::archive;
use crate::cli::{BackupArgs, RestoreArgs};
use crate::s3;
use anyhow::{Context, Result};
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::Client;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub async fn backup_database(args: BackupArgs) -> Result<()> {
    let client = Client::with_uri_str(&args.connection)
        .await
        .with_context(|| format!("failed to connect to MongoDB at {}", args.connection))?;
    let database = client.database(&args.database);
    let collections = database
        .list_collection_names()
        .await
        .with_context(|| format!("failed to list collections for database {}", args.database))?;

    let temp_dir = TempDir::new().context("failed to create temporary directory")?;
    let temp_path = temp_dir.path();

    for collection_name in collections {
        let mut cursor = database
            .collection::<Document>(&collection_name)
            .find(doc! {})
            .await
            .with_context(|| format!("failed to query collection {collection_name}"))?;

        let file_path = temp_path.join(format!("{collection_name}.json"));
        write_collection_as_json(&mut cursor, &file_path)
            .await
            .with_context(|| {
                format!(
                    "failed to write {collection_name} to {}",
                    file_path.display()
                )
            })?;
    }

    let output_path = args.output.unwrap_or_else(default_output_path);
    archive::zip_directory(temp_path, &output_path)?;
    println!("backup written to {}", output_path.display());

    if args.upload {
        let bucket = std::env::var("AWS_BUCKET")
            .context("environment variable AWS_BUCKET is required when --upload is used")?;
        let location = s3::upload_file(&output_path, &bucket).await?;
        println!("uploaded to {location}");
    }

    Ok(())
}

pub async fn restore_database(args: RestoreArgs) -> Result<()> {
    let source = args.source_path()?;
    let client = Client::with_uri_str(&args.connection)
        .await
        .with_context(|| format!("failed to connect to MongoDB at {}", args.connection))?;
    let database = client.database(&args.database);

    if args.is_archive() {
        let temp_dir =
            TempDir::new().context("failed to create temporary directory for extraction")?;
        archive::unzip_archive(&source, temp_dir.path())?;
        restore_from_folder(&database, temp_dir.path()).await?;
        return Ok(());
    }

    restore_from_folder(&database, &source).await?;
    Ok(())
}

async fn restore_from_folder(database: &mongodb::Database, folder: impl AsRef<Path>) -> Result<()> {
    let folder = folder.as_ref();
    let entries = fs::read_dir(folder)
        .with_context(|| format!("failed to read directory {}", folder.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let collection_name = collection_name_from_path(path.as_path())?;
        println!("restoring collection {collection_name}");

        let docs = read_collection_from_json(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;

        let collection = database.collection::<Document>(collection_name);
        collection
            .delete_many(doc! {})
            .await
            .with_context(|| format!("failed to clear collection {collection_name}"))?;
        if !docs.is_empty() {
            collection
                .insert_many(docs)
                .await
                .with_context(|| format!("failed to insert documents into {collection_name}"))?;
        }
    }

    Ok(())
}

async fn write_collection_as_json(
    cursor: &mut mongodb::Cursor<Document>,
    file_path: impl AsRef<Path>,
) -> Result<()> {
    let file = File::create(file_path.as_ref())
        .with_context(|| format!("failed to create {}", file_path.as_ref().display()))?;
    let mut writer = BufWriter::new(file);
    writer.write_all(b"[\n")?;

    let mut first = true;
    while let Some(document) = cursor.try_next().await? {
        if !first {
            writer.write_all(b",\n")?;
        }
        serde_json::to_writer_pretty(&mut writer, &document)
            .context("failed to serialize document as JSON")?;
        first = false;
    }

    writer.write_all(b"\n]\n")?;
    writer.flush()?;
    Ok(())
}

fn read_collection_from_json(path: impl AsRef<Path>) -> Result<Vec<Document>> {
    let file = File::open(path.as_ref())
        .with_context(|| format!("failed to open {}", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let docs: Vec<Document> = serde_json::from_reader(reader)
        .with_context(|| format!("failed to parse {}", path.as_ref().display()))?;
    Ok(docs)
}

fn collection_name_from_path<'a>(path: &'a Path) -> Result<&'a str> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .context("invalid collection file name")
}

fn default_output_path() -> PathBuf {
    PathBuf::from(format!(
        "hammer-backup-{}.zip",
        Utc::now().format("%Y%m%dT%H%M%SZ")
    ))
}

#[cfg(test)]
mod tests {
    use super::{collection_name_from_path, read_collection_from_json};
    use mongodb::bson::doc;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn collection_name_from_path_strips_json_extension() {
        let name = collection_name_from_path(std::path::Path::new("/tmp/users.json")).unwrap();
        assert_eq!(name, "users");
    }

    #[test]
    fn read_collection_from_json_parses_documents() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("users.json");
        fs::write(
            &file_path,
            r#"[
  {"name":"Ada"},
  {"name":"Linus","active":true}
]"#,
        )
        .unwrap();

        let docs = read_collection_from_json(&file_path).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0], doc! { "name": "Ada" });
        assert_eq!(docs[1], doc! { "name": "Linus", "active": true });
    }

    #[test]
    fn read_collection_from_json_rejects_invalid_json() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("broken.json");
        fs::write(&file_path, "{not json}").unwrap();

        let err = read_collection_from_json(&file_path).unwrap_err();
        assert!(err.to_string().contains("failed to parse"));
    }
}
