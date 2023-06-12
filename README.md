When using `reqwest` sometimes you need to decide how to consume the response
body based on the response status or even based on some external input.
`reqwest::Response`, however, doesn't allow you to do it easily.
`TypedResponse` in this crate allows you to do exactly that.
