# cargo-space

`cargo-space` adds workspace-centric functionality to Cargo through the `cargo space` command, extending Cargo with additional workflows for managing Rust workspaces, packages, and dependencies while matching Cargo's existing behavior and conventions. Some code is derived from Cargo and cargo-edit and modified to meet `cargo-space`'s needs.

## Commands

### `cargo space new`

Create a new workspace.

```bash
cargo space new my-project
```

Create a workspace and its package structure.

### `cargo space init`

Initialize a workspace in an existing directory.

```bash
cargo space init
```

### `cargo space create`

The default package type is a binary crate.

```bash
cargo space create app
```

### `cargo space add`

Add a dependency to a workspace or package.

```bash
cargo space add serde
```

A package can inherit the dependency from the workspace:

```bash
cargo space add serde -p app
```

This results in workspace-level dependency information:

```toml
[workspace.dependencies]
serde = "1"
```

and package-level inheritance:

```toml
[dependencies]
serde.workspace = true
```

Package-specific dependency configuration can also be specified:

```bash
cargo space add serde -p app --features derive --private features
```

which can produce:

```toml
[workspace.dependencies]
serde = "1"
```

```toml
[dependencies]
serde = { features = ["derive"], workspace = true }
```

### `cargo space remove`

Remove dependencies from a workspace or package.

```bash
cargo space remove serde
```

Multiple dependencies can be removed in one command:

```bash
cargo space remove serde tokio
```

A specific package can be selected with `-p`:

```bash
cargo space remove serde -p app
```

Development, build, and target-specific dependencies are supported.

### `cargo space update`

Update a dependency.

```bash
cargo space update serde
```

A version requirement can be specified using Cargo's `crate@version` style:

```bash
cargo space update serde@1.0.200
```

A complete semantic version is resolved as an exact version, while a partial semantic version is resolved as a compatible version requirement:

```bash
cargo space update serde@1.0
```

## Attribution

Some code in `cargo-space` is derived from Cargo and cargo-edit and has been modified and adapted for `cargo-space`'s needs while matching Cargo's existing behavior and conventions.

## Development Status

`cargo-space` is under development.

Current commands:

```text
cargo space new
cargo space init
cargo space create
cargo space add
cargo space remove
cargo space update
```

The project is focused on extending Cargo's existing workflow with a workspace-centric approach while maintaining compatibility with Cargo behavior.
