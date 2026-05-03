import argparse
import os
import sys
from pathlib import Path
from google import genai
from google.genai import types

api_key = os.environ.get("GOOGLE_API_KEY")
if not api_key:
    print("Error: GOOGLE_API_KEY environment variable not set.")
    sys.exit(1)

client = genai.Client(api_key=api_key)

def generate_infographic(description, output_path, ratio="1:1"):
    model_name = "gemini-3.1-flash-image-preview"

    try:
        # Nami-Inspired Color Palette & Style
        prompt = (
            f"Generate a professional and modern infographic. Content: {description}. "
            "Visual style: High-tech, clean, and futuristic. "
            "Theme: Dark theme with 'Nami' color palette: "
            "Primary Cyber Cyan (#00ffff) for accents and icons, "
            "Deep Cyber Space (#0f0f12) and Near-Black Ink (#1d1d1f) for backgrounds, "
            "Outfit Silver (#c0c0c0) for surfaces, and Pure White (#ffffff) for highlights. "
            "Style elements: Subtle holographic glows, clean sans-serif typography (Inter-style), "
            "and soft cinematic lighting. 8k resolution, premium quality."
        )

        print(f"Generating infographic with model '{model_name}' and ratio '{ratio}'...")

        response = client.models.generate_content(
            model=model_name,
            contents=[prompt],
            config=types.GenerateContentConfig(
                response_modalities=["IMAGE"],
                image_config=types.ImageConfig(
                    aspect_ratio=ratio,
                ),
            ),
        )

        image_saved = False
        if response.candidates:
            for part in response.candidates[0].content.parts:
                if part.inline_data is not None:
                    with open(output_path, "wb") as f:
                        f.write(part.inline_data.data)
                    image_saved = True
                    print(f"Success: Saved to '{output_path}'.")
                    break

        if not image_saved:
            print("No image returned.")

    except Exception as e:
        print(f"Error: {str(e)}")
        sys.exit(1)

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--description", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--ratio", default="1:1")
    args = parser.parse_args()
    generate_infographic(args.description, args.output, args.ratio)
