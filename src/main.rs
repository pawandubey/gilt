#[allow(unused_attributes, unused_imports, dead_code)]
use std::path::PathBuf;
use std::{
    env::current_dir,
    ffi::OsStr,
    fs,
    path::Path,
    process::{self, Command, Output},
};
use structopt::StructOpt;
use walkdir::WalkDir;

mod rendering;

use rendering::Render;

#[derive(Debug, StructOpt)]
#[structopt()]
struct Options {
    #[structopt(short, long)]
    #[allow(dead_code)]
    colorize: bool,

    #[structopt(short, long, help = "Follows symlinks, panicking on loops")]
    follow_symlinks: bool,

    #[structopt(
        short,
        long,
        parse(from_os_str),
        env = "HOME",
        help = "The location to scan for repositories"
    )]
    location: PathBuf,

    #[structopt(short, long, parse(from_str), help = "The command to execute")]
    #[allow(dead_code)]
    exec: String,

    #[structopt(short, long, default_value = "stdin", help = "json, stdin")]
    #[allow(dead_code)]
    output: OutputType,
}

#[derive(Debug)]
pub enum OutputType {
    Stdin,
    Json,
}

impl std::str::FromStr for OutputType {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(OutputType::Json),
            "stdin" => Ok(OutputType::Stdin),
            _ => Err("Unsupported output type."),
        }
    }
}

// TODO: Maybe cleanup
// impl Display for OutputType {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match &self {
//             OutputType::Json => write!(f, "json"),
//             OutputType::Stdin => write!(f, "stdin"),
//         }
//     }
// }

fn hidden(file_name: &OsStr) -> bool {
    file_name
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn is_git_repository(path: &Path) -> bool {
    let is_not_hidden = !hidden(path.file_name().unwrap());

    let has_dot_git = if let Ok(mut children) = fs::read_dir(path) {
        let mut count = 0;
        let none = children.any(|c| {
            count += 1;
            let child = c.unwrap();
            child.path().is_dir() && child.file_name().to_str().map(|s| s.eq(".git")).unwrap()
        });

        count > 0 && none
    } else {
        false
    };

    is_not_hidden && has_dot_git
}

fn main() {
    let options = Options::from_args();

    let cmd: Vec<String>;

    match shellwords::split(&options.exec) {
        Ok(args) => cmd = args,
        Err(_) => {
            eprintln!("Failed to parse exec string");
            process::exit(exitcode::USAGE)
        }
    }

    // collect all repositories under path
    // - check that provided path is a directory
    if options.location.is_dir() {
        let mut renderer = rendering::get_renderer(options.output);

        // - start traversal
        let mut walker = WalkDir::new(options.location)
            .follow_links(options.follow_symlinks)
            .into_iter();

        let mut repositories: Vec<PathBuf> = Vec::new();

        loop {
            let entry = match walker.next() {
                None => break,
                Some(Err(err)) => panic!("Error: {}", err),
                Some(Ok(entry)) => entry,
            };

            if entry.file_type().is_dir() && is_git_repository(entry.path()) {
                repositories.push(entry.into_path().canonicalize().unwrap());
                walker.skip_current_dir();
            }
        }
        // process command in each repository and collect result
        let current_dir = current_dir().expect("Could not get current directory.");
        repositories
            .iter()
            .for_each(|repo| match exec(&cmd, repo, &current_dir) {
                Ok(out) => renderer.log(out, repo).unwrap_or_else(
                    |e| {
                        eprintln!("{}", e);
                    }),
                Err(err) => eprintln!("{}", err),
            });
    } else {
        eprintln!(
            "{} is not a directory or does not exist.",
            options.location.to_str().unwrap()
        );
        process::exit(exitcode::USAGE)
    }
}

#[cfg(unix)]
fn exec(cmd: &[String], repo: &Path, curr: &Path) -> Result<Output, &'static str> {
    use std::env::set_current_dir;

    match set_current_dir(repo) {
        Ok(_) => {
            let cmd_out = match Command::new(&cmd[0]).args(&cmd[1..]).output() {
                Ok(out) => Ok(out),
                _ => Err("Failed to run command"),
            };

            set_current_dir(curr).expect("Failed to change back to original directory");

            cmd_out
        }
        Err(_) => Err("Could not open dir"),
    }
}

