# hammer

A Rust CLI to back up MongoDB databases into a zip archive and optionally upload the archive to S3.

## Docs

- [Architecture](ARCHITECTURE.md)
- [Contributing](CONTRIBUTING.md)
- [Agent notes](AGENTS.md)

## Building

```shell
make build
```

## Running

### Backup

```shell
hammer backup --connection mongodb://localhost:27017 --database hammer --upload
```

### Restore

```shell
hammer restore --folder /path/to/your/folder --connection mongodb://localhost:27017 --database example
hammer restore --archive /path/to/backup.zip --connection mongodb://localhost:27017 --database example
```

## Example of use

### Backup

```shell
AWS_ACCESS_KEY_ID=example AWS_SECRET_ACCESS_KEY=example AWS_REGION=us-east-1 AWS_BUCKET=example target/release/hammer backup --connection mongodb://localhost:27017 --database example --upload
```

### Restore

```shell
target/release/hammer restore --folder /path/to/your/folder --connection mongodb://localhost:27017 --database example
```

### Restore from archive

```shell
target/release/hammer restore --archive /path/to/backup.zip --connection mongodb://localhost:27017 --database example
```

## Development

```shell
make fmt
make test
make check
```
