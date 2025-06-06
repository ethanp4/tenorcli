use clap::{Parser, ValueEnum};

use serde::Deserialize;
use reqwest::Error;
use reqwest::header::USER_AGENT;

#[derive(Debug)]
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
	/// Desired number of results
	#[arg(short, default_value_t = 10)]
	num: u16,
	
	/// Automatically copy a random link to the clipboard selected from the value of -n
	#[arg(short, default_value_t = false)]
	copy: bool,

	/// Automatically download a random gif to the Pictures library selected from the value of -n
	#[arg(short, default_value_t = false)]
	download: bool,
	
	/// URL Type
	#[arg(short, value_enum, default_value_t = URLType::Page)]
	type_url: URLType,

	/// A search term to query the tenor api. With no options a list of 10 links will be printed to the console
	search_term: Vec<String>,
}

#[derive(Debug)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum URLType {
	/// Direct .gif link
	Gif,
	/// Page url to the tenor link, suitable for discord (will display full resolution)
	Page
}

#[tokio::main]
async fn main () -> Result<(), Error> {
	let args = Cli::parse();

	println!("all args: {:#?}", args);
	Ok(())
}