use std::{
    collections::HashMap,
    ops::Add,
    sync::Arc,
    time::{Duration, Instant},
};

use histogram::Histogram;
use itertools::Itertools;
use reqwest::{Method, StatusCode};

/// Represents all available and defineable CLI arguments.
struct ParsedArgs {
    url: String,
    method: Method,
    count: i32,
    headers: HashMap<String, String>,
    body: Option<String>,
}

/// Represents different timing bounds calculated from all of the results.
struct ResponsesTimings {
    min: Duration,
    avg: Duration,
    max: Duration,
    fiftieth_percentile: Duration,
    seventy_fifth_percentile: Duration,
    ninetieth_percentile: Duration,
    ninety_ninth_percentile: Duration,
}

/// ResponseStatistics represents timings, status codes and more pulled out from a request's response.
#[derive(Debug)]
struct ResponseStatistics {
    is_success: bool,
    status_code: Option<StatusCode>,
    response_time: Option<Duration>,
}

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

        print_request_summary(&args);

        let mut requests = vec![];
        for _ in 0..args.count {
            let c = client.clone();
            let a = args.clone();
            requests.push(tokio::spawn(async move { perform_request(c, a).await }));
        }

        let results: Vec<ResponseStatistics> = futures::future::join_all(requests)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        print_results(results)
    } else {
        show_help();
    }
}

