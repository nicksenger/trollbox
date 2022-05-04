# Iced Workshop

Welcome to the Iced GUI workshop!

Over the course of the hour we will be building a lightweight chat client (a trollbox) in Rust.

Before getting started, here are a few steps to complete on your own:

- Install rustup: https://rustup.rs/
- Clone this repository, navigate to it, and run `cargo check`. This will retrieve the dependencies and ensure that your Rust installation is good to go.

### Part 1 - Familiarization

- Lifecycle of an Iced application
- Review of the Application trait
- view -> update -> input (explore using print)

### Part 2 - Simple state machine

- Transitioning between two states
- Producing a new view based on the state
- Receiving messages via print
- Challenge: put the connection status into the application title, truncated form of latest message?

### Part 3 - Read only

- Append messages to the state
- View a list of messages
- A refactoring opportunity? Challenge: persist messages across connection failure!

### Part 4 - Scaffolding

- Add a text input to the view
- Wire it up. Messages: different input sources, same strategy!
- Using containers, layout and padding.
- Challenge: Add a list of "active" users

### Part 5 - Into the Future

- Add an on_submit handler
- Wire it up. Commands: handling asynchronous operations
- Challenge: optimistic rendering!

### Part 6 - Styles upon styles

- Add a new module for styles
- Implementing the stylesheet traits
- Challenge: make a Bahamas themed trollbox!
- Challenge: report errors with sending messages to the user, report connection errors, disconnection reasons
