use std::{path::PathBuf, process::exit};

use clap::{Parser, Subcommand};
use rusqlite::Connection;

static DB_FILE_VAR: &str = "STICKY_VAR_DB";
static DB_FILE_NAME: &str = "sticky-var.db";
static TABLE: &str = "vars";
static NAME_COL: &str = "name";
static VALUE_COL: &str = "value";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Set a sticky variable. This will also set it as an environment variable
    Set {
        /// The name of the variable
        name: String,
        /// The value to set the variable to
        value: String,
    },
    /// Load a sticky variable or all sticky variables and set it/them as (an)
    /// environment variable(s) in the current process
    Load {
        /// The name of the variable (if not given, all variables are loaded)
        name: Option<String>,
    },
    /// List all sticky variables and their values
    List,
}

fn main() -> rusqlite::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Set { name, value } => {
            // Yay, my first justified unsafe
            unsafe {
                std::env::set_var(&name, &value);
            }
            let db = open_conn()?;
            db.execute(
                &format!("REPLACE INTO {TABLE} ({NAME_COL}, {VALUE_COL}) VALUES (?1, ?2)"),
                (name, value),
            )?;
        }
        Commands::Load { name } => {
            let db = open_conn()?;
            if let Some(name) = name {
                let value: String = db.query_one(
                    &format!("SELECT {VALUE_COL} FROM {TABLE} WHERE {NAME_COL} = ?"),
                    (&name,),
                    |row| row.get(0),
                )?;
                unsafe {
                    std::env::set_var(&name, &value);
                }
                println!("{value}");
            } else {
                let mut stmt =
                    db.prepare(&format!("SELECT {NAME_COL}, {VALUE_COL} FROM {TABLE}"))?;
                let vars = stmt.query_map((), |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?;
                for var in vars {
                    let (name, value) = var?;
                    unsafe {
                        std::env::set_var(&name, &value);
                    }
                    println!("{name} = {value}");
                }
            }
        }
        Commands::List => {
            let db = open_conn()?;
            let mut stmt = db.prepare(&format!("SELECT {NAME_COL}, {VALUE_COL} FROM {TABLE}"))?;
            let vars = stmt.query_map((), |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            for var in vars {
                let (name, value) = var?;
                println!("{name} = {value}");
            }
        }
    }

    Ok(())
}

fn open_conn() -> rusqlite::Result<Connection> {
    let Some(db_path) = std::env::var(DB_FILE_VAR)
        .ok()
        .map(|path| PathBuf::from(path))
        .or_else(|| {
            dirs::state_dir()
                .or_else(|| dirs::data_local_dir())
                .map(|dir| dir.join(DB_FILE_NAME))
        })
        .or_else(|| dirs::data_local_dir())
    else {
        eprintln!(
            "Couldn't figure out where to look for stored sticky vars. Please set {} to the path to the sticky variable database",
            DB_FILE_VAR
        );
        exit(1);
    };
    let db = Connection::open(db_path)?;
    db.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {TABLE} (
                {NAME_COL} TEXT NOT NULL UNIQUE,
                {VALUE_COL} TEXT NOT NULL
            )"
        ),
        (),
    )?;
    Ok(db)
}
