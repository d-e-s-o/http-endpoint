Unreleased
----------
- Enabled CI pipeline comprising building, testing, and linting of the
  project
  - Added badge indicating pipeline status


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
