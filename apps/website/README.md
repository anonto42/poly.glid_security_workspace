# PolyGlid Website

The public PolyGlid website is a Dioxus Web application. Its design uses
semantic HTML rendered by Dioxus and ordinary CSS.

## Local development

Install the Dioxus CLI matching the workspace's Dioxus version, then run:

```sh
cd apps/website
dx serve --platform web
```

Build the GitHub Pages output from the repository root with:

```sh
scripts/site/build.sh
```
