use ged_io::Gedcom;
use ged_io::GedcomError;
use std::env;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Debug, Default)]
struct CliArgs {
    filename: Option<String>,
    individual_xref: Option<String>,
    individual_lastname: Option<String>,
    individual_firstname: Option<String>,
    help: bool,
}

fn print_help() {
    println!(
        "ged_io - GEDCOM inspection tool\n\
\n\
USAGE:\n\
  ged_io <file.ged>\n\
  ged_io --individual <XREF> <file.ged>\n\
  ged_io --individual-lastname <LASTNAME> <file.ged>\n\
  ged_io --individual-firstname <FIRSTNAME> <file.ged>\n\
\n\
OPTIONS:\n\
  -h, --help                        Print this help\n\
  --individual <XREF>               Display a single individual (e.g. @I1@)\n\
  --individual-lastname <LASTNAME>  Filter individuals by last name (case-insensitive)\n\
  --individual-firstname <FIRSTNAME> Filter individuals by first name (case-insensitive)\n\
\n\
NOTES:\n\
  If both --individual-lastname and --individual-firstname are set,\n\
  individuals matching BOTH filters are listed.\n"
    );
}

fn parse_args(argv: &[String]) -> Result<CliArgs, CliError> {
    let mut out = CliArgs::default();

    let mut i = 1;
    while i < argv.len() {
        match argv[i].as_str() {
            "-h" | "--help" => {
                out.help = true;
                i += 1;
            }
            "--individual" => {
                let xref = argv
                    .get(i + 1)
                    .ok_or_else(|| CliError::Usage("--individual expects an XREF".to_string()))?;
                out.individual_xref = Some(xref.clone());
                i += 2;
            }
            "--individual-lastname" => {
                let val = argv.get(i + 1).ok_or_else(|| {
                    CliError::Usage("--individual-lastname expects a LASTNAME".to_string())
                })?;
                out.individual_lastname = Some(val.clone());
                i += 2;
            }
            "--individual-firstname" => {
                let val = argv.get(i + 1).ok_or_else(|| {
                    CliError::Usage("--individual-firstname expects a FIRSTNAME".to_string())
                })?;
                out.individual_firstname = Some(val.clone());
                i += 2;
            }
            other if other.starts_with('-') => {
                return Err(CliError::Usage(format!("Unknown option: {other}")));
            }
            value => {
                if out.filename.is_some() {
                    return Err(CliError::Usage(format!(
                        "Found more args than expected: {:?}\n\
\
Hint: this tool expects exactly one .ged file path (quote it if it contains spaces), for example:\n\
  ged_io --individual-lastname \"/path/with spaces/family.ged\"",
                        &argv[1..]
                    )));
                }
                out.filename = Some(value.to_string());
                i += 1;
            }
        }
    }

    Ok(out)
}

fn extract_first_last_name(display_name: &str) -> (Option<String>, Option<String>) {
    let cleaned = display_name.replace('/', " ");
    let mut parts: Vec<&str> = cleaned.split_whitespace().collect();
    if parts.is_empty() {
        return (None, None);
    }

    // Heuristic: GEDCOM names are "First /Last/". Our Display implementation already
    // removes slashes, but fixtures may still include them. We treat last token as last name.
    let last = parts.pop().map(|s| s.to_string());
    let first = if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    };

    (first, last)
}

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
    let argv: Vec<String> = env::args().collect();
    let args = parse_args(&argv)?;

    if args.help {
        print_help();
        process::exit(0);
    }

    let filename = args
        .filename
        .as_deref()
        .ok_or_else(|| CliError::Usage("Missing filename.".to_string()))?;

    let contents = read_relative(filename)?;
    let mut doc = Gedcom::new(contents.chars())?;
    let data = doc.parse_data()?;

    if let Some(xref) = args.individual_xref.as_deref() {
        if let Some(individual) = data
            .individuals
            .iter()
            .find(|i| i.xref.as_deref() == Some(xref))
        {
            println!("{individual}");
            return Ok(());
        }
        return Err(CliError::Usage(format!("Individual not found: {xref}")));
    }

    if args.individual_lastname.is_some() || args.individual_firstname.is_some() {
        let filter_last = args
            .individual_lastname
            .as_deref()
            .map(|s| s.to_lowercase());
        let filter_first = args
            .individual_firstname
            .as_deref()
            .map(|s| s.to_lowercase());

        for individual in &data.individuals {
            let display_name = individual
                .name
                .as_ref()
                .map(|n| n.to_string())
                .unwrap_or_else(|| "(Unknown)".to_string());

            let (first, last) = extract_first_last_name(&display_name);

            let first_lower = first.as_deref().map(|s| s.to_lowercase());
            let last_lower = last.as_deref().map(|s| s.to_lowercase());

            let matches_last = filter_last
                .as_ref()
                .map(|f| last_lower.as_ref().map(|l| l.contains(f)).unwrap_or(false))
                .unwrap_or(true);

            let matches_first = filter_first
                .as_ref()
                .map(|f| {
                    first_lower
                        .as_ref()
                        .map(|fi| fi.contains(f))
                        .unwrap_or(false)
                })
                .unwrap_or(true);

            if matches_last && matches_first {
                println!("{individual}");
            }
        }

        return Ok(());
    }

    data.stats();

    Ok(())
}

fn read_relative(path: &str) -> Result<String, std::io::Error> {
    let path_buf: PathBuf = PathBuf::from(path);
    let absolute_path: PathBuf = fs::canonicalize(path_buf)?;
    fs::read_to_string(absolute_path)
}
