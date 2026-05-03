---
name: create-pdf
description: Use this skill to convert Markdown files into a professional PDF document. Triggers when the user asks to "convert to pdf", "create pdf", "print markdown", or "generate a pdf from".
allowed-tools:
  - read_file
  - write_file
  - list_dir
  - exec_command
  - glob_find
---

# create-pdf

This skill provides a standardized workflow for converting Markdown documents into beautifully formatted PDF files. It uses a bundled Node.js script that leverages the `md-to-pdf` package.

## Quick Start

When the user asks to convert a Markdown file to a PDF, use the `run_shell_command` tool to execute the provided generation script.

```bash
node workspace/.skills/create-pdf/scripts/generate_pdf.cjs path/to/your_file.md
```

The script will automatically:
1. Validate the file path.
2. Apply a clean, professional CSS stylesheet (typography, margins, code block formatting).
3. Set appropriate print margins.
4. Output the PDF in the exact same directory as the input Markdown file, with a `.pdf` extension.

## Workflow

1. **Verify File Existence**: Before running the script, ensure the target `.md` file actually exists using filesystem tools if you aren't sure.
2. **Execute Script**: Run the `generate_pdf.cjs` script.
3. **Verify Output**: The script will output `Success: PDF generated at path/to/your_file.pdf`. 
4. **Notify User**: Let the user know the absolute or relative path to the generated PDF.

## Dependencies

The `generate_pdf.cjs` script relies on a global installation of `md-to-pdf`. If the script fails indicating the command is not found, you can install it using:

```bash
npm install -g md-to-pdf
```
