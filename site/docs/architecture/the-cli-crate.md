---
sidebar_position: 4
---

# The `cli` crate

The `cli` crate contains binaries for managing different aspects of a Gerust project, mainly generating new files of various types, as well as executing database commands such as running migrations, etc. The `cli` command is part of the Gerust project for 2 reasons: a) to avoid an external dependency of the application on the `gerust` crate and b) to make the project fully functional with only a working Rust toolchain – more on both goals in the [architecture docs](./architecture).

A developer working on a Gerust project would typically not have to make changes to anything inside the `cli` crate directly, although in a future version we might add a mechanism for defining custom tasks inside the `cli` crate, similar to e.g. [how Ruby on Rails does it](https://guides.rubyonrails.org/command_line.html#custom-rake-tasks).

Gerust creates a workspace that is configured so that both binaries can conveniently be invoked as `cargo generate` and `cargo db` instead of the more convoluted `run --package my-app-cli --bin generate` and `run --package my-app-cli --bin db`.

## The `generate` binary

The `generate` binary is used to generate new files of certain types in the right places. These files will contain the basic structures and only need to be filled in by the developer. The kinds of files that can be generated include all elements of a Gerust project, e.g. migrations and entities, controllers, tests, and more. To see all of the options, run `cargo generate help`:

```
A CLI tool to generate project files.

Usage: generate [OPTIONS] <COMMAND>

Commands:
  middleware            Generate a middleware
  controller            Generate a controller
  controller-test       Generate a test for a controller
  migration             Generate a migration
  entity                Generate an entity
  entity-test-helper    Generate an entity test helper
  crud-controller       Generate an example CRUD controller
  crud-controller-test  Generate a test for a CRUD controller
  help                  Print this message or the help of the given subcommand(s)

Options:
      --no-color  Disable colored output.
      --debug     Enable debug output.
  -h, --help      Print help
  -V, --version   Print version
```

## The `db` binary

The `db` binary (which only exists for projects that use a database, otherwise it will not be generates) is used for running database operations such as executing migrations, seeding the database, etc. To see all of the available commands, run `cargo db help`:

```
A CLI tool to manage the project's database.

Usage: db [OPTIONS] <COMMAND>

Commands:
  drop     Drop the database
  create   Create the database
  migrate  Migrate the database
  reset    Reset (drop, create, migrate) the database
  seed     Seed the database
  prepare  Generate query metadata to support offline compile-time verification
  help     Print this message or the help of the given subcommand(s)

Options:
  -e, --env <ENV>  Choose the environment (development, test, production). [default: development]
      --no-color   Disable colored output.
      --debug      Enable debug output.
  -h, --help       Print help
  -V, --version    Print version
```