#[cfg(windows)]
fn exec(cmd: String, repo: &PathBuf) -> Result<process::Output, std::io::Error> {
    todo()
}

#[cfg(test)]
mod tests {
    use assert_fs::fixture::{FileTouch, PathChild};

    use crate::{hidden, is_git_repository};

    #[test]
    fn test_hidden_file_is_detected() {
        let file = assert_fs::NamedTempFile::new(".secret").unwrap();

        assert_eq!(".secret", file.file_name().unwrap().to_str().unwrap());
        assert!(hidden(file.file_name().unwrap()));

        file.close().unwrap()
    }

    #[test]
    fn test_hidden_directory_is_detected() {
        let dir = assert_fs::TempDir::new().unwrap();
        let file = dir.child(".secret/foo.txt");
        file.touch().unwrap();
        let hidden_dir = file.parent().unwrap();

        assert_eq!(".secret", hidden_dir.file_name().unwrap().to_str().unwrap());
        assert!(hidden(hidden_dir.file_name().unwrap()));

        dir.close().unwrap();
    }

    #[test]
    fn test_visible_file_is_detected() {
        let file = assert_fs::NamedTempFile::new("visible.txt").unwrap();

        assert_eq!("visible.txt", file.file_name().unwrap().to_str().unwrap());
        assert!(!hidden(file.file_name().unwrap()));

        file.close().unwrap()
    }

    #[test]
    fn test_visible_directory_is_detected() {
        let dir = assert_fs::TempDir::new().unwrap();
        let file = dir.child("visible/foo.txt");
        file.touch().unwrap();
        let visible_dir = file.parent().unwrap();

        assert_eq!(
            "visible",
            visible_dir.file_name().unwrap().to_str().unwrap()
        );
        assert!(!hidden(visible_dir.file_name().unwrap()));

        dir.close().unwrap();
    }

    #[test]
    fn test_git_repository_is_detected() {
        let dir = assert_fs::TempDir::new().unwrap();
        let file = dir.child("repo/.git/foo");
        file.touch().unwrap();
        let git_dir = file.parent().unwrap().parent().unwrap();

        assert_eq!("repo", git_dir.file_name().unwrap());
        assert!(is_git_repository(git_dir))
    }

    #[test]
    fn test_git_repository_with_multiple_subdirs_is_detected() {
        let dir = assert_fs::TempDir::new().unwrap();
        let mut file = dir.child("repo/.git/foo");
        file.touch().unwrap();
        file = dir.child("repo/src/foo.rs");
        file.touch().unwrap();

        let git_dir = file.parent().unwrap().parent().unwrap();

        assert_eq!("repo", git_dir.file_name().unwrap());
        assert!(is_git_repository(git_dir))
    }

    #[test]
    fn test_not_repository_is_detected() {
        let dir = assert_fs::TempDir::new().unwrap();
        let file = dir.child("not_repo/.fit/foo");
        file.touch().unwrap();
        let non_git_dir = file.parent().unwrap().parent().unwrap();

        assert_eq!("not_repo", non_git_dir.file_name().unwrap());
        assert!(!is_git_repository(non_git_dir))
    }

    #[test]
    fn test_hidden_git_repository_is_detected() {
        let dir = assert_fs::TempDir::new().unwrap();
        let file = dir.child(".hidden/.git/foo");
        file.touch().unwrap();
        let hidden_git_dir = file.parent().unwrap().parent().unwrap();

        assert_eq!(".hidden", hidden_git_dir.file_name().unwrap());
        assert!(!is_git_repository(hidden_git_dir))
    }
    #[test]
    fn test_non_existent_git_dir_is_detected() {
        let dir = assert_fs::TempDir::new().unwrap();
        let file = dir.child(".hidden/.git/foo");
        let non_existent_git_dir = file.parent().unwrap().parent().unwrap();

        assert!(!non_existent_git_dir.exists());
        assert!(!is_git_repository(non_existent_git_dir));
    }

    #[test]
    fn test_empty_dir_is_detected() {
        let dir = assert_fs::TempDir::new().unwrap();

        assert!(!is_git_repository(dir.path()));
    }
}
