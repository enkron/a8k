use chrono::prelude::*;
use clap::Parser;
use env_logger;
use log;
use std::io::Write;

//#[macro_use]
//extern crate rocket;
use rocket::response;

mod sniffer;
use sniffer::Config;

#[tokio::main]
async fn main() -> sniffer::Result<()> {
    init_logger();

    rocket::build()
        .mount("/", rocket::routes![index, buildservers_list])
        .mount("/stats", rocket::routes![buildserver_stats])
        .launch()
        .await?;

    Ok(())
}

fn init_logger() {
    let now = Utc::now().to_rfc3339();
    env_logger::Builder::from_default_env()
        .format(move |buf, rec| writeln!(buf, "{} [{}] - {}", now, rec.level(), rec.args()))
        .init();
}

#[rocket::get("/")]
async fn index() -> response::Redirect {
    response::Redirect::to(rocket::uri!(buildservers_list))
}

#[rocket::get("/all")]
async fn buildservers_list() -> String {
    let cli = sniffer::Cli::parse();
    let cfg = Config::new(cli).unwrap_or_else(|e| {
        log::error!("problem with configuration file: {}", e);
        std::process::exit(1);
    });
    //println!("buildservers: {:?}", cfg.buildservers);
    format!("{:#?}", cfg.buildservers)
}

#[rocket::get("/<buildserver>")]
async fn buildserver_stats(buildserver: &str) -> String {
    let srv_info = sniffer::digdeep(buildserver).await.unwrap_or_else(|e| {
        log::error!("an error occured: {}", e);
        std::process::exit(1);
    });

    srv_info
}

//#[rocket::async_trait]
//impl<'r> FromRequest<'r> for Config {
//    type Error = Box<dyn std::error::Error + Send + Sync>;
//
//    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
//        unimplemented!()
//    }
//}
