# Tower Defense — Living Design Brief

**Status:** Exploratory concept. This is a direction for experiments, not a locked specification.
**Purpose:** Shared reference across chats, prototypes, and design experiments. Preserve the core vision; feel free to challenge unproven mechanics.

## 1. Core Vision

A fantasy tower-defense game about **designing interacting systems**, not merely stacking the highest-DPS towers. The player prepares enemy states, combines tower effects, and chooses where and when to exploit those interactions. The fun should come from solving each wave as a tactical puzzle.

- **Perspective / presentation:** Fixed isometric or 3/4 view; likely 2.5D. Illustrated, hand-painted / pen-and-ink-inspired fantasy art is a target direction, not a technical constraint.
- **Enemy behavior:** Enemies advance toward a castle, hero, or objective. Towers are not normally attacked.
- **Tone:** Strong faction identity, readable effects, and occasional surprising visual humor. Humor should be rare enough to stay surprising.

## 2. Core Gameplay Hypothesis

A useful interaction pattern is:

1. **Prepare** — apply a state or shape enemy movement (e.g. Wet, Marked, Corroded).
2. **Exploit** — another tower or effect interacts with that state (e.g. lightning chains through Wet enemies).
3. **Finish** — a specialized effect capitalizes on the setup (e.g. a Shatter attack against Brittle enemies).

States may be applied, transformed, consumed, or combined. Prefer a small, readable set of reusable rules over a huge list of bespoke exceptions. Limit how many states can meaningfully coexist on one enemy if that helps readability.

Potential state interactions (examples only):
- Wet + Lightning → chaining / amplified conduction
- Chilled + Brittle → Shatter / area damage
- Fire + Ice → thermal burst or a steam effect
- Marked + precision attack → amplified damage
- Corroded + force effect → stronger knockback

These are hypotheses, not final balance rules.

## 3. Towers and Progression

A tower may be built from **2–3 composable levels/modules**:

- **Level 1 — Foundation:** defines delivery and primary role (e.g. close-range blades, long-range piercing shot, area debuff).
- **Level 2 — Transformation:** changes how the foundation behaves (e.g. flaming blades, frozen swamp, piercing flame).
- **Level 3 — Specialization:** adds a strong, focused payoff (e.g. bonus against a specific state, powerful control field, support amplification).

Avoid upgrades that are simply “the same attack, but bigger.” Specialization should create meaningful choices and trade-offs.

Tower-to-tower synergy is a separate layer from a tower's internal modules: one tower may prepare an enemy state while another exploits it.

**Possible progression:** individual towers gain experience from useful contribution, not kills alone, allowing support/control towers to develop as veterans. This is unproven.

## 4. Runes: Linking Towers into Battlefield Effects

A promising strategic layer is linking at least three towers into a **rune**: a geometric pattern that creates a field, trap, or active weapon. Players can break and reconfigure links as the battle evolves.

Possible rune behaviors:
- **Passive field:** continuously slows enemies or amplifies another effect.
- **Automatic trap:** fires when a condition is met (e.g. a crowd enters, a boss appears, or enemies have a state combination).
- **Manual weapon:** the player chooses when to trigger a powerful effect.

Rune behavior may depend on:
- Tower elements and count (e.g. three Fire towers create a burst; Fire + Ice creates a hybrid effect; distinct elements prepare or control enemies).
- Geometry and coverage: a compact pattern might concentrate power; a stretched pattern might cover more ground at lower intensity.
- Cooldown, activation conditions, energy, or instability.

Reconfiguring a rune should be a meaningful strategic choice. Any cost or downside should emerge from interesting decisions—not arbitrary penalties.

Rune rules, shapes, thresholds, and examples are experimental. Do not assume every combination must have a unique bespoke effect.

## 5. Factions and Identity

Initial faction direction: **Evil** and **Good**.

- **Evil:** grotesque, experimental, chaotic towers; the first faction to explore because its visual and mechanical identity is immediately distinctive.
- **Good:** disciplined, controlled, preventative, or defensive tools—not merely a bland mirror of Evil.

Additional factions may be considered later. Do not expand faction count before the core system works.

## 6. Encounter Design

Waves should pose different problems: swarms, speed, armor, regeneration, resistances, and mixed formations. Avoid a single universally optimal build.

Counters should have multiple viable answers. Aim for readable cause-and-effect and strategic flexibility rather than strict rock-paper-scissors.

Longer-term possibility: an attacker designs enemy waves or compositions while a defender builds the tower system. This is a future direction only; multiplayer is explicitly **not** a first milestone.

## 7. Prototype / Lab Principles

The lab exists to test whether the central interactions are fun and understandable—not to build the whole game.

- Build the smallest experiment that can answer one clear question.
- Start with a few elements, a few enemy states, and a handful of interactions.
- Test composition, geometry, timing, readability, and counterplay separately where possible.
- Use temporary shapes/debug visuals if they expose the mechanic more clearly than finished art.
- Record the hypothesis, observed result, surprises, and next decision.
- Simplify or reject any mechanic that adds complexity without creating an interesting decision.

## 8. Current Unknowns

- Is the main fun in tower modules, cross-tower state synergies, rune geometry, or their combination?
- How much player control should rune activation allow?
- What trade-off makes linking towers interesting without making it feel like a tax?
- How many enemy states and elements can players comfortably understand?
- How should tower placement, coverage, cooldowns, and resource costs balance against rune power?
- What is the smallest playable experiment that proves the core loop?

**Guiding rule:** Protect the central promise—players create clever, interacting systems to solve waves—while treating every specific implementation as replaceable until tested.
