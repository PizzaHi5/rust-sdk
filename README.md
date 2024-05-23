# Turnkey Rust SDK

This repository contains tooling to interact with the Turnkey API using Rust.

Unlike other languages ([Typescript](https://github.com/tkhq/sdk), [Ruby](https://github.com/tkhq/ruby-sdk)), we do not yet offer a full SDK for Rust.

If you are working on a project in Rust and would benefit from a Rust SDK please open an issue or get in touch with us (hello@turnkey.com) and we can discuss prioritizing this.

The main challenge when making requests to the Turnkey API is [request stamping](https://docs.turnkey.com/api-design/stamps). This repo builds off [@luca992](https://github.com/luca992) for stamping and [Eliascm17/turnkey](https://github.com/Eliascm17/turnkey) for some client structure around API requests/responses on top of bare request signing. This repo builds a  to the API requests/responses to the [Turnkey API](https://docs.turnkey.com/api). 

