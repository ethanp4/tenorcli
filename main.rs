use std::path::Path;
use std::env;
use std::fs::File;
use std::io::Write;
use std::process::{self, Command, Stdio};

use clap::{Parser, ValueEnum};
use colored::Colorize;
use rand::{random_range, Rng};
use serde::Deserialize;
use reqwest::Error;
use reqwest::header::USER_AGENT;
use dotenv;

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
	media_formats: MediaFormats,
	content_description_source: String,

}

#[derive(Deserialize, Debug)]
struct MediaFormats {
	nanowebm: MediaInfo,
	nanomp4: MediaInfo,
	mp4: MediaInfo,
	nanogif: MediaInfo,
	tinymp4: MediaInfo,
	tinygifpreview: MediaInfo,
	webp: MediaInfo,
	gif: MediaInfo,
	mediumgif: MediaInfo,
	nanogifpreview: MediaInfo,
	tinywebm: MediaInfo,
	webm: MediaInfo,
	loopedmp4: MediaInfo,
	tinygif: MediaInfo,
	gifpreview: MediaInfo
}

#[derive(Deserialize, Debug)]
struct MediaInfo {
	url: String,
	duration: f32,
	preview: String,
	size: u32
}

#[derive(Parser, Debug)]
#[command(version, about = "Tenorcli allows you to use tenor from the cli", after_help = format!("{}
 - tenorcli -sl1 freebsd cat {}
 - tenorcli -l5 -cq kitten good morning {}
 - tenorcli --limit 15 yakuza goro majima watermelon {}
 - tenorcli -t file -r nano-gif dog {}", 
 	"Example usage:".bold().underline(), 
	"-- save the result to a file in the users pictures library".bold(),
	"-- copy a random link from the first 5 results, with no output".bold(),
	"-- list 15 gif links".bold(),
	"-- copy a random link from the first 10 results with a very small resolution".bold()
), long_about = None)]
	
struct Cli {
	/// Number of items to list
	#[arg(long, short, default_value_t = 10, value_parser = clap::value_parser!(u8).range(1..=50))]
	limit: u8,
	
	/// Copy a random link (according to -t) to the clipboard selected from the list derived with <LIMIT>
	#[arg(long, short, default_value_t = false)]
	copy_random: bool,

	/// Save a random gif to the Pictures library selected from the list derived with <LIMIT>
	#[arg(long, short, default_value_t = false)]
	save_random: bool,

	/// Don't print anything to stdout (except errors and debug)
	#[arg(long, short, default_value_t = false)]
	quiet: bool,
	
	/// URL Type to display / copy
	#[arg(long, short, value_enum, default_value_t = URLType::File)]
	r#type: URLType,

	/// Lots of media types are provided by the api. You can see links to all of them with -e. This option is only effective with -t file (default behaviour)
	#[arg(long, short, value_enum, default_value_t = GifResolution::MediumGif)]
	resolution: GifResolution,

	/// Print all gif details. Not effected by -t
	#[arg(long, short, default_value_t = false)]
	extended: bool,

	/// Display the args struct and the api request url
	#[arg(long, short, default_value_t = false)]
	debug: bool,

	/// Set a v2 api key that you got from Google here: https://developers.google.com/tenor/guides/quickstart
	#[arg(long)]
	set_api_key: Option<String>,

	/// A search term to query the tenor api
	#[arg(required_unless_present = "set_api_key")]
	query: Vec<String>,
}

#[derive(Debug)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum URLType {
	/// Direct media link
	File,
	/// Page url to the tenor link, suitable for discord embeds
	Page
}

#[derive(Debug)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum GifResolution {
	/// Can be quite large
	Gif,
	/// Slightly compressed but not noticable
	MediumGif,
	/// Noticably compressed
	TinyGif,
	/// Even more compressed
	NanoGif,
	/// Animated webp
	Webp,
	/// Regular thumbnail
	GifPreview,
	/// Small thumbnail
	TinyGifPreview,
	/// Very small thumbnail
	NanoGifPreview,
	Mp4,
	LoopedMp4,
	TinyMp4,
	NanoMp4,
	Webm,
	TinyWebm,
	NanoWebm,
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

fn macos_copy_to_clipboard(text: &str) -> Result<(), std::io::Error> {
	let mut child = Command::new("pbcopy")
		.stdin(Stdio::piped())
		.spawn()?;

	child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
	Ok(())
}

