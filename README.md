# Single Treaded TCP Server

A non blocking single threaded TCP server that multiple clients can connect to, send and receive messages.

### Libraries used

- `anyhow` - Used for simple error messages
- `tokio` - Because I am running in single threaded mode, this could have been done without however tokios async runtime is much cleaner to reason about.
- `clap` - Used for cli arg parsing + defaults
- `futures` - futures is only used due to the polling functions that I am using to read from a stream. futures `noop_waker_ref()` is a filler waker that does nothing so we can poll the stream manually.   

## Obervations 

Initially I started on a multi threaded version as that is what I am used to but thought it wouldn't be too hard as its a pretty small usecase. After looking into the tokio docs I was able to find alternatives for the `read_line` function that does not block (`poll_read`).

## Architecture

The design of the program is very simple, it consists of three main structures.
- `Server`- The server is where the bulk of the control logic sits. It has a main loop that will poll at a regular interval so that we do not overload the CPU with io bandwidth (This happened). It polls for new TCP requests that are coming into the server and parses the events which are then processed.
- `Client` - The client is a strucutre that holds the references to each indivitial client TCP stream and also has some functionality to read and write to the stream without blocking.
- `Event` - The events is just a simple enum with some encoding to format different actions on the server. 

