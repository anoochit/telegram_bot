---
name: create-epub
description: Use this skill to convert Markdown files into an EPUB e-book document. Triggers when the user asks to "convert to epub", "create epub", "make an ebook", or "generate an epub from".
allowed-tools:
  - read_file
  - write_file
  - list_dir
  - exec_command
  - glob_find
---

# create-epub

This skill provides a standardized workflow for converting Markdown documents into EPUB e-book files. It uses a bundled Node.js script that leverages the `md-to-epub` package, which is ideal for compiling documentation or books into e-reader friendly formats.

## Quick Start

When the user asks to convert a Markdown file to an EPUB, use the `run_shell_command` tool to execute the provided generation script.

```bash
node workspace/.skills/create-epub/scripts/generate_epub.cjs path/to/your_file.md [optional/output/path.epub]
```

The script will automatically:
1. Validate the file path and sanitize any encoding issues (like Windows UTF-16 BOMs).
2. Run the markdown-to-epub converter.
3. Locate the output file and move it to the requested location (defaults to the same directory as the input Markdown file, with an `.epub` extension).

## Workflow

1. **Verify File Existence**: Before running the script, ensure the target `.md` file actually exists using filesystem tools if you aren't sure.
2. **Execute Script**: Run the `generate_epub.cjs` script.
3. **Verify Output**: The script will output `Success: EPUB generated at path/to/your_file.epub`. 
4. **Notify User**: Let the user know the absolute or relative path to the generated EPUB file so they can open it in an e-reader (like Apple Books, Calibre, or Kindle).

## Dependencies

The `generate_epub.cjs` script relies on a global installation of `md-to-epub`. If the script fails indicating the command is not found, you can install it using:

```bash
npm install -g md-to-epub
```
