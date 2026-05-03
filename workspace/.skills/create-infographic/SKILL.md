---
name: create-infographic
description: Generate professional infographics from text data or descriptions using Gemini 2.5 Flash Image. Use when you need to visualize data, processes, or summaries.
---

# Skill: Create Infographic

Use this skill to transform raw data or a text description into a visually appealing infographic.

## Description

Runs the `generate_infographic.py` script to create a modern infographic based on your provided data using AI image generation.

## Prerequisites

* **Python 3.x** installed.
* **Dependencies:** Ensure `google-genai` is installed.
* **API Key:** `GOOGLE_API_KEY` must be set in your environment.

## Usage

Execute the script from the project root using the following command:

```bash
python .skills/create-infographic/scripts/generate_infographic.py \
  --description "Top 5 programming languages in 2024: 1. Python (25%), 2. JavaScript (20%), 3. Java (15%), 4. C++ (10%), 5. Go (8%)" \
  --output infographic_result.png \
  --ratio "16:9"
```

## Parameters

| Parameter | Description | Required | Default |
| --- | --- | --- | --- |
| `--description` | The data or description of the infographic content. | Yes | - |
| `--output` | The filename/path where the finished infographic will be saved. | Yes | - |
| `--ratio` | Aspect ratio for the image (e.g., 1:1, 4:3, 16:9). | No | 1:1 |
