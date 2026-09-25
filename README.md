# Leitsys - Mobile

Mobile app for [leitsys_api](https://github.com/sJ3x4AiGYbt/leitsys_api), an implementation of the **Ebbinghaus spaced repetition method** for learning. Built with **Rust / Dioxus 0.7**.

```
Question created → Step 1 (1d) → Step 2 (3d) → … → Step 7 (90d) → Mastered ✓
                                           → wrong answer → back to Step 1
```

## Details

- Dioxus mobile target, sharing the same API as [leitsys_web](https://github.com/sJ3x4AiGYbt/leitsys_web).
- Currently scaffolding — no app logic yet.

## Development

```bash
dx serve --platform mobile
```
