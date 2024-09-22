use clap::{Parser, Subcommand};
use colored::Colorize;
pub use password::Password;
use rusqlite::{params, Connection, Error};
use std::process;

mod password;

#[derive(Debug, Parser)]
#[command(version,about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn run() -> Cli {
        Cli::parse()
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(about = "Add password")]
    Add {
        service: String,
        login: String,
        password: String,
    },

    #[clap(about = "Show password")]
    Print {
        #[arg(help = "ID of the password to display")]
        id: Option<u16>,

        #[arg(short, long, help = "Display all passwords", default_value_t = false)]
        all: bool,
    },

    #[clap(about = "Change password")]
    Modify(Password),

    #[clap(about = "Remove password")]
    Remove {
        #[arg(help = "ID")]
        id: Option<u16>,

        #[arg(short, long, default_value_t = false, help = "Remove all passwords")]
        all: bool,
    },
}

pub fn create_table(file: &Connection) -> Result<(), Error> {
    file.execute(
        "CREATE TABLE IF NOT EXISTS password (
            id INTEGER PRIMARY KEY,
            service TEXT NOT NULL,
            login TEXT NOT NULL,
            password TEXT NOT NULL
        )",
        (), // Создание пустой таблицы
    )?;
    Ok(())
}

pub fn insert(file: &Connection, service: &str, login: &str, password: &str) -> Result<(), Error> {
    file.execute(
        "INSERT INTO password (service, login, password) VALUES (?1, ?2, ?3)",
        params![service, login, password],
    )?;
    Ok(())
}

pub fn search_by_id(conn: &Connection, id: u16) -> Option<Password> {
    let query = "SELECT id, service, login, password FROM password WHERE id = ?1".to_string();

    let mut stmt = conn.prepare(&query).unwrap();
    let user_iter = stmt
        .query_map([id], |row| {
            Ok(Password {
                id: row.get(0).unwrap(),
                service: row.get(1).unwrap(),
                login: row.get(2).unwrap(),
                password: row.get(3).unwrap(),
            })
        })
        .unwrap();

    let users: Result<Vec<Password>, Error> = user_iter.collect();
    let users = users.unwrap();

    if users.is_empty() {
        return None;
    }

    Some(users[0].clone())
}

pub fn search(conn: &Connection) -> Option<Vec<Password>> {
    let query = "SELECT * FROM password".to_string();

    let mut stmt = conn.prepare(&query).unwrap();
    let user_iter = stmt
        .query_map([], |row| {
            Ok(Password {
                id: row.get(0).unwrap(),
                service: row.get(1).unwrap(),
                login: row.get(2).unwrap(),
                password: row.get(3).unwrap(),
            })
        })
        .unwrap();

    let users: Result<Vec<Password>, Error> = user_iter.collect();

    let users = users.unwrap_or_else(|_| {
        panic!(
            "{}",
            "Couldn't get the password".to_string().red().to_string()
        )
    });

    if users.is_empty() {
        return None;
    }

    Some(users)
}

pub fn print(conn: &Connection, all: bool, id: Option<u16>) {
    if !all {
        if let Some(id) = id {
            let user = match search_by_id(conn, id) {
                Some(p) => p,
                None => {
                    eprintln!("{}", "The password could not be found".to_string().red());
                    process::exit(1);
                }
            };
            // Выводим результаты
            println!(
                "ID: {}, Service: {}, Login: {}, Password: {}",
                user.id, user.service, user.login, user.password
            );
        }
    } else {
        let users = match search(conn) {
            Some(p) => p,
            None => {
                eprintln!("{}", "There are no passwords for output".to_string().red());
                process::exit(1);
            }
        };
        // Выводим результаты
        users.iter().for_each(|e| {
            println!(
                "ID: {}, Service: {}, Login: {}, Password: {}",
                e.id, e.service, e.login, e.password
            )
        });
    }
}

pub fn remove(conn: &Connection, id: Option<u16>, all: bool) {
    if !all {
        if let Some(id) = id {
            let mut passwords = search(conn).unwrap();

            passwords.retain(|e| {
                if e.id == id {
                    println!(
                        "Remove: {} {} {} {}",
                        e.id.to_string().red(),
                        e.service.to_string().red(),
                        e.login.to_string().red(),
                        e.password.to_string().red()
                    );
                    false
                } else {
                    true
                }
            });

            conn.execute("DELETE FROM password", []).unwrap();

            for i in passwords {
                match insert(conn, &i.service, &i.login, &i.password) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("{}", e.to_string().red());
                        process::exit(1);
                    }
                }
            }
        }
    } else {
        conn.execute("DELETE FROM password", []).unwrap();
        match create_table(conn) {
            Ok(()) => println!("Passwords have been deleted"),
            Err(e) => {
                eprintln!("{}", e.to_string().red());
                process::exit(1);
            }
        }
    }
}