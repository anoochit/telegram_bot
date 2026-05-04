---
name: create-infographic
description: Generate professional infographics from text data or descriptions using Gemini 2.5 Flash Image. Use this skill whenever the user wants to visualize data, stats, processes, comparisons, or summaries as an image — even if they say "make a chart image", "turn this into a visual", "illustrate these stats", or "create an infographic." Trigger on any request to turn structured or unstructured information into a shareable visual graphic.
---

# Skill: Create Infographic

Transform raw data or a text description into a visually appealing infographic using AI image generation.

## Prerequisites

* **Python 3.x** installed.
* **Dependencies:** `pip install google-genai` if not already installed.
* **API Key:** Set `GOOGLE_API_KEY` in your environment, or place it in a `.env` file at the project root as `GOOGLE_API_KEY=your_key_here`.

## Usage

If the user provides a description, use it directly in `--description`. If they don't, ask for the content they want visualized, then run the script from the project root:

```bash
python .skills/create-infographic/scripts/generate_infographic.py \
  --description "Top 5 programming languages in 2024: 1. Python (25%), 2. JavaScript (20%), 3. Java (15%), 4. C++ (10%), 5. Go (8%)" \
  --output infographic_result.png \
  --ratio "16:9"
```

## Parameters

| Parameter | Description | Required | Default |
| --- | --- | --- | --- |
| `--description` | The data or content to visualize. Be specific — include labels, values, and any layout preferences. | Yes | — |
| `--output` | Filename or path where the finished infographic will be saved (PNG). | Yes | — |
| `--ratio` | Aspect ratio for the image. Options: `1:1`, `4:3`, `16:9`. | No | `1:1` |

## Description Guidance

The quality of the output depends heavily on the `--description`. When the user's input is vague, expand it before passing to the script:

* **Include numbers and labels:** "3 steps: 1. Sign up, 2. Configure, 3. Launch" beats "the onboarding process."
* **Specify layout intent if known:** "side-by-side comparison", "vertical timeline", "pie breakdown."
* **Choose the right ratio:** Use `16:9` for wide/landscape layouts (presentations, banners), `1:1` for social media squares, `4:3` for general use.

## Workflow

1. **Gather content:** If the user hasn't provided structured data, ask what they want the infographic to show.
2. **Run the script** with a detailed `--description`.
3. **Confirm output:** The script saves the image to `--output`. Share the file path with the user.
4. **Iterate if needed:** If the result doesn't match expectations, refine the description (more specific labels, different ratio) and re-run.

## Troubleshooting

* **`google-genai` not found:** Run `pip install google-genai` and retry.
* **API key error:** Ensure `GOOGLE_API_KEY` is set in your shell environment or `.env` file at the project root.
* **Poor visual output:** The description was likely too vague. Add specific values, categories, or layout instructions and re-run.
* **Wrong dimensions:** Adjust `--ratio` — `16:9` for wide layouts, `1:1` for square.