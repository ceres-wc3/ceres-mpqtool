use std::error::Error;
use std::fs;
use std::io;
use std::io::BufReader;
use std::io::Write;

use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, AppSettings, Arg,
    ArgMatches, SubCommand,
};

use ceres_mpq::*;
mod error;
use error::ToolError;

pub type AnyError = Box<dyn Error>;

fn main() {
    let matches = app_from_crate!()
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::ColorNever)
        .subcommand(
            SubCommand::with_name("extract")
                .about("extracts files from an archive")
                .arg(
                    Arg::with_name("archive")
                        .index(1)
                        .value_name("archive")
                        .help("archive file to extract from")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("output")
                        .value_name("dir")
                        .short("o")
                        .long("output")
                        .help("directory where to output extracted files")
                        .default_value("./")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("filter")
                        .value_name("pattern")
                        .long("filter")
                        .short("f")
                        .help("if specified, will only extract files which match the specified glob-pattern")
                        .takes_value(true)
                )
        )
        .subcommand(
            SubCommand::with_name("view")
                .about("views a single file in an archive")
                .arg(
                    Arg::with_name("archive")
                        .index(1)
                        .value_name("archive")
                        .help("archive file to extract from")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("file")
                        .index(2)
                        .value_name("filename")
                        .help("file inside the archive to view")
                        .takes_value(true)
                        .required(true)
                )
        )
        .get_matches_safe();

    let result = match matches {
        Err(error) => error.exit(),
        Ok(matches) => match matches.subcommand() {
            ("extract", Some(matches)) => command_extract(matches),
            ("view", Some(matches)) => command_view(matches),
            ("create", Some(matches)) => command_create(matches),
            (cmd, _) => {
                eprintln!("Unknown subcommand {} encountered", cmd);
                std::process::exit(1)
            }
        },
    };

    if let Err(error) = result {
        eprintln!("An error occured: {}", error);
    }
}

fn command_extract(matches: &ArgMatches) -> Result<(), AnyError> {
    Ok(())
}

fn command_view(matches: &ArgMatches) -> Result<(), AnyError> {
    let archive_path = matches.value_of("archive").unwrap();
    let filename = matches.value_of("file").unwrap();

    let archive_file = fs::OpenOptions::new()
        .read(true)
        .open(archive_path)
        .map_err(|cause| ToolError::FileOpenError {
            path: archive_path.into(),
            cause,
        })?;
    let archive_file = BufReader::new(archive_file);

    let mut archive =
        Archive::open(archive_file).map_err(|cause| ToolError::MpqOpenError { cause })?;
    let file_contents = archive
        .read_file(filename)
        .map_err(|cause| ToolError::MpqReadFileError { cause })?;

    let stdout = io::stdout();
    let mut lock = stdout.lock();

    lock.write_all(&file_contents)?;

    Ok(())
}

fn command_create(matches: &ArgMatches) -> Result<(), AnyError> {
    Ok(())
}
