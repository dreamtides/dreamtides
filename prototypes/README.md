# Prototypes

This directory contains standalone Go prototypes.

## Layout

Each prototype should live in its own subdirectory with its own `go.mod`.
The `go.work` file in this directory tracks all active prototypes so tooling can
run across every module at once.

## Current prototype

- `hello_world/`: basic binary used as a starter template.

## Adding a new prototype

1. Create a new subdirectory, e.g. `prototypes/my_proto`.
2. Run `go mod init dreamtides/prototypes/my_proto` inside it.
3. Add the module path to `prototypes/go.work` under `use (...)`.
