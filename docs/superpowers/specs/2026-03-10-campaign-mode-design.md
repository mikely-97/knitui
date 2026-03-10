# Campaign Mode Design

## Overview

Linear level progression with 3 campaign lengths (Short/Medium/Long). Player selects a campaign track, plays through handcrafted levels of increasing difficulty, earns bonus rewards that carry over, and can use ad rescues within per-level limits. Progress persists across launches.

## Data Model

### CampaignLevel (campaign_levels.rs)

```rust
pub struct CampaignLevel {
    pub board_height: u16,
    pub board_width: u16,
    pub color_number: u16,
    pub obstacle_percentage: u16,
    pub conveyor_percentage: u16,
    pub scissors: u16,        // base starting bonuses
    pub tweezers: u16,
    pub balloons: u16,
    pub ad_limit: u16,        // max ad rescues
    pub reward_scissors: u16, // earned on win
    pub reward_tweezers: u16,
    pub reward_balloons: u16,
}
```

Three const arrays: `SHORT_CAMPAIGN` (~15), `MEDIUM_CAMPAIGN` (~25), `LONG_CAMPAIGN` (~40).

### CampaignTrack (campaign.rs)

```rust
pub enum CampaignTrack { Short, Medium, Long }
```

### CampaignState (campaign.rs)

```rust
pub struct CampaignState {
    pub track: CampaignTrack,
    pub current_level: usize,
    pub banked_scissors: u16,
    pub banked_tweezers: u16,
    pub banked_balloons: u16,
    pub completed: bool,
}
```

Persistence: `~/.config/knitui/campaign.json` via serde_json. Stores a Vec of up to 3 CampaignState (one per track).

## UI Flow

1. Main menu → "Campaign" → CampaignSelect screen (pick Short/Medium/Long)
2. CampaignSelect shows progress for each track if save exists
3. Enter starts at current_level (or level 0 if no save)
4. Brief level intro card: "Level N/M — WxH, C colors"
5. Play the level
6. On Win: award rewards, advance level, save, show "N:Next Level M:Menu"
7. On Stuck: show "R:Retry A:Ad M:Menu" (ad limited per level)
8. On final level win: victory message, mark completed, return to menu
9. Completed campaigns can be restarted from CampaignSelect

## Game-over behavior in Campaign

- **Won**: N/R key advances to next level (not restart). Rewards banked.
- **Stuck**: R key retries same level. A key triggers ad rescue (if ad_limit allows).
- **M/Esc**: Return to menu. Progress saved at current level.

## Carry-over Bonuses

On level start: `config.scissors = level.scissors + state.banked_scissors` (same for tweezers, balloons).
On level win: `state.banked_scissors += level.reward_scissors` (same for tweezers, balloons).

## Ad Rescue

Each level's `ad_limit` passed to engine. Engine's existing `can_watch_ad()` / `watch_ad()` handles it. Early levels: generous (2-3). Late levels: tight (0-1).

## Difficulty Curve

All three tracks follow the same pattern at different paces:
- Early: small boards, few colors, no obstacles/generators, generous ads
- Mid: introduce obstacles, then generators, growing board/color counts
- Late: full complexity (8x8+, 8 colors, obstacles, generators, tight ads)

Rewards scale with difficulty — harder levels give more bonus items.

## New TuiState Variants

- `CampaignSelect { selected: usize }` — track picker
- `CampaignLevelIntro { track_idx: usize }` — brief level card

## Campaign Context in Playing State

A new field in main.rs: `campaign_ctx: Option<(usize, CampaignState)>` where usize is the track index. When Some, gameplay uses campaign behaviors (next level on R, show level progress).

## Files Changed

| File | Change |
|------|--------|
| `src/campaign_levels.rs` | NEW: CampaignLevel struct + 3 const level arrays |
| `src/campaign.rs` | NEW: CampaignTrack, CampaignState, load/save, level→config |
| `src/lib.rs` | Add `pub mod campaign; pub mod campaign_levels;` |
| `src/main.rs` | CampaignSelect/LevelIntro states, campaign_ctx, game-over R behavior |
| `src/renderer.rs` | render_campaign_select(), render_level_intro(), campaign keybar/overlay |
| `src/engine.rs` | Pass ad_limit from config (currently hardcoded to None) |
