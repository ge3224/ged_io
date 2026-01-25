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
    individual_lastname: bool,
    individual_firstname: bool,
    help: bool,
}

fn print_help() {
    println!(
        "ged_io - GEDCOM inspection tool\n\
\n\
USAGE:\n\
  ged_io <file.ged>\n\
  ged_io --individual <XREF> <file.ged>\n\
  ged_io --individual-lastname <file.ged>\n\
  ged_io --individual-firstname <file.ged>\n\
\n\
OPTIONS:\n\
  -h, --help                  Print this help\n\
  --individual <XREF>          Display a single individual (e.g. @I1@)\n\
  --individual-lastname        List all individuals as last name only\n\
  --individual-firstname       List all individuals as first name only\n\
\n\
NOTES:\n\
  If both --individual-lastname and --individual-firstname are set,\n\
  individuals are listed as \"<Last> <First>\".\n"
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
                out.individual_lastname = true;
                i += 1;
            }
            "--individual-firstname" => {
                out.individual_firstname = true;
                i += 1;
            }
            other if other.starts_with('-') => {
                return Err(CliError::Usage(format!("Unknown option: {other}")));
            }
            value => {
                if out.filename.is_some() {
                    return Err(CliError::Usage(format!(
                        "Found more args than expected: {:?}",
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

    if args.individual_lastname || args.individual_firstname {
        for individual in &data.individuals {
            let display_name = individual
                .name
                .as_ref()
                .map(|n| n.to_string())
                .unwrap_or_else(|| "(Unknown)".to_string());

            let (first, last) = extract_first_last_name(&display_name);

            let out = match (args.individual_lastname, args.individual_firstname) {
                (true, true) => match (last, first) {
                    (Some(l), Some(f)) => format!("{l} {f}"),
                    (Some(l), None) => l,
                    (None, Some(f)) => f,
                    (None, None) => "(Unknown)".to_string(),
                },
                (true, false) => last.unwrap_or_else(|| "(Unknown)".to_string()),
                (false, true) => first.unwrap_or_else(|| "(Unknown)".to_string()),
                (false, false) => unreachable!(),
            };

            if let Some(ref xref) = individual.xref {
                println!("{xref} {out}");
            } else {
                println!("{out}");
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
