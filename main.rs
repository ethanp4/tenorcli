use std::path::Path;
use std::time::SystemTime;
use std::{env, io};
use std::fs::File;
use std::io::{stdin, Read, Write};
use std::process::{self, Command, Stdio};

use clap::{Parser, ValueEnum};

use colored::Colorize;
use rand::{random_range, Rng};
use serde::Deserialize;
use reqwest::Error;
use reqwest::header::USER_AGENT;
use dotenv::dotenv;

#[derive(Deserialize, Debug)]
struct ApiResult {
	results: Vec<Gif>
}

#[derive(Deserialize, Debug)]
struct Gif {
	id: String,
	created: f32,
	/// Tenor's ai generated description
	content_description: String,
	/// Page url
	itemurl: String,
	/// .gif url
	url: String,
	/// User supplied tags
	tags: Vec<String>,
	/// "Media format" containing data about each file format of that item
	media_formats: MediaFormats
}

#[derive(Deserialize, Debug)]
struct MediaFormats {
	mediumgif: MediaInfo,
	gif: MediaInfo
}

#[derive(Deserialize, Debug)]
struct MediaInfo {
	url: String,
	size: u32
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
	/// Number of items to list
	#[arg(short, default_value_t = 10, value_parser = clap::value_parser!(u8).range(1..=50))]
	limit: u8,
	
	/// Automatically copy a random link to the clipboard selected from the value of -l
	#[arg(short, default_value_t = false)]
	copy: bool,

	/// Automatically save a random gif to the Pictures library selected from the value of -l
	#[arg(short, default_value_t = false)]
	save: bool,

	/// Don't print anything to stdout (except errors and debug)
	#[arg(short, default_value_t = false)]
	quiet: bool,
	
	/// URL Type
	#[arg(short, value_enum, default_value_t = URLType::Gif)]
	type_url: URLType,

	/// Print extended details. When set, both url types are printed regardless of -t
	#[arg(short, default_value_t = false)]
	extended: bool,

	/// Debug options
	#[arg(short, default_value_t = false)]
	debug: bool,

	/// A search term to query the tenor api. When run without arguments you get cat gifs
	query: Vec<String>,
}

#[derive(Debug)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum URLType {
	/// Direct .gif link
	Gif,
	/// Page url to the tenor link, suitable for discord (will display full resolution)
	Page
}

fn x11_copy_to_clipboard(text: &str) -> Result<(), std::io::Error> {
	let mut child = Command::new("xclip")
		.args(["-sel", "clip"])
		.stdin(Stdio::piped())
		.spawn()?;
    
	child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
	Ok(())
}

fn wayland_copy_to_clipboard(text: &str) -> Result<(), std::io::Error> {
	let mut child = Command::new("wl-copy")
		.stdin(Stdio::piped())
		.spawn()?;

	child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
	Ok(())
}

fn windows_copy_to_clipboard(text: &str) -> Result<(), std::io::Error> {
	let mut child = Command::new("clip")
		.stdin(Stdio::piped())
		.spawn()?;

	child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
	Ok(())
}

#[tokio::main]
async fn main () -> Result<(), Error> {
	dotenv().ok();
	let key = std::env::var("API_KEY").expect("Set an api key with --set-api-key <TOKEN>");
	let args = Cli::parse();
	// let mut stdin_query = String::new();
	// stdin().read_to_string(&mut stdin_query)?;
	let query = if args.query.len() != 0 { args.query.join(" ") } else { "cat".to_string() };
	// println!("{}", query);
	let request_url = format!("https://g.tenor.com/v2/search?q={query}&key={key}&limit={limit}",
		query = query,
		key = key,
		limit = args.limit,
	);

	//send request
	let client = reqwest::Client::new();
	let response = client
		.get(&request_url)
		.header(USER_AGENT, "rust-web-api-client")
		.send()
		.await?;

	// println!("{}", response.status());
	let result: ApiResult = response.json().await?;
	let gifs: Vec<Gif> = result.results;

	//print debug info
	if args.debug {
		println!("====== DEBUG ======");
		println!("api result: {:#?}", &gifs);
		println!("args struct: {:#?}", &args);
		println!("request url: {}", &request_url);
		println!("====== END DEBUG ======");
	} 

	if !args.quiet {
		//print the array
		let mut idx = 0;
		for gif in &gifs {
			if args.extended {
				idx += 1;
				println!("{}{}:\n {}\n {}\n {:?}\n \"{}\"\n", "Gif ".underline(), idx.to_string().underline(), gif.itemurl, gif.url, gif.tags, gif.content_description);
			} else {
				match args.type_url {
					URLType::Gif => {
						println!("{}", gif.url);
					}
					URLType::Page => {
						println!("{}", gif.itemurl);
					}
				}
			}
		}
	}
	
	if args.copy || args.save {
		let max = gifs.len();
		let idx = rand::rng().random_range(0..max);
		let random_gif = &gifs[idx];
		let gif_direct_link = &random_gif.media_formats.gif.url;
		let random_gif_link = if args.type_url == URLType::Gif { &gif_direct_link } else { &random_gif.itemurl };
		let supported_os = ["linux", "openbsd", "freebsd", "netbsd", "windows"];
		let os = env::consts::OS;

		if args.copy {
			match os {
				"linux"|"openbsd"|"freebsd"|"netbsd" => {
					if env::var_os("DISPLAY").is_some() {
						if let Err(e) = x11_copy_to_clipboard(&random_gif_link) {
							eprintln!("An error occured when calling `xclip`: {e}\nHeres your random link: {}", &random_gif_link);
							process::exit(1);
						}
					} else if env::var_os("WAYLAND_DISPLAY").is_some() {
						if let Err(e) = wayland_copy_to_clipboard(&random_gif_link) {
							eprintln!("An error occured when calling `wl-copy`: {e}\nHeres your random link: {}", &random_gif_link);
							process::exit(1);
						}
					} else {
						eprintln!("Failed to detect display server, are DISPLAY or WAYLAND_DISPLAY set?\nHeres your random link: {}", &random_gif_link);
						process::exit(1);
					}
				},
				"windows" => {
					if let Err(e) = windows_copy_to_clipboard(&random_gif_link) {
						eprintln!("An error occured when calling `clip`: {e}\nHeres your random link: {}", &random_gif_link);
						process::exit(1);
					}
				}
				_ => {
					eprintln!("Unsupported os \"{}\" for the copy function. Supported operating systems are {:?}\nHeres your random link: {}", os, supported_os, &random_gif_link);
					process::exit(1);
				}
			}
		}
	
		if args.save {
			let picture_dir = dirs_next::picture_dir().unwrap();
			//send request
			let client = reqwest::Client::new();
			let response = client
				.get(gif_direct_link)
				.header(USER_AGENT, "rust-web-api-client")
				.send()
				.await?;

			let mut filename = gif_direct_link.split("/").last().unwrap().to_string().clone();
			let mut path = picture_dir.join(&filename);
			if Path::new(&path).exists() {
				let random = random_range(0..=100000).to_string();
				filename.insert_str(filename.len()-4, &random);
				path = picture_dir.join(&filename);
			}

			let mut file = File::create(&path).expect("Failed to create file");
			
			let response_bytes = response.bytes().await?;
			file.write_all(&response_bytes).unwrap();
			println!("Saved file to {:?}", &path);
		}
	}


	Ok(())
}