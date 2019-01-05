extern crate reqwest;
extern crate regex;
extern crate clap;

use clap::{App, Arg};
use regex::Regex;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};

fn main() {
    println!("-=<|[[[ COMMON WORD GENERATOR STARTED ]]]|>=-");

    let arguments =
        App::new("Common Word Generator")
            .arg(Arg::with_name("LINK_FILE")
                .required(true)
                .help("The file containing a list of the URLs to build the blacklist from")
            )
            .arg(Arg::with_name("OUTPUT_FILE")
                .value_name("FILENAME")
                .help("The filename to output the blacklist into")
            )
            .arg(Arg::with_name("ratio of matches")
                .short("r")
                .long("match-ratio")
                .value_name("FLOAT")
                .help("Specify the ratio of links which must contain the same word \
                out of the total for it to be considered common. Defaults to 1.00")
            ).get_matches();


    //gather parameters and ensure the program is ready to run
    let input_filename = arguments.value_of("LINK_FILE").unwrap();
    println!("### - Reading links from {}", input_filename);
    let input_file = match File::open(input_filename) {
        Ok(opened) => opened,
        Err(e) => panic!("!!! - Could not open output file: {}", e)
    };
    let input_reader = BufReader::new(input_file);

    let output_filename = arguments.value_of("OUTPUT_FILE").unwrap_or("blacklist.txt");
    let mut output_file = match File::create(output_filename) {
        Ok(file) => file,
        Err(e) => panic!("!!! - Could not create output file: {}", e)
    };

    let match_ratio = match arguments.value_of("ratio of matches") {
        Some(value) => {
            match value.parse::<f64>() {
                Ok(float) => float,
                Err(_) => 1.00
            }
        },
        None => 1.00
    };


    //hard-coded links
    let mut url_vector = vec![];
    for line in input_reader.lines() {
        let value = line.unwrap();
        url_vector.push(value);
    }

    println!{"### - Fetching each link..."}; //: Vec<&mut String>
    let mut responses = vec![];
    for url in url_vector {
        match reqwest::get(url.as_str()) {
            Ok(mut response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.text() {
                        Ok(text) => {
                            let owned_text = text.to_owned();
                            responses.push(owned_text);
                        }
                        Err(e) => println!("!!! - encountered an error: {}", e)
                    }
                } else {
                    println!("!!! - Failed to GET \"{}\" due to receiving status code {}", url, response.status())
                }
            }
            Err(e) => println!("!!! - Failed to GET the URL \"{}\": {}", url, e)
        }
    }
    println!("$$$ - Done");

    println!("### - Cleaning responses of junk text...");
    for response in &mut responses {
        remove_scripts(response);
        remove_style(response);
        remove_html_nodes(response);
        remove_html_text(response);
        remove_numbers(response);
    }
    println!("$$$ - Done");

    println!("### - Extracting words from responses...");
    let mut extracted_words = vec![];
    for response in &responses {
        extracted_words.push(extract_words(response));
    }
    if extracted_words.len() == 0 {
        panic!("!!! - No words were extracted and the program is terminating prematurely");
    }
    println!("$$$ - Done.");

    println!("### - Finding common words using a ratio of {:.2}...", match_ratio);
    let mut common_words = vec![];
    'main: for word in &extracted_words[0] {
        let mut occurrences: usize = 1;
        'outer: for i in 1..extracted_words.len() {
            'inner: for value in &extracted_words[i] {
                if word.eq_ignore_ascii_case(value) {
                    occurrences += 1;
                    break 'inner;
                }
            }
        }
        if occurrences >= (match_ratio * extracted_words.len() as f64) as usize {
            common_words.push(word);
        }
    }
    println!("$$$ - Done");

    println!("### - Deduping the list of common words");
    for start in 0..common_words.len() {
        let mut end = start;
        while end < common_words.len() { //use a while loop rather than for since removing changes index
            if start != end && common_words[start].eq_ignore_ascii_case(common_words[end].as_str()) {
                common_words.remove(end);
            } else {
                end += 1;
            }
        }
    }
    println!("$$$ - Done");

    println!("### - Writing common words to output file: {}", output_filename);
    for word in common_words {
        match write!(output_file, "{}\n", word) {
            Ok(_) => (),
            Err(e) => println!("!!! - Error writing to output file: {}", e)
        }
    }
    println!("$$$ - Done");

    println!("-=<|[[[COMMON WORD GENERATOR COMPLETED ]]]|>=-");
}

fn extract_words(text: &String) -> Vec<String> {
    let rex = Regex::new(r"\w+").unwrap();
    let result: Vec<String> = rex.captures_iter(text)
        .filter(|cap| cap.get(0).unwrap().as_str().len() > 4)
        .map(|cap| cap.get(0).unwrap().as_str().to_string())
        .collect();
    result
}

fn remove_scripts(text: &mut String) {
    let rex = Regex::new(r"<script[\s\S]*?>[\s\S]*?</script>").unwrap();
    *text = rex.replace_all(text, "").to_string();
}

fn remove_style(text: &mut String) {
    let rex = Regex::new(r"<style[\s\S]*?>[\s\S]*?</style>").unwrap();
    *text = rex.replace_all(text, "").to_string();
}

fn remove_html_nodes(text: &mut String) {
    let rex = Regex::new(r"<[\s\S]*?>").unwrap();
    *text = rex.replace_all(text, "").to_string();
}

fn remove_html_text(text: &mut String) {
    let rex = Regex::new(r"&.*?;").unwrap();
    *text = rex.replace_all(text, "").to_string();
}

fn remove_numbers(text: &mut String) {
    let rex = Regex::new(r"\d+(?:\.\d+)?").unwrap();
    *text = rex.replace_all(text, "").to_string();
}