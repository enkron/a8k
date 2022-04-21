use clap::Parser;
use openssh::{KnownHosts, Session};
use serde::Deserialize;
use std::{env, fs, path::Path};
use toml;

mod consts;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn digdeep(buildserver: &str) -> Result<String> {
    let session = Session::connect(format!("ssh://{}:22", buildserver), KnownHosts::Strict).await?;

    let host = session.command("uname").arg("-n").output().await?;
    let free = session.command("free").arg("-h").output().await?;
    let du = session
        .command("du")
        .arg("-d")
        .arg("1")
        .arg("-h")
        .arg("/home/")
        .arg("2>/dev/null")
        .output()
        .await?;

    session.close().await?;

    Ok(format!(
        "hostname: {}{}\ndisk utilization per user:\n{}",
        String::from_utf8(host.stdout)?,
        String::from_utf8(free.stdout)?,
        String::from_utf8(du.stdout)?,
    ))
}

#[derive(Deserialize, Debug)]
pub struct Config {
    // Config struct represents congig.toml file structure
    pub buildservers: Vec<String>,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    //#[clap(short, long)]
    /// configuration file [default: $HOME/.config/a8k/config.toml]
    pub cfg: Option<String>,
}

impl Config {
    pub fn new(cli: Cli) -> Result<Self> {
        let config_str = match cli.cfg {
            Some(v) => {
                if Path::new(&v).ends_with(consts::CONFIG) {
                    // read configuration file into a string
                    fs::read_to_string(&v)?
                } else if Path::new(&v).exists() && !Path::new(&v).ends_with(consts::CONFIG) {
                    return Err(Box::from("incorrect config file"));
                } else {
                    return Err(Box::from(format!("no such file - {}", v)));
                }
            }

            None => fs::read_to_string(
                Path::new(env::var("HOME")?.as_str())
                    .join(".config")
                    .join("a8k")
                    .join(consts::CONFIG)
                    .to_str()
                    .unwrap(),
            )?,
        };

        // get .toml structure from string
        let cfg: Config = toml::from_str(&config_str)?;

        Ok(Config {
            buildservers: cfg.buildservers,
        })
    }
}
