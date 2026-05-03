---
name: book-mockup
description: Generate photo-realistic book mockup images using Gemini 2.5 Flash Image. Use when you need to visualize a book cover in a real-world setting with high-quality textures and natural lighting.
---

# Skill: Generate Book Mockup

Use this skill to transform a flat 2D book cover image into a realistic 3D mockup using a dedicated Python script.

## Description

Runs the `generate_mockup.py` script to overlay a cover image onto a 3D-rendered book scene based on a text description.

## Prerequisites

* **Python 3.x** installed.
* **Dependencies:** Ensure `Pillow`, `google-genai`, and `python-dotenv` are installed (see `pyproject.toml`).
* **Assets:** The input cover image must exist in the root or specified directory.

## Usage

Execute the script from the project root using the following command structure:

```bash
python <skill-path>/generate_mockup.py \
  --cover <data-path>cover.png \
  --description "A book lying on a wooden table with soft morning light" \
  --output <output-path>\mockup_result.png

```

## Parameters

| Parameter | Description | Required |
| --- | --- | --- |
| `--cover` | Path to the source 2D cover image (e.g., .png, .jpg). | Yes |
| `--description` | A natural language prompt describing the mockup environment. | Yes |
| `--output` | The filename/path where the finished mockup will be saved. | Yes |
