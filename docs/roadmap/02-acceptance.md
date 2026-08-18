# Phase 2 Acceptance

The acceptance stage converts integration and profiling evidence into a reproducible release
verdict. It never copies Skyrim data into the repository or CI artifacts. Real-world acceptance
requires a legally owned, converted asset set on the target Windows GPU machine.

## Verdicts

- `accepted`: every required gate passed, a baseline was available, all real scenarios ran and the
  visual review was signed.
- `accepted-with-warnings`: automated gates passed, but non-required evidence is pending or the
  campaign was synthetic-only, baseline-free, skipped a gate, or used a dirty worktree.
- `rejected`: preflight, quality, robustness, functional, threshold, regression, screenshot, or a
  required visual gate failed.

Warnings are never silently converted into approval. `rejected` exits non-zero.

## Campaign

Run a short asset-independent plumbing check:

```powershell
./scripts/phase2-acceptance.ps1 -Quick -SkipQualityGates -SkipRobustness -SkipBuild
```

Run the complete target-hardware campaign:

```powershell
./scripts/phase2-acceptance.ps1 `
  -Assets D:\SkyrimConverted `
  -Worldspace 0x3c `
  -RuralGridX 0 -RuralGridY 0 `
  -DenseGridX 18 -DenseGridY -5 `
  -WaterGridX 7 -WaterGridY -2 `
  -Repetitions 3 `
  -RequireVisualSignoff `
  -VisualReview D:\Evidence\visual-review.json
```

Defaults are 2 minutes synthetic, 5 minutes per representative world area, 10 minutes streaming
stress and 30 minutes stability. Each scenario is run three times and evaluated by its median.

## Automated gates

Preflight records the OS, PowerShell, Cargo, Git, disk space, commit/worktree, CPU/GPU/driver and
exact converter/database contracts. Quality runs formatting, all workspace tests, Clippy with
warnings denied and a release workspace build. Robustness executes the stale manifest, cache,
database, bundle-output and missing-assets rejection paths.

The scenario matrix covers synthetic 250k instances, rural, dense, water, fast streaming stress and
stability. Required thresholds default to average FPS >= 60, frame P95 <= 16.67 ms, memory growth <=
0.5 GiB and zero streaming failures. Against `acceptance-baseline.json`, a regression above 5% is a
warning and above 10% is a failure.

## Visual evidence

The engine captures a PNG after warm-up for the first repetition of each scenario. `screenshots.json`
records its path, byte size and SHA-256. The campaign also writes `visual-review-template.json`.
Complete it with a named reviewer and `pass` for rural, dense, water and stress checkpoints:

```json
{
  "format_version": 1,
  "reviewer": "Reviewer Name",
  "reviewed_at": "2026-08-17T15:00:00-03:00",
  "checkpoints": [
    { "scenario": "rural", "status": "pass", "notes": "No terrain seams." },
    { "scenario": "dense", "status": "pass", "notes": "Materials and visibility are stable." },
    { "scenario": "water", "status": "pass", "notes": "Reflection is stable and non-recursive." },
    { "scenario": "stress", "status": "pass", "notes": "No duplicate or orphaned cells." }
  ]
}
```

An empty reviewer or missing checkpoint cannot satisfy visual sign-off.

## Evidence package

Every attempt writes `target/acceptance/<timestamp>-<hardware>/`, including metadata, preflight,
quality and robustness results, functional results, performance medians, comparisons, profiling
bundles, screenshots, visual-review template, `acceptance-report.json`, logs and
`acceptance-summary.md`. This directory is the release evidence; proprietary inputs are referenced
but never copied.

Use `-UpdateBaseline` only on a non-rejected target-hardware campaign. The manual `Phase 2
Acceptance` workflow runs on a self-hosted Windows GPU runner and retains evidence for 90 days.
