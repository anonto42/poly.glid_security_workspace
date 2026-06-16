# PolyGlid Brand Style Guide

## 🎨 1. Core Visual Concept & Logo Directions

The branding should visually merge the concepts of **multi-layered architecture (Poly)** and **frictionless systems speed (Glid)**.

### Logo Icon Concept:

* **The Glid-Hex:** A sharp hexagon broken into segmented, layered parallel lines that appear to be shifting or moving forward smoothly. It emphasizes both the hexagonal architecture and high execution velocity.
* **The Monogram:** An abstract geometric letter **P** combined with a terminal bracket (`>`), built completely from clean vector angles.

---

## 💻 2. Color System & UI Themes

Security engineers spend long hours staring at diagnostic pages. PolyGlid should default to a premium, low-fatigue dark mode, but adapt elegantly to light environments.

### 🌑 Primary Dark Mode (Default)

* **Canvas Backdrop:** `#0B0F19` (Deep Cyber Blue-Grey) – Keeps contrast high without burning the eyes.
* **Surface Panels:** `#161F30` (Muted Steel) – For sidebar containers, terminal spaces, and inner tab layouts.
* **Primary Accent:** `#00E5FF` (Electric Cyan) – Used for active scan indicators, metrics, and primary buttons. Represents the "Glide" speed.
* **Secondary Accent:** `#A855F7` (Neon Purple) – Used to denote core engine processes, host connections, and WASM plugin states.
* **System Alerts:** * *Success/Secure:* `#10B981` (Emerald Green)
* *Warning/Vulnerability:* `#F59E0B` (Amber Orange)
* *Critical/Error:* `#EF4444` (Crimson Red)



### ☀️ Light Mode (Alternative)

* **Canvas Backdrop:** `#F8FAFC` (Clean Slate)
* **Surface Panels:** `#FFFFFF` (Pure White)
* **Text Primary:** `#0F172A` (Ink Navy)

---

## 🔤 3. Typography & Font Families

The font architecture must handle both high-density data arrays (like port listings or exploit codes) and clean interface navigation labels.

### Interface & UI Typography (Headings, Sidebars, Tabs)

* **Font Family:** **JetBrains Sans** or **Inter**
* **Why it fits:** Extremely clean, geometric sans-serif typefaces with open counters. They remain readable at tiny sizes (like 11px sidebars on mobile devices or tablets).

### Data & Code Typography (Terminals, Log Views, Payload Editors)

* **Font Family:** **JetBrains Mono** or **Fira Code**
* **Why it fits:** Outstanding coding fonts with crisp legibility and strong layout ligatures (e.g., turning `==>` into an explicit arrow symbol automatically). This makes parsing thousands of live terminal logs incredibly smooth for an engineer.

---

## 📂 4. Interface Layout Structure

To support multiple tasks at once without feeling cluttered, PolyGlid uses a classic three-tier layout pattern designed for scannability.

1. **The Navigation Utility Strip (Left Edge):** Thin vertical dock containing core app functions (Workspace, Plugin Marketplace, Local Engine Settings).
2. **The Active Workspace Console (Center/Right):** Supporting side-by-side split screens so you can view an ongoing dynamic port scan on the left pane while analyzing a script console on the right pane.
3. **The Global Live Status Footer (Bottom Edge):** Displays active memory consumption, current host status, and background thread activity loops.

---

## 📄 5. Official Brand Guidelines Document Structure

When you publish PolyGlid to GitHub, you should include a `BRANDING.md` file in the root directory. Copy and paste this exact markdown format to structure your repository's brand manual:

```markdown
# PolyGlid Identity & Style Manual

Welcome to the official brand repository for the **PolyGlid Security Workspace**. This document outlines the core structural design language, coloring palettes, and asset guidelines for developers building plugins or extending the core interface framework.

## Brand Essence
PolyGlid is a cross-platform, multi-window security environment built natively in Rust. It utilizes WebAssembly to execute isolated, bare-metal diagnostic tools with zero runtime lag.

## Core Typography
- **UI & Interface Elements:** `Inter`, Sans-Serif (Weight: 400 Regular, 600 Semi-Bold)
- **Log Interfaces & Terminals:** `JetBrains Mono`, Monospace (Weight: 400 Regular)

## Official Color Palettes

### 1. Dark Mode System
| Element | Hex Code | Purpose |
| :--- | :--- | :--- |
| Canvas Background | `#0B0F19` | Main application shell canvas |
| Surface Panel | `#161F30` | Active tab bodies, sidebars, panels |
| Accent Primary | `#00E5FF` | Active triggers, highlights, focus states |
| Accent Secondary | `#A855F7` | Core routing, module indicators |

### 2. Semantic Alerts
- **Secure / Clean System:** `#10B981` (Green)
- **Auditing Alert / Warning:** `#F59E0B` (Amber)
- **Exploit Match / Critical Failure:** `#EF4444` (Red)

## Component Spacing & Layout
- All dashboard boundaries must strictly adhere to an **8px grid spacing system** to optimize rendering scaling across Linux, Windows, macOS, and Android viewports.
- Interactive workspace windows must support dynamic viewport fracturing (horizontal/vertical split sheets) without collapsing critical navigation targets.

```