/// Parses the given arguments into a struct that contains all of the options available.
fn parse_args(args: Vec<String>) -> Option<ParsedArgs> {
    let mut path = String::from("");
    let mut method = Method::GET;
    let mut count = 1;
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut body: Option<String> = None;

    let mut iterator = 1;
    while iterator < args.len() {
        match args[iterator].as_str() {
            "-u" | "--url" => path = get_next_argument(&mut iterator, &args)?,
            "-m" | "--method" => {
                method =
                    match Method::from_bytes(get_next_argument(&mut iterator, &args)?.as_bytes()) {
                        Ok(r) => r,
                        Err(_) => return None,
                    }
            }
            "-c" | "--count" => {
                count = get_next_argument(&mut iterator, &args).and_then(|s| s.parse().ok())?;
                if count <= 0 {
                    return None;
                }
            }
            "-h" | "--header" => {
                let kvp = get_next_argument(&mut iterator, &args)?;

                // If it's not a key value pair.
                if !kvp.contains("=") {
                    return None;
                }

                let kvp_split: Vec<&str> = kvp.split("=").collect_vec();

                // If it's a badly formatted key value pair.
                if kvp_split.len() != 2 {
                    return None;
                }

                headers.insert(kvp_split[0].to_owned(), kvp_split[1].to_owned());
            }
            "-b" | "--body" => body = Some(get_next_argument(&mut iterator, &args)?),
            _ => return None,
        }
    }

    return Some(ParsedArgs {
        url: path,
        method: method,
        count,
        headers,
        body,
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

example: smashit -u https://my-api.com/users -c 25 -h \"Authorization=Bearer Foo\"

options:
  -c | --count  The number of times to call the endpoint (default: 1)
  -u | --url    The URL to load test
  -m | --method The HTTP method to use in the request (default: GET)
  -h | --header A header key value pair specified in the format of KEY=VALUE to be sent in the request"
    );
}

/// Performs the request for a given set of arguments parsed from the command line.
async fn perform_request(
    client: Arc<reqwest::Client>,
    parsed_args: Arc<ParsedArgs>,
) -> ResponseStatistics {
    let before_request = Instant::now();

    let mut request = client.request(parsed_args.method.clone(), parsed_args.url.clone());

    for (header, value) in &parsed_args.headers {
        request = request.header(header, value);
    }

    if parsed_args.body.is_some() {
        request = request.body(parsed_args.body.to_owned().unwrap());
    }

    let result = match request.send().await {
        Ok(r) => r,
        _ => {
            return ResponseStatistics {
                is_success: false,
                status_code: None,
                response_time: None,
            }
        }
    };

    if !result.status().is_success() {
        return ResponseStatistics {
            is_success: false,
            status_code: Some(result.status()),
            response_time: Some(before_request.elapsed()),
        };
    }

    let status = result.status();

    match result.bytes().await {
        Ok(bytes) => bytes.len(),
        _ => {
            return ResponseStatistics {
                is_success: false,
                status_code: Some(status),
                response_time: Some(before_request.elapsed()),
            }
        }
    };

    return ResponseStatistics {
        is_success: true,
        status_code: Some(status),
        response_time: Some(before_request.elapsed()),
    };
}

/// Prints a summary of the CLI arguments used.
fn print_request_summary(args: &ParsedArgs) {
    println!("🪄 Request summary");
    println!("\tURL: {0}", args.url);
    println!("\tMethod: {0}", args.method);
    println!("\tCount: {0}\n", args.count);
}

/// Generates and prints collated results from the collected request statistics.
fn print_results(results: Vec<ResponseStatistics>) {
    println!("\n🎉 Result summary");
    print_summaries(&results);
    println!("");
    print_status_code_counts(&results);
    println!("");
    print_timings(&results);
}

/// Prints a summary of the requests and their response outcomes.
fn print_summaries(results: &Vec<ResponseStatistics>) {
    println!(
        "\t{0} successful, {1} failed.",
        results.iter().filter(|r| r.is_success).count(),
        results.iter().filter(|r| !r.is_success).count(),
    );
}

/// Prints a table of the returned status codes and the number of times they occurred.
fn print_status_code_counts(results: &Vec<ResponseStatistics>) {
    println!("\t{0: <12} | {1: <12}", "Status Code", "Count");
    for (key, value) in get_ordered_status_code_counts_from_results(&results) {
        println!(
            "\t{0: <12} | {1: <12}",
            key.map_or_else(|| String::from("None"), |f| String::from(f.as_str())),
            value,
        );
    }
}

/// Prints a table of the timings of the responses.
fn print_timings(results: &Vec<ResponseStatistics>) {
    println!(
        "\t{0: <6} | {1: <6} | {2: <6} | {3: <6} | {4: <6} | {5: <6} | {6: <6}",
        "Min", "Avg", "Max", "50th", "75th", "90th", "99th"
    );

    let timings = get_timings_from_results(&results);
    println!(
        "\t{0: <6} | {1: <6} | {2: <6} | {3: <6} | {4: <6} | {5: <6} | {6: <6}",
        format!("{}ms", timings.min.as_millis()),
        format!("{}ms", timings.avg.as_millis()),
        format!("{}ms", timings.max.as_millis()),
        format!("{}ms", timings.fiftieth_percentile.as_millis()),
        format!("{}ms", timings.seventy_fifth_percentile.as_millis()),
        format!("{}ms", timings.ninetieth_percentile.as_millis()),
        format!("{}ms", timings.ninety_ninth_percentile.as_millis()),
    );
}

// Gets the minimum, average, maximum and percentile based timings from the results.
fn get_timings_from_results(results: &Vec<ResponseStatistics>) -> ResponsesTimings {
    let mut min = Duration::MAX;
    let mut max = Duration::ZERO;

    // average
    let mut count = 0;
    let mut total = Duration::ZERO;

    // percentiles
    let mut histogram = Histogram::new();

    for result in results {
        let response_time = match result.response_time {
            Some(r) => r,
            None => continue,
        };

        if response_time < min {
            min = response_time
        }

        if response_time > max {
            max = response_time
        }

        count = count + 1;
        total = total + response_time;
        histogram
            .increment(response_time.as_millis() as u64)
            .unwrap()
    }

    ResponsesTimings {
        min,
        avg: if count > 0 {
            total / count
        } else {
            Duration::ZERO
        },
        max,
        fiftieth_percentile: Duration::from_millis(histogram.percentile(50.0).unwrap()),
        seventy_fifth_percentile: Duration::from_millis(histogram.percentile(75.0).unwrap()),
        ninetieth_percentile: Duration::from_millis(histogram.percentile(90.0).unwrap()),
        ninety_ninth_percentile: Duration::from_millis(histogram.percentile(99.0).unwrap()),
    }
}

/// From a vector of response statistics generate an ordered hashmap grouping of the status codes in the response and
/// their counts.
fn get_ordered_status_code_counts_from_results(
    results: &Vec<ResponseStatistics>,
) -> HashMap<Option<StatusCode>, usize> {
    let mut response: HashMap<Option<StatusCode>, usize> = HashMap::new();

    for result in results {
        if response.contains_key(&result.status_code) {
            *response.get_mut(&result.status_code).unwrap() += 1;
        } else {
            response.insert(result.status_code, 1);
        }
    }

    return response
        .into_iter()
        .sorted_by(|a, b| a.1.cmp(&b.1))
        .collect();
}
