# Custom Code Folding for Zed

A Zed extension that adds configurable code folding via custom comment markers.

Zed doesn't natively support custom foldable regions (like VS Code's `#region`/`#endregion`). This extension provides an LSP server that implements `textDocument/foldingRange`, letting you define fold regions with comment markers like `# +++ Section` / `# ---`.

The extension registers for 40+ languages. See `extension.toml` for the full list and please feel free to open a pull request to add support for more languages.

Regions can be nested:

```python
# +++ Outer
  # +++ Inner
  code
  # ---
# ---
```

## Setup

### 1. Install the extension

Search for "Custom Code Folding" in Zed's extension panel, or install from the command palette.

For local development:
```sh
cargo install --path crates/custom-code-folding-server
```
Then use "zed: install dev extension" from the command palette and select this repo.

### 2. Enable LSP folding ranges

Add to your Zed `settings.json`:

```jsonc
{
  "document_folding_ranges": "on"
}
```

**Important:** This replaces Zed's built-in indent/tree-sitter folding with LSP-provided folding ranges. Since Zed merges ranges from all active language servers, structural folds (functions, classes, etc.) are preserved as long as your primary language server provides them (most do: pyright, rust-analyzer, gopls, typescript-language-server, etc.).

## Default Fold Patterns

Two patterns are built in:

### `plus-minus`
```python
# +++ Section Name
code here
# ---
```
Works with `#` and `//` comment styles.

### `region`
```javascript
// #region Section Name
code here
// #endregion
```
Works with `//`, `#`, and `/* */` comment styles.

## Custom Patterns

Add custom patterns via `settings.json`:

```jsonc
{
  "lsp": {
    "custom-code-folding": {
      "initialization_options": {
        // Set false to disable built-in patterns
        "include_defaults": true,

        // Additional patterns (appended to defaults)
        "patterns": [
          {
            "name": "begin-end",
            "start": "^\\s*//\\s*BEGIN\\s+(?P<label>.*?)\\s*$",
            "end": "^\\s*//\\s*END"
          }
        ]
      }
    }
  }
}
```

- `start`: Regex matching the opening marker. Use `(?P<label>...)` to capture a label shown when folded.
- `end`: Regex matching the closing marker.
- `include_defaults`: When `true` (default), your patterns are added alongside the built-in ones. Set `false` to use only your custom patterns.

## Development

```sh
# Build the server
cargo build -p custom-code-folding-server

# Run tests
cargo test -p custom-code-folding-server

# Build the WASM extension
cargo build --target wasm32-wasip2 --release
```

## License

MIT
