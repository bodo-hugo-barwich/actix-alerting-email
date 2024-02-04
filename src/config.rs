/*
* @author Bodo (Hugo) Barwich
* @version 2024-01-31
* @package Grafana Alerting
* @subpackage Configuration Loader

* This Module defines functions to load the application configuration
*
*---------------------------------
* Requirements:
*/

extern crate serde;
extern crate serde_yaml;

use serde_derive::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

const CONFIG_FILE: &'static str = ".env";

//==============================================================================
// Structure SMTPConfig Declaration

/// Structure for the SMTP Configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct SMTPConfig {
    pub host: String,
    pub port: String,
    pub login: String,
    pub password: String,
    pub full_name: String,
    pub email_address: String
}

//==============================================================================
// Structure AppConfig Declaration

/// Structure for the Application Configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub component: String,
    pub project: String,
    pub web_root: String,
    pub main_directory: String,
    pub config_file: String,
    pub mail_worker: u16,
    pub smtp: SMTPConfig,
}

//==============================================================================
// Structure AppConfig Implementation

impl Default for SMTPConfig {
    /*----------------------------------------------------------------------------
     * Default Constructor
     */

    fn default() -> Self {
        SMTPConfig::new()
    }
}

impl SMTPConfig {
    /*----------------------------------------------------------------------------
     * Constructors
     */

    pub fn new() -> SMTPConfig {
        SMTPConfig {
            host: String::new(),
            port: String::new(),
            login: String::new(),
            password: String::new(),
            full_name: String::new(),
            email_address: String::new()
        }
    }
}

impl Clone for SMTPConfig {
    /*----------------------------------------------------------------------------
     * Administrative Methods
     */

    fn clone(&self) -> SMTPConfig {
        SMTPConfig {
            host: self.host.clone(),
            port: self.port.clone(),
            login: self.login.clone(),
            password: self.password.clone(),
            full_name: self.full_name.clone(),
            email_address: self.email_address.clone()
        }
    }
}

//==============================================================================
// Structure AppConfig Implementation

impl Default for AppConfig {
    /*----------------------------------------------------------------------------
     * Default Constructor
     */

    fn default() -> Self {
        AppConfig::new()
    }
}

impl AppConfig {
    /*----------------------------------------------------------------------------
     * Constructors
     */

    pub fn new() -> AppConfig {
        AppConfig {
            component: String::from("unknown"),
            project: String::from("Actix Alerting Email"),
            web_root: String::from("/"),
            main_directory: String::new(),
            config_file: String::new(),
            mail_worker: 2,
            smtp: SMTPConfig::new(),
        }
    }

    pub fn from_yaml() -> AppConfig {
        let config_yaml = "---
component: 'unknown'
project: 'Actix Alerting Email'
web_root: '/'
main_directory: ''
config file: ''
mail_worker: 2
smtp:
  host: ''
  port: ''
  login: ''
  password: ''
  full_name: ''
  email_address: ''
";
        // Deserialize it back to a Rust type.
        let config: AppConfig = match serde_yaml::from_str(&config_yaml) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Default Config could not be parsed: {:?}", e);
                AppConfig::new()
            }
        };

        config
    }

    pub fn from_file() -> AppConfig {
        let mut config: Option<AppConfig> = None;

        match try_find_file(Path::new(&*CONFIG_FILE)) {
            Ok(file) => {
                config = match try_config_from_path(&file) {
                    Ok(cfg) => Some(cfg),
                    Err(e) => {
                        eprintln!("Config File {:?}: File could not be read: {:?}", file, e);
                        None
                    }
                };
            }
            Err(e) => {
                eprintln!(
                    "Config File '{}': File could not be found: {:?}",
                    CONFIG_FILE, e
                );
            }
        };

        if config.is_none() {
            eprintln!("Falling back to default configuration ...");
            config = Some(AppConfig::from_yaml());
        }

        match config {
            Some(cfg) => cfg,
            None => AppConfig::new(),
        }
    }
}

impl Clone for AppConfig {
    /*----------------------------------------------------------------------------
     * Administrative Methods
     */

    fn clone(&self) -> AppConfig {
        AppConfig {
            component: self.component.clone(),
            project: self.project.clone(),
            web_root: self.web_root.clone(),
            main_directory: self.main_directory.clone(),
            config_file: self.config_file.clone(),
            mail_worker: self.mail_worker,
            smtp: self.smtp.clone(),
        }
    }
}

//==============================================================================
// Auxiliary Functions

fn try_find_file(file: &Path) -> Result<PathBuf, Error> {
    let work_dir = std::env::current_dir().map_err(|e| {
        Error::new(
            ErrorKind::NotFound,
            format!(
                "Working Directory: find directory failed with Error: {:?}",
                e
            ),
        )
    })?;
    println!("Working Directory: '{}'", work_dir.display());

    let mut search_dir: Option<&Path> = Some(Path::new(work_dir.as_path()));
    let mut find_file: Option<PathBuf> = None;

    while search_dir.is_some() && find_file.is_none() {
        if let Some(d) = search_dir {
            println!("Search Directory: '{}'", d.display());

            let mut search_file = PathBuf::from(d);

            search_file.push(file);

            if search_file.exists() {
                find_file = Some(search_file);
            } else {
                // Continue searching in Parent Directory
                search_dir = d.parent();
            }
        } //if let Some(d) = search_dir
    } //while search_dir.is_some() && find_file.is_none()

    if let Some(f) = find_file {
        Ok(f)
    } else {
        //Config File does not exist
        Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "Working Directory '{}' - Config File {:?}: file does not exist in any parent directory!",
                work_dir.display(),
                file.file_name()
            ),
        ))
    } //if let Some(f) = find_file
}

fn try_config_from_path(file: &Path) -> Result<AppConfig, Error> {
    let config_yaml = fs::read_to_string(file).map_err(|e| {
        Error::new(
            ErrorKind::NotFound,
            format!(
                "Config File {:?}: read file failed with Error: '{:?}'",
                file, e
            ),
        )
    })?;
    let config: AppConfig = serde_yaml::from_str(&config_yaml).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!(
                "Config File {:?}: parse file failed with Error: '{:?}'",
                file.file_name(),
                e
            ),
        )
    })?;

    Ok(config)
}

#[allow(dead_code)]
fn find_path_parent(current: &Path, name: &str) -> Option<PathBuf> {
    let mut odir = None;

    let osearch = Some(OsStr::new(name));

    for p in current.ancestors() {
        if odir.is_none() && p.is_dir() && p.file_name() == osearch {
            odir = Some(p);
        }
    }

    if let Some(d) = odir {
        odir = d.parent();
    }

    match odir {
        Some(d) => Some(PathBuf::from(d)),
        None => None,
    }
}
