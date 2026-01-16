# Reviews

- `Node`
  - `max_attempts` isn't necessary. The runner will have a global max attempts limit.
  - How does this compare to the oriignal Ralph Loop version? (`docs/knowledge`)
- Doc comments are missing. Comments should be first class for docs in Ruststst.
- `V1_SCHEMA`: is using JSON Schemas idiomatic for rust? Instead of using structs? I'm asking because I don't know well. What's the robust & idiomatic solution? Also, is it directly inlining it in the code good code?
- Did you run clippy? Is clippy not included in `just ci`? If so, why?
