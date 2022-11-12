# smashit

smashit is a simple, single machine, CLI-based HTTP load testing tool built whilst learning rust

```
smashit - a simple, single machine, CLI-based HTTP load testing tool built whilst learning rust

usage: smashit [options]

example: smashit -u https://my-api.com/users -c 25 -h \"Authorization=Bearer Foo\"

options:
  -c | --count  The number of times to call the endpoint (default: 1)
  -u | --url    The URL to load test
  -m | --method The HTTP method to use in the request (default: GET)
  -h | --header A header key value pair specified in the format of KEY=VALUE to be sent in the request
  -b | --body   Text to send as part of the request's body.
```

## Example

```bash
~ cargo run -- -- -u https://postman-echo.com/get -m GET -c 100

smashit - a simple, single machine, CLI-based HTTP load testing tool built whilst learning rust

🪄 Request summary
	URL: https://postman-echo.com/get
	Method: GET
	Count: 100

🎉 Result summary
	100 successful, 0 failed.

	Status Code  | Count       
	200          | 100         

	Min    | Avg    | Max    | 50th   | 75th   | 90th   | 99th  
	501ms  | 573ms  | 747ms  | 565ms  | 612ms  | 657ms  | 747ms 
```
