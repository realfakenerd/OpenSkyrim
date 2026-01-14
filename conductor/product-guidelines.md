# OpenSkyrim Product Guidelines

## Visual Design & Aesthetics
- **Skeuomorphic Interface:** The UI layer (Svelte 5) should prioritize a skeuomorphic design that mimics the original Skyrim aesthetic. This includes textures like weathered stone, parchment, and metallic accents to maintain player immersion.
- **Nostalgic Branding:** Branding assets, including the engine logo and launcher, should draw heavy inspiration from original Skyrim motifs and the iconic dragon imagery.

## User Experience & Feedback
- **Non-Intrusive Feedback:** Error messages and system feedback should remain subtle. Use non-intrusive notifications or console-only logs for minor issues to avoid breaking the user's flow.
- **In-Game Consistency:** While the tech is modern, the "feel" of the UI should always align with the classic Elder Scrolls experience.

## Documentation & Prose
- **Technical Precision:** Documentation and internal prose must be technical, precise, and objective. Focus on architectural details, API signatures, and performance benchmarks rather than narrative or in-universe flavor.

## Developer Experience (DX)
- **Modder-First Scripting:** Prioritize high-quality Developer Experience for Lua modders by providing robust autocomplete, clear API definitions, and effective debugging tools.
- **Reactive UI Architecture:** Emphasize efficient state management in the Svelte 5 frontend using Runes ($state, $derived, $effect) to ensure a high-performance HUD and menu system.
- **Rust Excellence:** Maintain strict safety and performance standards in the Core/Backend (Rust/Bevy) through rigorous linting and performance tracking.
