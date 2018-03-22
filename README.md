# too-funky

Too funky is a tiny x86 kernel which attempts to apply Rust's ownership model to
operating systems, with a few minor changes.

Files and processes are strongly bound together.  A process is similar to Rust's
scope and a file is similar to Rust's non-`Copy` type.  The specific semantics
will remain under discussion in issue
[#1](https://github.com/pi-pi3/too-funky/issues/1) for a while.
