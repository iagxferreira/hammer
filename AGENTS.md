# Agents

This file is for automated contributors working in this repository.

## Purpose

`hammer` is a Rust CLI that:

1. Connects to MongoDB.
2. Exports each collection to JSON.
3. Packs the collection files into a zip archive.
4. Optionally uploads the archive to S3.
5. Restores from either a folder of JSON files or a zip archive.

## Code Layout

- `src/main.rs`: binary entrypoint.
- `src/cli.rs`: CLI parsing and argument types.
- `src/mongo.rs`: backup and restore orchestration.
- `src/archive.rs`: zip and unzip helpers.
- `src/s3.rs`: S3 upload logic.

## Working Rules

- Prefer small, focused changes.
- Keep the CLI behavior consistent with the README and `CONTRIBUTING.md`.
- Add or update tests for any logic change in `src/archive.rs`, `src/cli.rs`, or `src/mongo.rs`.
- Run `cargo fmt` and `cargo test` before finishing.
- Avoid adding new runtime dependencies unless they clearly simplify the implementation.

## Runtime Notes

- `backup --upload` expects `AWS_BUCKET` to be set.
- The S3 client uses the default AWS credential chain and region resolution.
- `restore --folder` and `restore --archive` are mutually exclusive.
