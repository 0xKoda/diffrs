# JSON Diff Tool

This Rust program provides a terminal user interface (TUI) for diffing JSON files. It uses Crossterm for handling terminal input/output and Ratatui for the TUI components. The tool can display differences between two JSON files in a clear, highlighted format.

## Features

- Edit JSON files directly within the TUI.
- Display differences between two JSON files with color highlights.
- Load JSON files from `./left.json` and `./right.json` using the `-f` flag.

## Usage

### Running the Tool

1. Ensure you have Rust installed. You can install Rust by following the instructions on [rust-lang.org](https://www.rust-lang.org/learn/get-started).
2. Clone the repository and navigate to the project directory.
3. Run the tool with the following command:

```sh
cargo run
```

### Running with the `-f` Flag

To load JSON files from `./left.json` and `./right.json`, use the `-f` flag:

```sh
cargo run -- -f
```

### Key Bindings

- **a**: Edit the left JSON file.
- **b**: Edit the right JSON file.
- **c**: Clear both JSON files.
- **d**: Diff the JSON files and display the result.
- **q**: Quit the application.

## Editing JSON Files

The tool uses the default editor set in your environment (e.g., `vim`). Ensure your `EDITOR` environment variable is set to your preferred text editor.

```sh
export EDITOR=vim
```

## License

This project is licensed under the MIT License.
