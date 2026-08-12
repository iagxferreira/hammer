use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "hammer", about = "Backup and restore MongoDB databases")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Backup(BackupArgs),
    Restore(RestoreArgs),
}

#[derive(Debug, Args, Clone)]
pub struct BackupArgs {
    #[arg(short, long)]
    pub connection: String,

    #[arg(short, long)]
    pub database: String,

    #[arg(short, long)]
    pub output: Option<PathBuf>,

    #[arg(short, long)]
    pub upload: bool,
}

#[derive(Debug, Args, Clone)]
pub struct RestoreArgs {
    #[arg(short, long)]
    pub connection: String,

    #[arg(short, long)]
    pub database: String,

    #[arg(short, long, conflicts_with = "archive")]
    pub folder: Option<PathBuf>,

    #[arg(short, long, conflicts_with = "folder")]
    pub archive: Option<PathBuf>,
}

impl RestoreArgs {
    pub fn source_path(&self) -> anyhow::Result<PathBuf> {
        self.folder
            .clone()
            .or_else(|| self.archive.clone())
            .ok_or_else(|| anyhow::anyhow!("either --folder or --archive must be provided"))
    }

    pub fn is_archive(&self) -> bool {
        self.archive.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command};
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn parses_backup_command() {
        let cli = Cli::parse_from([
            "hammer",
            "backup",
            "--connection",
            "mongodb://localhost:27017",
            "--database",
            "hammer",
            "--output",
            "/tmp/backup.zip",
            "--upload",
        ]);

        match cli.command {
            Command::Backup(args) => {
                assert_eq!(args.connection, "mongodb://localhost:27017");
                assert_eq!(args.database, "hammer");
                assert_eq!(args.output, Some(PathBuf::from("/tmp/backup.zip")));
                assert!(args.upload);
            }
            _ => panic!("expected backup command"),
        }
    }

    #[test]
    fn parses_restore_archive_command() {
        let cli = Cli::parse_from([
            "hammer",
            "restore",
            "--connection",
            "mongodb://localhost:27017",
            "--database",
            "hammer",
            "--archive",
            "/tmp/backup.zip",
        ]);

        match cli.command {
            Command::Restore(args) => {
                assert_eq!(args.archive, Some(PathBuf::from("/tmp/backup.zip")));
                assert_eq!(
                    args.source_path().unwrap(),
                    PathBuf::from("/tmp/backup.zip")
                );
                assert!(args.is_archive());
            }
            _ => panic!("expected restore command"),
        }
    }
}
