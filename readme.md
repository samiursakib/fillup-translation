# 🤖 Fillup Translation: Automated JSON Localization

**Fillup Translation** is a command-line tool designed to significantly accelerate the developer workflow for managing multilingual projects. It automates the generation of translations in target language JSON files by using a master English (or other source language) file as a reference and leveraging AI to fill the gaps.

If you frequently add new keys to your English language JSONs, this tool ensures all other language files stay in sync with minimal effort.

## ✨ Features

- **Automatic Key Synchronization:** Compares the reference language (e.g. `en`) JSON file against all other language files in your defined structure.
- **Gap Filling:** Automatically requests translations only for the keys that are **missing** in the target language files.
- **Structure Preservation:** Maintains the original file structure, order of key-value pairs and indentation of the target JSON file.
- **Rate-Limit Handling:** Built-in sleep functionality to prevent API rate-limit errors (e.g., 503 errors).
- **Re-run Capability:** Allows for automatic re-runs to resolve failed translation tasks.

## 🛠️ Prerequisites

- **Rust / Cargo:** The tool is built with Rust and installed using `cargo`. You must have the [Rust toolchain](https://www.rust-lang.org/tools/install) installed on your system.
- **Folder Structure:** Folder structure must be like below.

```
public/
├── en/
│   ├── file1.json
│   └── file2.json
├── de/
│   ├── file1.json
│   └── file2.json
├── it/
│   ├── file1.json
│   └── file2.json
└── ja/
    ├── file1.json
    └── file2.json
```

## 🚀 Installation

Install the tool directly from the Git repository. This command will compile the project and place the `filltra` executable in your Cargo bin directory:

```bash
cargo install --git https://github.com/samiursakib/fillup-translation.git filltra
```

Note: This tool contains multiple packages. Keep it mind to include `filltra` in your install comand.

## 💡 Usage

The `filltra` tool uses command-line arguments to define the operation.

### Basic Command Structure

```bash
filltra <public-folder-path> [OPTIONS]
```

### 1\. Translate All Files (Default Behavior)

If no specific file is provided, `filltra` will look at the source language folder (`public/en` by default) and process **every JSON file** found, checking for missing keys in all other language folders.

```bash
filltra <public-folder-path>
```

### 2\. Translate a Specific File

To focus the job on just one file (e.g., `file1.json`), use the `--file` flag:

```bash
# Only synchronizes keys for file1.json, file2.json across all target languages.
filltra <public-folder-path> --file file1.json --file file2.json
```

### 3\. Change source language code

If your source language is something different than `en` then you need to use the `--lan-code` flag to reference that language.

```bash
# Target other language codes (e.g. de) to translate from.
filltra <public-folder-path> --lan-code de
```

### 4\. Adjust Indentation and API Delay

You can customize the output JSON indentation and the delay between API requests to respect rate limits.

| Option     | Description                                   | Default | Example      |
| :--------- | :-------------------------------------------- | :------ | :----------- |
| `--indent` | Number of spaces for JSON indentation.        | 4       | `--indent 2` |
| `--sleep`  | Seconds to wait between translation requests. | 0       | `--sleep 3`  |

```bash
# Sets indentation to 2 spaces and waits 3 seconds between API calls.
filltra <public-folder-path> --indent 2 --sleep 3
```