fn get_requested_media_url<'a>(gif: &'a Gif, resolution: GifResolution) -> &'a std::string::String {
	return match resolution {
		GifResolution::Gif => &gif.media_formats.gif.url,
		GifResolution::MediumGif => &gif.media_formats.mediumgif.url,
		GifResolution::TinyGif => &gif.media_formats.tinygif.url,
		GifResolution::NanoGif => &gif.media_formats.nanogif.url,
		GifResolution::Webp => &gif.media_formats.webp.url,
		GifResolution::GifPreview => &gif.media_formats.gifpreview.url,
		GifResolution::TinyGifPreview => &gif.media_formats.tinygifpreview.url,
		GifResolution::NanoGifPreview => &gif.media_formats.nanogifpreview.url,
		GifResolution::Mp4 => &gif.media_formats.mp4.url,
		GifResolution::LoopedMp4 => &gif.media_formats.loopedmp4.url,
		GifResolution::TinyMp4 => &gif.media_formats.tinymp4.url,
		GifResolution::NanoMp4 => &gif.media_formats.nanomp4.url,
		GifResolution::Webm => &gif.media_formats.webm.url,
		GifResolution::TinyWebm => &gif.media_formats.tinywebm.url,
		GifResolution::NanoWebm => &gif.media_formats.nanowebm.url,
	};
}

#[tokio::main]
async fn main () -> Result<(), Error> {
	let config_filename = dirs_next::config_dir().expect("dirs_next couldnt get a config dir").join("tenorcli.conf");
	let args = Cli::parse();
	if args.set_api_key.is_some() {
		let mut file = File::create(&config_filename).unwrap();
		file.write_all(format!("API_KEY={}", args.set_api_key.as_ref().unwrap()).as_bytes()).expect("Couldnt write to config file");
		println!("New API key was written to {:?}", config_filename);
		process::exit(0);
	}
	
	if config_filename.exists() {
		dotenv::from_filename(config_filename).ok();
	}
	
	let key = std::env::var("API_KEY").expect(format!("{}", "An API key is required. Get one from https://developers.google.com/tenor/guides/quickstart and set it with --set-api-key <TOKEN>".bold()).as_str());
	
	// let mut stdin_query = String::new();
	// stdin().read_to_string(&mut stdin_query)?;
	let query = args.query.join(" ");
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

	//print debug info
	if args.debug {
		println!("====== DEBUG ======");
		// println!("api result: {:#?}", &gifs);
		println!("args struct: {:#?}", &args);
		println!("request url: {}", &request_url);
		println!("====== END DEBUG ======");
	} 

	if !response.status().is_success() {
		eprintln!("Didn't get a success code from the api, your api key might be incorrect");
		process::exit(1);
	}

	// println!("{}", response.status());
	let result: ApiResult = response.json().await?;
	let gifs: Vec<Gif> = result.results;

	if !args.quiet {
		//print the array
		for gif in &gifs {
			if args.extended {
				// println!("{}{}:\n {}\n {}\n Tags: {:?}\n {:#?}\nDescription: \"{}\"\n", "Gif ".underline(), idx.to_string().underline(), gif.itemurl, gif.url, gif.tags, gif.media_formats, gif.content_description);
				println!("{:#?}", gifs);
			} else {
				match args.r#type {
					URLType::File => {
						let requested_url = get_requested_media_url(gif, args.resolution);
						println!("{}", requested_url);
					}
					URLType::Page => {
						println!("{}", gif.itemurl);
					}
				}
			}
		}
	}
	
	if args.copy_random || args.save_random {
		let max = gifs.len();
		let idx = rand::rng().random_range(0..max);
		let random_gif = &gifs[idx];
		let gif_direct_link = get_requested_media_url(random_gif, args.resolution);
		let random_gif_link = if args.r#type == URLType::File { &gif_direct_link } else { &random_gif.itemurl };
		let supported_os = ["linux", "openbsd", "freebsd", "netbsd", "windows", "macos"];
		let os = env::consts::OS;

		if !supported_os.contains(&os) {
			eprintln!("Unsupported os \"{}\" for the copy function. Supported operating systems are {:?}\nHeres your random link: {}", os, supported_os, &random_gif_link);
			process::exit(1);
		}

		if args.copy_random {
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
				},
				"macos" => {
					if let Err(e) = macos_copy_to_clipboard(&random_gif_link) {
						eprintln!("An error occured when calling `pbcopy`: {e}\nHeres your random link: {}", &random_gif_link);
						process::exit(1);
					}
				}
				_ => {} // this path is already handled above
			}
		}
	
		if args.save_random {
			let picture_dir = dirs_next::picture_dir().expect("dirs_next couldnt get a picture dir");
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
			file.write_all(&response_bytes).expect("Couldn't write to file");
			println!("Saved file to {:?}", &path);
		}
	}

	Ok(())
}