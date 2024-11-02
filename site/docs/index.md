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
- and many more

For now, Gerust is just a project generator that creates the files and structure to get you started. There is no runtime dependency on Gerust – all the code is under your control.

[Architecture](./architecture) goes into more details about the reference architecture Gerust defines.

## Getting Started

There's several ways to get started:

- Starting fresh with an empty project, adding your code step-by-step.

Alternatively, you can start out with one of the example apps which comes with example implementations for all of Gerust's main concepts such as entities, controllers, middlewares, etc. Gerust comes with two different example apps:

- The full example is a task management app with a REST API. It uses all of Gerust's concepts.
- The minimal example uses no database and is as simple as it gets.

### Starting Fresh

### The Full Example App

### The Minimal Example App

## What's a "Gerust"?

"Gerust" is a play on "Gerüst", the German word for "framework" and Rust – thanks to [@skade](https://github.com/skade) who had the idea originally and allowed us to use it!
