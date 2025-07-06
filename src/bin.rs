use ged_io::Gedcom;
use ged_io::GedcomError;
use std::env;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Debug)]
enum CliError {
    Io(std::io::Error),
    Gedcom(GedcomError),
    Usage(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::Io(err) => write!(f, "IO error: {err}"),
            CliError::Gedcom(err) => write!(f, "Gedcom error: {err}"),
            CliError::Usage(msg) => write!(f, "Usage error: {msg}"),
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError::Io(err)
    }
}

impl From<GedcomError> for CliError {
    fn from(err: GedcomError) -> Self {
        CliError::Gedcom(err)
    }
}

fn main() {
    match run() {
        Ok(_) => {
            println!("Parsing complete!");
            process::exit(0);
        }
        Err(e) => {
            let exit_code = match &e {
                CliError::Io(_) => 1,
                CliError::Gedcom(_) => 2,
                CliError::Usage(_) => 3,
            };
            eprintln!("Error: {e}");
            process::exit(exit_code);
        }
    }
}

fn run() -> Result<(), CliError> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => return Err(CliError::Usage("Missing filename.".to_string())),
        s if s > 2 => {
            return Err(CliError::Usage(format!(
                "Found more args than expected: {:?}",
                &args[1..]
            )))
        }
        _ => (),
    };

    let filename = &args[1];

    if filename == "--help" || filename == "-h" {
        println!("Usage: parse_gedcom ./path/to/gedcom.ged");
        return Ok(());
    }

    let contents = read_relative(filename)?;
    let mut doc = Gedcom::new(contents.chars())?;
    let data = doc.parse_data()?;

    data.stats();

    Ok(())
}

fn read_relative(path: &str) -> Result<String, std::io::Error> {
    let path_buf: PathBuf = PathBuf::from(path);
    let absolute_path: PathBuf = fs::canonicalize(path_buf)?;
    fs::read_to_string(absolute_path)
}
