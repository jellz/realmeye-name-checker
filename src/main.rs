use std::{fs::File, io::Write, path::PathBuf};

use chrono::{Datelike, Timelike, Utc};
use path_absolutize::Absolutize;
use structopt::StructOpt;
use termprogress::prelude::*;

const PROFILE_API_URL: &str = "https://realmeye.com/player/";
const RAW_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const RAW_CHARS_DEBUG: &str = "abc";

#[derive(Debug, StructOpt)]
#[structopt()]
struct Opt {
	#[structopt(short, long)]
	debug: bool,
}

fn main() {
	let mut progress = Bar::default();
	let mut checked_count: f64 = 0.0;

	progress.println(
		"= Welcome to RealmEye Name Checker =\n\nThe program will start checking all 2-letter A-z names for availability on RealmEye.\nThe speed depends on your internet speed. This process could take up to 2 hours.\nThe name dump file will automatically be opened once the process is complete.",
	);

	let raw_chars = get_chars();
	let chars: Vec<&str> = raw_chars.split("").collect();

	let all_names = get_names(chars);

	let available_names: Vec<String> = all_names
		.iter()
		.filter(|n| {
			progress.set_title(&format!("Checking name: {}", n).to_string());
			let available =
				is_name_available(n).expect("Unable to check name availability in filter");
			checked_count += 1.0;
			let percentage = checked_count / all_names.len() as f64;
			progress.set_progress(percentage);
			available
		})
		.map(|s| s.clone())
		.collect();

	let file_path = create_dump_file(available_names).expect("Unable to create name dump file");
	println!("\nCreated name dump file at {}", file_path);
}

fn get_names(chars: Vec<&str>) -> Vec<String> {
	let mut names: Vec<String> = Vec::new();
	for c1 in &chars {
		for c2 in &chars {
			let name = format!("{}{}", c1, c2);
			if name.len() == 2 {
				names.push(name);
			}
		}
	}
	names
}

fn is_name_available(name: &str) -> Result<bool, Box<dyn std::error::Error>> {
	let url = format!("{}{}", PROFILE_API_URL, name);
	let client = reqwest::blocking::Client::new();

	let request = client
		.get(&url)
		.header("User-Agent", "RealmEye Name Checker 1.0");

	let text = request.send()?.text().expect("Unable to get response text");

	let available = !text.contains("Sorry, but we either"); // probably means the profile doesn't exist
	Ok(available)
}

fn get_chars() -> &'static str {
	let opt = Opt::from_args();
	if opt.debug {
		RAW_CHARS_DEBUG
	} else {
		RAW_CHARS
	}
}

fn create_dump_file(names: Vec<String>) -> Result<String, std::io::Error> {
	let now = Utc::now();
	let file_name = format!(
		"name-dump-{}-{}-{}-{}-{}-{}.txt",
		now.year(),
		now.month(),
		now.day(),
		now.hour(),
		now.minute(),
		now.second()
	);

	let path = PathBuf::from(&file_name)
		.absolutize()
		.unwrap()
		.to_str()
		.unwrap()
		.to_string();

	let mut file = File::create(&path).expect("Unable to *create* the name dump file");
	file.write_all(names.join("\n").as_bytes())
		.expect("Unable to write to the name dump file");

	open::that(&path)?;
	Ok(path)
}
