use std::{
    ops::Add,
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::StatusCode;

#[tokio::main]
async fn main() {
    if let Some(parsed_args) = parse_args(std::env::args().collect()) {
        if parsed_args.url.is_empty() {
            show_help();
            return;
        }

        println!("\nsmashit - a simple, single machine, CLI-based HTTP load testing tool built whilst learning rust\n");

        let client = Arc::new(reqwest::Client::new());
        let args = Arc::new(parsed_args);

        println!("🪄 Request summary");
        println!("\tURL: {0}", args.url);
        println!("\tMethod: {0}", args.method);
        println!("\tCount: {0}\n", args.count);

        let mut requests = vec![];
        for _ in 0..args.count {
            let c = client.clone();
            let a = args.clone();
            requests.push(tokio::spawn(async move { perform_request(c, a).await }));
        }

        let results: Vec<RequestStatistics> = futures::future::join_all(requests)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        let successful_results: Vec<&RequestStatistics> = results
            .iter()
            .filter(|r| match r {
                RequestStatistics::Success(_) => true,
                _ => false,
            })
            .collect();

        let failed_results: Vec<&RequestStatistics> = results
            .iter()
            .filter(|r| match r {
                RequestStatistics::Failure(_) => true,
                _ => false,
            })
            .collect();

        println!(
            "{0} successful requests, {1} failed requests.",
            successful_results.len(),
            failed_results.len()
        )
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
#[derive(Clone)]
struct SuccessfulRequestStatistics {
    status_code: StatusCode,
    response_size: usize,
    latency: Duration,
    total_response_duration: Duration,
}

/// SuccessfulRequestStatistics represents timings, status codes and more pulled out from a failed request response.
#[derive(Clone)]
struct UnsuccessfulRequestStatistics {
    status_code: Option<StatusCode>,
}

/// RequestStatistics represents the outcomes from a given request (either success or failure, each of which have their
/// own values).
#[derive(Clone)]
enum RequestStatistics {
    Success(SuccessfulRequestStatistics),
    Failure(UnsuccessfulRequestStatistics),
}

/// perform_request performs the request for a given set of arguments parsed from the command line.
async fn perform_request(
    client: Arc<reqwest::Client>,
    parsed_args: Arc<ParsedArgs>,
) -> RequestStatistics {
    let before_request = Instant::now();

    let result = match client.get(&parsed_args.url).send().await {
        Ok(r) => r,
        _ => {
            return RequestStatistics::Failure(UnsuccessfulRequestStatistics { status_code: None })
        }
    };

    if !result.status().is_success() {
        return RequestStatistics::Failure(UnsuccessfulRequestStatistics {
            status_code: Some(result.status()),
        });
    }

    let latency = before_request.elapsed();

    let status = result.status();

    let response_size = match result.bytes().await {
        Ok(bytes) => bytes.len(),
        _ => {
            return RequestStatistics::Failure(UnsuccessfulRequestStatistics {
                status_code: Some(status),
            })
        }
    };

    let total_response_duration = before_request.elapsed();

    return RequestStatistics::Success(SuccessfulRequestStatistics {
        status_code: status,
        response_size: response_size,
        latency: latency,
        total_response_duration,
    });
}
