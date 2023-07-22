# MathsPrompt

A simple tool that schedules maths problems to be solved.

For the initial input of basis questions, spin up the Rust server from the base
directory with a simple `cargo run`. This will start the server on port 8000.

Then, navigate to `localhost:8000` in your browser and enter the questions you
want. (You'll want to update the hardcoded `database_url` in the `src/main.rs` file.)
