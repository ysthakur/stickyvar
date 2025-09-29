use std::{path::PathBuf, time::SystemTime};

use clap::{Parser, Subcommand, ValueEnum};
use rusqlite::Connection;

mod nu;
mod sh;

static DB_FILE_VAR: &str = "STICKY_VAR_DB";
/// Name of directory in which to store sticky variable database
static STICKYVAR_DIR: &str = "stickyvar";
static DB_FILE_NAME: &str = "sticky-var.db";
static TABLE: &str = "vars";
static NAME_COL: &str = "name";
static VALUE_COL: &str = "value";
static TIME_COL: &str = "time";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// The only subcommand you should need to run here is init
#[derive(Subcommand)]
enum Commands {
    /// Generate shell-specific code to set up functions or whatever
    Init {
        /// The shell/shell family to generate code for
        shell: ShellFamily,
    },
    /// Add a sticky variable to the database
    Set {
        /// The name of the variable
        name: String,
        /// The value to set the variable to
        value: String,
    },
    /// List all sticky variables and their values
    List,
    /// Get the value of the given sticky variable
    Get { name: String },
    /// Get the values of all sticky variables in the format '{varname}={url encoded value}'
    GetAll,
    /// Get the path to the database used to store variables
    /// (to change this, set STICKY_VAR_DB to, say, '~/myvars.sqlite')
    DbPath,
    /// Decode a value provided by the `get` subcommand. This is only included
    /// because we need a way to decode values on every system.
    DecodeValue {
        /// URL-encoded form of a UTF-8 string
        value: String,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum ShellFamily {
    /// sh and POSIX-y shells such as Bash and Zsh
    Sh,
    /// Nushell (https://nushell.sh)
    Nu,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { shell } => {
            // When generating the init script, rather than assuming this program is
            // installed as "stickyvar," we use the path used to invoke the current
            // binary
            // TODO if it's just the name of a binary, resolve using $PATH?
            let my_path = std::env::args()
                .next()
                .expect("Expected to be run with 1 argument (the program name)");
            let my_path = PathBuf::from(&my_path);
            let my_path = my_path.canonicalize().unwrap_or(my_path);
            let my_path = my_path.to_string_lossy().into_owned();
            let code = match shell {
                ShellFamily::Sh => sh::init(&my_path),
                ShellFamily::Nu => nu::init(&my_path),
            };
            print!("{}", code);
        }
        Commands::Set { name, value } => {
            let db = open_conn();
            let time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            db.execute(
                &format!(
                    "REPLACE INTO {TABLE} ({NAME_COL}, {VALUE_COL}, {TIME_COL}) VALUES (?1, ?2, ?3)"
                ),
                (name, value, time),
            )
            .unwrap_or_exit(|e| eprintln!("Insert failed due to: {}", e));
        }
        Commands::Get { name } => {
            let db = open_conn();
            let value: String = db
                .query_one(
                    &format!("SELECT {VALUE_COL} FROM {TABLE} WHERE {NAME_COL} = ?"),
                    (&name,),
                    |row| row.get(0),
                )
                .unwrap_or_exit(|e| eprintln!("Query failed due to: {}", e));
            println!("{}", value);
        }
        Commands::GetAll => {
            let db = open_conn();
            let mut stmt = db
                .prepare(&format!("SELECT {NAME_COL}, {VALUE_COL} FROM {TABLE}"))
                .unwrap_or_exit(|e| eprintln!("Couldn't construct query: {}", e));
            let vars = stmt
                .query_map((), |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .unwrap_or_exit(|e| eprintln!("Query failed due to: {}", e));
            for var in vars {
                let (name, value) = var.unwrap();
                println!("{}={}", name, urlencoding::encode(&value))
            }
        }
        Commands::List => {
            let db = open_conn();
            let mut stmt = db
                .prepare(&format!("SELECT {NAME_COL}, {VALUE_COL} FROM {TABLE}"))
                .unwrap_or_exit(|e| eprintln!("Couldn't construct query: {}", e));
            let vars = stmt
                .query_map((), |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .unwrap_or_exit(|e| eprintln!("Query failed due to: {}", e));
            for var in vars {
                let (name, value) = var.unwrap();
                // Make the value a single-line string
                let value = value
                    .replace('\\', r#"\\"#)
                    .replace('"', r#"\""#)
                    .replace('\n', r#"\n"#)
                    .replace('\r', r#"\r"#);
                println!("{name}=\"{value}\"");
            }
        }
        Commands::DbPath => {
            println!("{}", get_db_path().display());
        }
        Commands::DecodeValue { value } => {
            println!(
                "{}",
                urlencoding::decode(&value).expect("Expected valid URL-encoded UTF-8 string")
            );
        }
    }
}

fn get_db_path() -> PathBuf {
    if let Ok(db_path) = std::env::var(DB_FILE_VAR) {
        PathBuf::from(db_path)
    } else if let Some(dir) = dirs::state_dir().or_else(dirs::data_local_dir) {
        let sv_dir = dir.join(STICKYVAR_DIR);
        if !sv_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&sv_dir) {
                eprintln!(
                    "Couldn't create directory for database file {}: {}",
                    sv_dir.display(),
                    e
                );
            }
        }
        sv_dir.join(DB_FILE_NAME)
    } else {
        eprintln!(
            "Couldn't figure out where to look for stored sticky vars. Please set {} to the path to the sticky variable database",
            DB_FILE_VAR
        );
        std::process::exit(1);
    }
}

fn open_conn() -> Connection {
    let db_path = get_db_path();
    let db = Connection::open(&db_path).unwrap_or_exit(|e| {
        eprintln!("Couldn't connect to {}", db_path.display());
        eprintln!("Error: {}", e);
    });
    db.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {TABLE} (
                {NAME_COL} TEXT NOT NULL UNIQUE,
                {VALUE_COL} TEXT NOT NULL,
                {TIME_COL} INTEGER
            )"
        ),
        (),
    )
    .unwrap_or_exit(|e| eprintln!("Couldn't create table {TABLE}: {e}"));
    db
}

trait UnwrapOrExit<T, E> {
    fn unwrap_or_exit<F>(self, handle_err: F) -> T
    where
        F: FnOnce(E);
}
impl<T, E> UnwrapOrExit<T, E> for rusqlite::Result<T, E> {
    fn unwrap_or_exit<F>(self, handle_err: F) -> T
    where
        F: FnOnce(E),
    {
        match self {
            Ok(res) => res,
            Err(err) => {
                handle_err(err);
                std::process::exit(1);
            }
        }
    }
}
