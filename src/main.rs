use std::error::Error;
use std::fs;
use std::io;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use ceres_mpq::*;
use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use clap::{AppSettings, Arg, ArgMatches, SubCommand};
use glob::Pattern as GlobPattern;
use walkdir::WalkDir;
use path_absolutize::Absolutize;

mod error;

use error::ToolError;

pub type AnyError = Box<dyn Error>;

fn main() {
    let matches = app_from_crate!()
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::ColorNever)
        .setting(AppSettings::VersionlessSubcommands)
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
        .subcommand(
            SubCommand::with_name("list")
                .about("lists files in an archive")
                .arg(
                    Arg::with_name("archive")
                        .index(1)
                        .value_name("archive")
                        .help("archive file to extract from")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("filter")
                        .short("f")
                        .long("filter")
                        .value_name("pattern")
                        .help("glob pattern to filter files with")
                        .takes_value(true)
                )
        )
        .subcommand(
            SubCommand::with_name("new")
                .about("creates a new archive")
                .arg(
                    Arg::with_name("input")
                        .value_name("dir")
                        .index(1)
                        .help("directory to create an archive from")
                        .takes_value(true)
                        .required(true)
                )
                .arg(
                    Arg::with_name("output")
                        .index(2)
                        .value_name("archive")
                        .help("output file to write to")
                        .takes_value(true)
                        .required(true),
                )
        )
        .get_matches_safe();

    let result = match matches {
        Err(error) => error.exit(),
        Ok(matches) => match matches.subcommand() {
            ("extract", Some(matches)) => command_extract(matches),
            ("view", Some(matches)) => command_view(matches),
            ("list", Some(matches)) => command_list(matches),
            ("new", Some(matches)) => command_new(matches),
            (cmd, _) => {
                eprintln!("Unknown subcommand {} encountered", cmd);
                std::process::exit(1)
            }
        },
    };

    if let Err(error) = result {
        eprintln!("ERROR: {}", error);
    }
}

fn command_extract(matches: &ArgMatches) -> Result<(), AnyError> {
    let pattern = pattern_from_matches(matches)?;
    let archive_path = matches.value_of("archive").unwrap();
    let out_dir: PathBuf = matches.value_of("output").unwrap().into();

    let archive_file = open_readonly_file(archive_path)?;
    let mut archive =
        Archive::open(archive_file).map_err(|cause| ToolError::MpqOpenError { cause })?;

    let listfile = archive.files().ok_or(ToolError::ListfileNotFound)?;
    let files = listfile.iter().map(|s| (s, s.replace("\\", "/")));

    create_dir(&out_dir)?;

    for (file, file_normalized) in files {
        if let Some(pattern) = &pattern {
            if !pattern.matches(&file_normalized) {
                continue;
            }
        }

        let path = PathBuf::from_str(&file_normalized).unwrap();
        let file_result = archive.read_file(&file);

        match file_result {
            Ok(contents) => {
                let out_path = out_dir.join(path);
                let out_path_dir = out_path.parent().unwrap();
                create_dir(out_path_dir)?;
                if let Err(error) = fs::write(&out_path, contents) {
                    eprintln!("Could not write file {}: {}", out_path.display(), error)
                }
            }
            Err(error) => eprintln!("Could not extract file {}: {}", file, error),
        }
    }

    Ok(())
}

fn command_view(matches: &ArgMatches) -> Result<(), AnyError> {
    let archive_path = matches.value_of("archive").unwrap();
    let filename = matches.value_of("file").unwrap();

    let archive_file = open_readonly_file(archive_path)?;
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

fn command_list(matches: &ArgMatches) -> Result<(), AnyError> {
    let pattern = pattern_from_matches(matches)?;
    let archive_path = matches.value_of("archive").unwrap();
    let archive_file = open_readonly_file(archive_path)?;
    let mut archive =
        Archive::open(archive_file).map_err(|cause| ToolError::MpqOpenError { cause })?;

    let listfile = archive.files().ok_or(ToolError::ListfileNotFound)?;

    let files = listfile.iter().map(|s| s.replace("\\", "/"));

    if let Some(pattern) = pattern {
        for file in files {
            if pattern.matches(&file) {
                println!("{}", file);
            }
        }
    } else {
        for file in files {
            println!("{}", file);
        }
    };

    Ok(())
}

fn pattern_from_matches(matches: &ArgMatches) -> Result<Option<GlobPattern>, AnyError> {
    let pattern = matches.value_of("filter").map(|s| GlobPattern::new(s));

    let pattern = match pattern {
        None => None,
        Some(Err(error)) => return Err(Box::new(error)),
        Some(Ok(pattern)) => Some(pattern),
    };

    Ok(pattern)
}

fn command_new(matches: &ArgMatches) -> Result<(), AnyError> {
    let output_path = matches.value_of("output").unwrap();
    let input_dir = matches.value_of("input").unwrap();

    let output_file = open_write_file(output_path)?;
    let input_dir_absolute = PathBuf::from_str(input_dir).unwrap().absolutize()?;

    let mut creator = Creator::default();
    let file_options = FileOptions {
        encrypt: false,
        compress: true,
        adjust_key: false,
    };

    for entry in WalkDir::new(input_dir).follow_links(true) {
        match entry {
            Ok(entry) => if entry.file_type().is_file() {
                let path = entry.path().absolutize()?;
                let relative_path = path.strip_prefix(&input_dir_absolute)?;

                let file_contents = match fs::read(&path) {
                    Ok(contents) => contents,
                    Err(error) => {
                        eprintln!("Could not add file {}: {}", path.display(), error);
                        continue
                    }
                };

                creator.add_file(relative_path.to_str().unwrap(), file_contents, file_options);
            },
            Err(error) => eprintln!("{}", error),
        }
    }

    if let Err(error) = creator.write(output_file) {
        eprintln!("Failed to create archive: {}", error);
    }

    Ok(())
}

fn open_readonly_file(path: &str) -> Result<fs::File, ToolError> {
    fs::OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|cause| ToolError::FileOpenError {
            path: path.into(),
            cause,
        })
}

fn open_write_file(path: &str) -> Result<fs::File, ToolError> {
    fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .map_err(|cause| ToolError::FileOpenError {
            path: path.into(),
            cause,
        })
}

fn create_dir<P: AsRef<Path>>(dir: P) -> Result<(), ToolError> {
    fs::create_dir_all(&dir).map_err(|error| ToolError::OutDirCreationError {
        cause: error,
        path: dir.as_ref().to_owned(),
    })
}
