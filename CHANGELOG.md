0.6.0
-----
- Updated `http` dependency to `1.0`
- Switched to using GitHub Actions as CI provider
- Switched from using `test-env-log` to `test-log`
- Bumped minimum supported Rust version to `1.61`


0.5.0
-----
- Removed all JSON specifics, including default parsing functionality
- Added support for providing per-endpoint headers through new
  `Endpoint::headers` method
- Adjusted `Endpoint::body` to return an `Option` on success
- Adjusted `Endpoint::body` and `Endpoint::parse` to return
  `Endpoint::ConversionError` on failure
- Adjusted `Endpoint::query` return a `Result`
- Require `std::error::Error` instead of `std::fmt::Display` for
  `Endpoint::ApiError` type
- Removed `DeserializeOwned` requirement from `Endpoint::Output` and
  `Endpoint::ApiError` types
- Removed `serde` dependency
- Bumped minimum supported Rust version to `1.46`


0.4.0
-----
- Require `'static` for `Endpoint::Error` type


0.3.0
-----
- Require `std::fmt::Debug`, `std::fmt::Display`, and
  `std::error::Error` for `Endpoint::Error` type
- Enabled CI pipeline comprising building, testing, linting, and
  coverage collection of the project
  - Added badges indicating pipeline status and code coverage percentage


0.2.0
-----
- Removed dependency on `hyper` crate
  - Changed `Endpoint::body` method to work with a `Cow<[u8]>` instead
    of `hyper::Body`
  - Removed `Hyper` variant from `Error` enum
- Preserved endpoint error message when converting into generic
  `Error::HttpStatus` variant


0.1.1
-----
- Added `base_url` method to `Endpoint` trait


0.1.0
-----
- Initial release
