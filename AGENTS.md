# PolyGlid Agent Loader

The canonical shared knowledge base is `.agents/shared/`.

Before changing code, read `.agents/shared/agent-startup.md` and follow its
selective loading rules. Load only the scope and rule files relevant to the
task.

Do not copy context into tool-specific folders. Keep stable project knowledge
in `.agents/shared/`; keep temporary session notes in the ignored local history
described by the startup file.
