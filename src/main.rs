use std::{
    error::Error,
    ops::Add,
    time::{Duration, Instant},
};

use reqwest::StatusCode;

fn main() {
    if let Some(parsed_args) = parse_args(std::env::args().collect()) {
        if parsed_args.url.is_empty() {
            show_help();
            return;
        }

        println!("\nsmashit - a simple, single machine, CLI-based HTTP load testing tool built whilst learning rust\n");

        let client = reqwest::blocking::Client::new();

        // Asynchronously distribute all requests.
        // Collate the responses.
        // Return the summary of all responses, including the number of those that failed and their response code.

        println!("🪄 Request summary");
        println!("\tURL: {0}", parsed_args.url);
        println!("\tMethod: {0}", parsed_args.method);
        println!("\tCount: {0}\n", parsed_args.count);

        match perform_request(&client, &parsed_args) {
            RequestStastics::Success(response) => {
                println!("😀 Request succeeded");
                println!("\tStatus Code: {0}", response.status_code.to_string());
                println!("\tResponse Size: {0}", response.response_size);
                println!("\tLatency: {0}ms", response.latency.as_millis());
                println!(
                    "\tResponse Time: {0}ms",
                    response.total_response_duration.as_millis()
                );
            }
            _ => println!("💀 Request failed"),
        }
    } else {
        show_help();
    }
}

/// Parses the given arguments into a struct that contains all of the options available.
fn parse_args(args: Vec<String>) -> Option<ParsedArgs> {
    let mut path = String::from("");
    let mut method = String::from("GET");
    let mut count = 1;

    let mut iterator = 1;
    while iterator < args.len() {
        match args[iterator].as_str() {
            "-u" | "--url" => path = get_next_argument(&mut iterator, &args)?,
            "-m" | "--method" => method = get_next_argument(&mut iterator, &args)?,
            "-c" | "--count" => {
                count = get_next_argument(&mut iterator, &args).and_then(|s| s.parse().ok())?
            }
            _ => return None,
        }
    }

    return Some(ParsedArgs {
        url: path,
        method: method,
        count,
    });
}

/// Given a current position and a vector of arguments, return the current position + 1 argument if it exists and it is
/// not empty.
fn get_next_argument(current_position: &mut usize, args: &Vec<String>) -> Option<String> {
    if args.len() - 1 < *current_position + 1 || args[*current_position + 1].is_empty() {
        None
    } else {
        *current_position = current_position.add(2);
        Some(args[*current_position - 1].clone())
    }
}

/// Shows the multi-line CLI help documentation for smashit.
fn show_help() {
    println!(
        "
smashit - a simple, single machine, CLI-based HTTP load testing tool built whilst learning rust

usage: smashit [options]

example: smashit -u https://my-api.com/users -c 25

options:
  -c | --count  The number of times to call the endpoint (default: 1)
  -u | --url    The URL to load test
  -m | --method The HTTP method to use in the request (default: GET)"
    );
}

/// Represents all available and defineable CLI arguments.
struct ParsedArgs {
    url: String,
    method: String,
    count: i32,
}

/// SuccessfulRequestStatistics represents timings, status codes and more pulled out from a successful request response.
struct SuccessfulRequestStatistics {
    status_code: StatusCode,
    response_size: usize,
    latency: Duration,
    total_response_duration: Duration,
}

/// SuccessfulRequestStatistics represents timings, status codes and more pulled out from a failed request response.
struct UnsuccessfulRequestStatistics {
    status_code: StatusCode,
}

enum RequestStastics {
    Success(SuccessfulRequestStatistics),
    Failure(UnsuccessfulRequestStatistics),
}

/// perform_request performs the request for a given set of arguments parsed from the command line.
fn perform_request(
    client: &reqwest::blocking::Client,
    parsed_args: &ParsedArgs,
) -> RequestStastics {
    let before_request = Instant::now();
    let result = client.get(&parsed_args.url).send()?;
    let latency = before_request.elapsed();
    let status = result.status();
    let response_size = result.bytes()?.len();
    let total_response_duration = before_request.elapsed();

    return Ok::<SuccessfulRequestStatistics, reqwest::Error>(SuccessfulRequestStatistics {
        status_code: status,
        response_size: response_size,
        latency: latency,
        total_response_duration,
    });
}
