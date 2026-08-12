# Architecture

`hammer` is a single Rust binary with three core layers.

## 1. CLI Layer

`src/cli.rs` defines the command line interface:

- `backup`
- `restore`

This layer only parses input and validates flags.

## 2. Orchestration Layer

`src/mongo.rs` coordinates the end-to-end flows:

- Connect to MongoDB.
- Read collection names.
- Export collections to JSON files.
- Zip the export directory.
- Upload the zip to S3 when requested.
- Restore collection documents from either a directory or archive.

## 3. Utility Layer

- `src/archive.rs` handles zip and unzip operations.
- `src/s3.rs` handles S3 uploads.

## Data Flow

Backup flow:

1. Connect to MongoDB.
2. List collections.
3. Export each collection to `<collection>.json`.
4. Zip the temporary export directory.
5. Optionally upload the archive to S3.

Restore flow:

1. Resolve the source path.
2. If the source is a zip archive, extract it to a temporary directory.
3. Read each `*.json` file.
4. Replace the target collection contents with the restored documents.

## Current Tradeoffs

- Restore currently replaces collection contents instead of merging documents.
- S3 upload relies on AWS default credential resolution plus `AWS_BUCKET`.
- The archive format is intentionally simple: one JSON file per collection.
