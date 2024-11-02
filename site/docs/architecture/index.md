---
sidebar_position: 2
---

# Architecture

**Gerust is built with API servers in mind.** We expect it to be used in projects that provide an HTTP interface (likely JSON, e.g. REST, although Gerust makes not assumption on the exact data format used) over a network. That interface might be used to access and manipulate data in a database (however, that part is optional – see below) but Gerust can also be used e.g. as a proxy in front of other services. We specifically didn't create Gerust for projects that render HTML directly, or handle e.g. form submissions originating from browsers directly.

Gerust's goal is to handle aspects of backend projects that are not essential to the concrete system a developer is implementing. Questions like where to put what kind of file, or tasks like figuring out how to set up proper tracing isolating tests from each other, etc. shouldn't be what developers spend their time on. At the same time, Gerust aims for offering flexibility for aspects that are very well essential to projects – we make no assumptions on how exactly the data format of the API should look or how entities from the database map to resources that are exposed via the API.

Besides finding that balance between strictness on non-essential aspects and flexibility on essential aspects, Gerust aims for maintainability. Separating different concerns clearly, choosing the right dependencies and enabling simple and efficient workflows, all contribute to codebases that developers will be able to work on efficiently for the long-term.

## Main Choices

* axum
* postgres
* tracing out-of-the-box

## Project Structure

### The `web` crate

#### Testing

### The `db` crate

### The `config` crate

### The `cli` crate

### The `macros` crate
