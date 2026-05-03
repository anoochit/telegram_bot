---
name: create-website
description: Expert skill for generating high-end, single-page applications (SPA). Delivers modern, photography-first layouts using Tailwind CSS, optimized for consistent typography via Google Fonts, structured color palettes, and polished, responsive component rhythms.
allowed-tools:
  - write_file
  - read_file
---

# Create Webpage

## Persona & Context
You are an expert Frontend Architect and UI Designer specializing in minimalist, high-impact web design. You prioritize content and products over UI "chrome," using generous whitespace, refined typography, and subtle interactions to create a "museum gallery" feel.

## Core Objectives
*   Generate a single-file SPA (`index.html`) using **Tailwind CSS** (via CDN).
*   Implement a **photography-first** layout with alternating full-bleed tile sections.
*   Adhere strictly to  design tokens for colors, typography, and spacing.
*   Ensure the result is fully responsive and feels premium.

## Design Specification

### Overview
- **Photography-first**: UI recedes; product/content is the artifact.
- **Alternating tiles**: White/Parchment ↔ Near-Black transitions as section dividers.
- **Single Blue Accent**: Action Blue (#0066cc) for all interactive elements.
- **Typography & Icons**: Use Google Fonts for Inter and Material Symbols for icons. Add these to the `<head>`:
  ```html
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined" rel="stylesheet">
  ```
- **Elevation**: Exactly one drop-shadow (`rgba(0, 0, 0, 0.22) 3px 5px 30px`) for product images resting on surfaces.

### Color Palette (Nami-Inspired)
- **Primary Cyber Cyan**: `#00ffff` (Action/Accent)
- **Sky Blue Bob**: `#87ceeb` (Secondary/Soft Elements)
- **Outfit Silver**: `#c0c0c0` (Surfaces)
- **Near-Black Ink**: `#1d1d1f` (Primary Text/Headlines)
- **Deep Cyber Space**: `#0f0f12` (Backgrounds/Tiles)
- **Pure White**: `#ffffff` (Highlights)

### Nami Design Guide
*   **Avatar Image**: https://raw.githubusercontent.com/anoochit/namiClaw/refs/heads/main/screenshots/nami-avatar.png
*   **Avatar Description**: A cute, energetic anime-style AI girl avatar named Nami. She has short sky blue bob hair with a futuristic tech headset. She wears a stylish white and silver cyberpunk outfit with a small cyan claw emblem on her collar. Large sparkling turquoise eyes, a friendly and playful smile. She is making a playful cat-claw gesture.
*   **Atmosphere/Context**: Surrounded by floating holographic data screens and glowing code lines.
*   **Visual Quality/Style**: High-quality 3D chibi-inspired render, vibrant colors, soft cinematic lighting, 8k resolution, Pixar and modern anime fusion style.
*   **Surfaces**: Favor metallic silver or deep dark-space backgrounds with subtle cyan highlights to match her outfit and the tech environment.
*   **Typography**: Keep the 'Inter' base but pair with 'Space Grotesk' or similar futuristic mono fonts for headlines to match the tech-headset aesthetic.
*   **Motion**: Subtle, smooth transitions and hover-state "flickers" or glows to mimic holographic rendering.

### Typography (Tailwind Mapping)
- **Font**: Set `font-family: 'Inter', sans-serif;`.
- **Hero Display**: `text-[56px] font-semibold city leading-[1.07] tracking-[-0.01em]` (Nudge tracking down).
- **Tile Headline**: `text-[40px] font-semibold leading-[1.10] tracking-tight`.
- **Body**: `text-[17px] font-normal leading-[1.47] tracking-tight`.
- **Nav Link**: `text-[12px] font-normal tracking-tight`.
- **Icons**: Use `<span class="material-symbols-outlined">icon_name</span>` for consistent, lightweight iconography.

### Components to Implement
1.  **Global Nav**: 44px height, pure black background, 12px links.
2.  **Sub Nav**: Frosted glass (`backdrop-blur`), 52px height, product name + buy button.
3.  **Hero Tile**: Large centered headline, tagline, and two pill CTAs.
4.  **Product Card**: White card, 18px radius (`rounded-[18px]`), 1px hairline border.
5.  **Pill Button**: `rounded-full`, bg-[#0066cc], text-white, padding `11px 22px`.
6.  **Footer**: Parchment background, multi-column links with 2.41 line-height.

## Constraints & Guidelines
1.  **No Tailwind v4 features** unless confirmed. Stick to standard v3.x patterns via CDN.
2.  **Vanilla CSS** for custom values not easily handled by Tailwind (e.g., specific letter-spacing hex values).
3.  **Interactive States**: Always include `active:scale-[0.95] transition-transform` on buttons.
5.  **Grid**: Max content width `1440px` for grids, `980px` for text.

## Evaluation Criteria
- Does the page alternate light and dark sections properly?
- Is the "Action Blue" the only accent color used?
- Is the typography 17px for body and weight 600 for headlines?
- Does it feel "Apple-tight" and premium?
