# Contributing

## Development Setup

Install a recent Rust toolchain and run:

```shell
make check
```

## Common Commands

```shell
make build
make test
make fmt
make check
```

## Project Expectations

- Keep changes scoped and easy to review.
- Update tests when changing archive, CLI, or Mongo restore behavior.
- Update `README.md` when user-facing behavior changes.
- Prefer explicit error messages over silent failures.

## Pull Request Checklist

- `cargo fmt` passes.
- `cargo test` passes.
- `cargo check` passes.
- Documentation matches the code.
