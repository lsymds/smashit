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
