# Tide (Typst IDE)
<div align="center">
    <img width="120" height="120" src="https://github.com/lilBchii/tide/blob/main/assets/icons/thierry_with_bg.png">
    <sub><sup><p>Official Tide logo, designed by <a href="https://github.com/lilBchii">@lilBchii</a>.</p></sup></sub>
</div>

Tide is a lightweight, cross-platform IDE for Typst, written in Rust with
[Iced](https://github.com/iced-rs/iced). It offers a user-friendly graphical
interface for editing, compiling, and exporting Typst documents.

Tide focuses on local editing, it aims to enhance writing documents such as
course notes, curriculum vitae, letters, ... It is not meant for collaborative work
but will make working with your personal templates way easier with an offline setup.

Tide is available for Windows, Linux, and macOS.

This is work in progress. There are lot of features we would like to
add and issues to fix. Please refer to <a href='#Contributing'>How to Contribute</a>
if you have any great idea.
## Table of contents

<a href='#Features'>Features</a><br>
<a href='#Installation'>Installation</a><br>
<a href='#SysEnv'>System Environment</a><br>
<a href='#Configuration'>Configuration</a><br>
<a href='#UsageOverview'>Usage Overview</a><br>
<a href='#Contributing'>How to Contribute</a><br>
<a href='#License'>License</a><br>
<a href='#Creators'>Creators</a><br>

<a id='Features'></a>
## Features
- Project and file management (create, import, delete)
- Typst preview and compilation
- Export to PDF or SVG
- Local templates management
- Customizable appearance and editor behavior via `config.toml`
- Keyboard shortcuts and GUI actions for efficiency

<a id='Installation'></a>
## Installation
### From Built Executable
Executables for each platform are available on the
[Releases page](https://github.com/lilBchii/tide/releases). Download the appropriate
binary for your operating system and run it.

At first launch, Tide creates a system environment directory where user settings,
templates, projects cache and fonts are managed.
### From Source
To build Tide from source, ensure you have Rust installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

> [!TIP]
> Visit the [Official Rust Installation Guide](https://www.rust-lang.org/tools/install) for more informations.

Then, clone this repository and build the project with Cargo:
```bash
git clone https://github.com/lilBchii/tide
cd tide
cargo build --release
```
The resulting executable will be found in `target/release/`.

<a id='SysEnv'></a>
## System Environment
Upon first use, a configuration directory is created:
| Platform | Path                                     |
| -------- | ---------------------------------------- |
| Windows  | `%APPDATA%/Tide`                         |
| macOS    | `$HOME/Library/Application Support/Tide` |
| Linux    | `$HOME/.config/Tide`                     |

This directory contains:
- `templates/`: User-defined Typst templates
- `fonts/`: Custom fonts for Typst
- `config.toml`: Application configuration
- `recent.cache`: Cache of recent projects

> [!WARNING]
> Avoid deleting or moving this directory manually.

<a id='Configuration'></a>
## Configuration
The `config.toml` file allows customization of:
- Colors (`background`, `text`, `primary`, `success`, `danger`)
- Font sizes (global and editor-specific)
- Editor auto-pairs

Example:
```toml
[colors]
background = "#facae5"
text = "#2e0013"
primary = "#d900b6"
success = "#7000ff"
danger = "#e64169"

[general]
font-size = 14
window-scale-factor = 1.0

[editor]
font-size = 25

[editor.auto-pairs]
"(" = ")"
"<" = ">"
"{" = "}"
"[" = "]"
"|" = "|"
'"' = '"'
"$" = "$"
"`" = "`"
```

<a id='UsageOverview'></a>
## Usage Overview
### Home Screen
- Create a new project (blank or from template)
- Access recent projects
- Open existing projects
- Access Typst resources via direct links

### Project Management
- New Project: Creates a project with an initial `main.typ`
- From Template: Use an existing `.typ` file as a base
- File Tree: View and interact with project files
- Import File: Add files to your project
- Delete File: Permanently remove a file

### Editing
- Rich text editor with syntax support
- Autocompletion zone for Typst
- Document preview pane
- Debug console for Typst errors
- Status bar with save status and cursor position

### Compilation and Export
- Define a `.typ` file as main for compilation
- Auto-compile on save
- Export as:
    - PDF (default)
    - SVG (one file per page)
    - Typst Template (for reuse)

### Keyboard Shortcuts
- `Tab`: Add four spaces
- `Ctrl + S`: Force preview and save the current file
- `Ctrl + Arrow Right`: Move to the right boundary of a word
- `Ctrl + Shift + Arrow Right`: Move to the right boundary of a word and select all of its characters
- `Ctrl + Arrow Left`: Move to the left boundary of a word
- `Ctrl + Shift + Arrow Left`: Move to the left boundary of a word and select all of its characters
- `Del`: Delete the next character
- `Ctrl + O`: Move to the end of the line and break the current line
- `Ctrl + E`: Export current project as a PDF
- `Ctrl + Space`: Open the autocomplete context

<a id='Contributing'></a>
## How to Contribute
We would love to benefit everyone's experience and interest so feel free to
contribute. See [CONTRIBUTING](https://github.com/lilBchii/tide/blob/main/CONTRIBUTING.md)
for a guide on how to contribute.

<a id='License'></a>
## License
Tide is distributed under the terms of Mozilla Public License (Version 2.0).

See [LICENSE](https://github.com/lilBchii/tide/blob/main/LICENSE) for details.

<a id='Creators'></a>
## Creators
Tide is a student project started by [@lilBchii](https://github.com/lilBchii),
[@pacotine](https://github.com/pacotine), [@Mey](https://github.com/mey-vltn) and
[@Steorr4](https://github.com/Steorr4) at the Université Paris Cité. It's still
maintained by them.
