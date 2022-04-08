# Get As a Function, Eh!

[![crates.io](https://img.shields.io/crates/v/gafe.svg)](https://crates.io/crates/gafe)
[![CI](https://github.com/pietro/gafe/actions/workflows/ci.yml/badge.svg)](https://github.com/pietro/gafe/actions/workflows/ci.yml)

An AWS Lambda that will do HTTP Get requests for you.

## Is this any good?

Yes.

## What?

Send a JSON payload to the lambda with `headers` and `uri` and the code does request for the `uri` using your `headers`. You get back a JSON with the AWS Request ID of the lambda invocation, the HTTP status and the response headers. The response body is base64 encoded.

### Example

#### Request

```json
{
	"uri": "https://example.org",
	"headers": {
		"x-foo": "bar"
	}
}
```

#### Response

``` json
{
	"req_id": "dd50c345-8373-4145-8883-1717f2da7f75",
	"status": 200,
	"headers": {
		"accept-ranges": "bytes",
		"vary": "Accept-Encoding"
        ...
	},
	"body": "PCF..."
}
```

## How?

The Lambda Rust Runtime has [detailed information](https://github.com/awslabs/aws-lambda-rust-runtime#building-and-deploying-your-lambda-functions) on how to buld and deploy rust functions. But the quick and dirty is:

1. Build and package the binary using the `rustserverless/lambda-rust` docker image:

``` bash
docker run --rm -u $(id -u):$(id -g) \
  -e BIN=gafe \
  -v ${PWD}:/code \
  -v ${HOME}/.cargo/registry:/cargo/registry \
  -v ${HOME}/.cargo/git:/cargo/git \
  rustserverless/lambda-rust:latest
```

You can use the image `rustserverless/lambda-rust:latest-arm64` if your CPU is a 64-bit ARM.

2. Deploy function using the AWS CLI:

``` bash
 aws lambda create-function --function-name rustTest \
  --handler bootstrap \
  --zip-file fileb://./target/lambda/gafe/bootstrap.zip \
  --runtime provided.al2
  --role arn:aws:iam::XXXXXXXXXXXXX:role/your_lambda_execution_role \
  --environment Variables={LOG_LEVEL=info}
```

3. Test it out

``` bash
$ aws lambda invoke
  --cli-binary-format raw-in-base64-out \
  --function-name rustTest \
  --payload '{"uri": "https://example.org", "headers": {"x-foo":"bar"}}' \
  output.json
$ cat output.json # Prints the full non-formated version of the example response from above
```



# NOTES:

Payload limit (request and response):

6 MB (synchronous)
256 KB (asynchronous)
