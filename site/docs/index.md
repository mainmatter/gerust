---
sidebar_position: 1
---

# Intro to Gerust

Gerust provides an architecture and tooling for Rust backend projects. It takes care of the accidental complexity that comes with writing backends with Rust so you can stay focused on the essence of the system you're building:

- Separating distinct parts of the system into separate crates
- Organizing files into a logical folder structure
- Maintaining and running database migrations
- Isolating test cases that access the database
- Tracing and error handling
- and much more

For now, Gerust is just a project generator that creates the files and structure to get you started. There is no runtime dependency on Gerust – all the code that goes into your project remains under your control.

[Architecture](./architecture) goes into more details about the reference architecture Gerust defines.

## Getting Started

When getting started with Gerust, you need to decide whether your project is going to use a database or not.

Gerust calls projects that don't use a database, "minimal" projects. Those really are as simple as it gets: just a web server, functionality for reading in configuration if there is any, and CLI tooling for creating new controllers and other project files.

The other option is to create a "full" project that uses a database. Full projects also consist of a web server, functionality for reading in configuration, as well as CLI tooling for project management. In addition, they come with a bunch of additional concepts such as entity definitions, changesets, database access, and validations.

### Minimal Projects

- only web, no db
- tutorial shows how to Gerust's default minimal app works and how to create a new controller and add a middleware

### Full Projects

- web and db
- tutorial shows how to build a simple tasks mgmt app step-by-step
- generate full example app to see the results

## What's a "Gerust"?

"Gerust" is a play on "Gerüst", the German word for "framework" and Rust – thanks to [@skade](https://github.com/skade) who had the idea originally and allowed us to use it!
