# Alien Pet — Project Plan

A bioluminescent alien creature that lives on a Raspberry Pi, expresses emotion through
form and light rather than a face, is fed via Telegram, and gradually learns its own
behavioral personality through a custom ML model you train over three months.

**Stack:** Rust (renderer, state machine, logger, systemd service) · Python (Telegram bot,
Claude API calls, ML training) · Claude API (behavior engine during collection) · SQLite
(shared state bus between processes)

---

## Overview

| Phase | Duration | Goal |
|---|---|---|
| 1 | Weeks 1–2 | Hardware up, creature on screen, Telegram working |
| 2 | Weeks 3–5 | Claude API drives behavior, creature comes alive, data logging begins |
| 3 | Weeks 6–12 | Silent data collection running, ML fundamentals studied in parallel |
| 4 | Month 3+ | Train custom model, replace Claude API, creature runs fully offline |

---

## Phase 1 — Hardware and Telegram core

**Duration:** Weeks 1–2
**Goal:** A creature on screen that you can feed from your phone.

### What you are building

- Raspberry Pi boots directly into the avatar renderer with no desktop environment
- Small screen shows the creature in idle state — 8 tendrils, teal, slow pulse
- Telegram bot accepts `/feed`, `/vitals`, and `/mood` commands
- Pet state (hunger, energy, happiness) persists in SQLite
- OpenClaw GPIO button counts as a physical feed event, identical to Telegram feed

### Rust — renderer skeleton

Set up your Rust project with `pixels` or `minifb` for framebuffer rendering. The creature
at this stage is just the idle state. No emotion switching yet. The goal is a stable render
loop at a solid framerate before adding any complexity.

```toml
[dependencies]
pixels     = "0.13"
winit      = "0.29"
rusqlite   = { version = "0.31", features = ["bundled"] }
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
```

Cross-compile on your main machine and `scp` the binary to the Pi. The render loop runs
as a systemd service so it starts on boot without a desktop session.

```ini
[Unit]
Description=Alien Pet Renderer

[Service]
ExecStart=/home/pi/alien-pet/renderer
Restart=always
Environment=DISPLAY=:0

[Install]
WantedBy=multi-user.target
```

### Python — Telegram bot

Use `python-telegram-bot` (async). This is the one place you deliberately choose Python
over Rust. The `teloxide` crate is good but async plus borrow checker plus lifetimes is
too much cognitive overhead at project start.

```python
from telegram.ext import ApplicationBuilder, CommandHandler
import sqlite3, datetime

DB = "pet.db"

async def feed(update, ctx):
    conn = sqlite3.connect(DB)
    conn.execute(
        "INSERT INTO events (type, ts) VALUES ('feed', ?)",
        (datetime.datetime.now().isoformat(),)
    )
    conn.commit()
    await update.message.reply_text("Your creature stirs. It feels the resonance.")

app = ApplicationBuilder().token("YOUR_TOKEN").build()
app.add_handler(CommandHandler("feed", feed))
app.run_polling()
```

The Python bot and Rust renderer communicate only through SQLite — no IPC, no sockets,
no message queues. The Rust process polls the `events` table every few seconds and updates
the state struct accordingly. This is the entire integration layer.

### SQLite schema

Lock this down before Phase 2 begins and do not change it afterward. Every column in
`behavior_log` becomes a potential feature or label for ML training. Adding or removing
columns mid-collection creates inconsistency that is painful to resolve before training.

```sql
CREATE TABLE state (
    id            INTEGER PRIMARY KEY,
    hunger        REAL DEFAULT 0.0,
    energy        REAL DEFAULT 1.0,
    happiness     REAL DEFAULT 0.7,
    current_state TEXT DEFAULT 'null-state',
    updated_at    TEXT
);

CREATE TABLE events (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    type TEXT,
    ts   TEXT
);
-- type values: 'feed', 'gpio_feed', 'telegram_mood_check', 'telegram_vitals'

CREATE TABLE behavior_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    ts          TEXT,

    -- Layer 1: current physiological state
    hunger                      REAL,
    energy                      REAL,
    happiness                   REAL,
    hours_since_fed             REAL,
    time_of_day                 TEXT,
    day_of_week                 TEXT,

    -- Layer 2a: what the owner did today
    feeds_today                 INTEGER,
    mood_checks_today           INTEGER,
    interactions_today          INTEGER,
    hours_since_any_interaction REAL,

    -- Layer 2b: 10-day memory features
    memory_neglect_days         INTEGER,
    memory_care_trend           REAL,
    memory_max_neglect_streak   INTEGER,
    memory_mood_baseline        REAL,
    memory_days_available       INTEGER,
    memory_daily_pattern        TEXT,

    -- derived relational label
    owner_relationship          TEXT,

    -- previous state for transition learning
    last_state                  TEXT,

    -- outputs (training labels)
    new_state        TEXT,
    pulse_rate       REAL,
    spread           REAL,
    color_hue        INTEGER,
    particle_mode    TEXT,
    lean             REAL,
    inner_experience TEXT,

    -- meta
    decision_source TEXT
    -- values: 'claude_api', 'local_model'
);
```

### Deliverable

Creature visible on screen in idle state. `/feed` in Telegram inserts a row into events.
Rust polls the DB and visibly shifts the creature's energy level (brighter pulse).

---

## Phase 2 — Claude API behavior engine

**Duration:** Weeks 3–5
**Goal:** The creature thinks. Every 10 minutes it makes a behavioral decision driven by
Claude. All 6 emotional states are live. Data logging begins.

### How the behavior engine works

Every 10 minutes the Python behavior service:

1. Runs `build_full_context()` — reads SQLite history and assembles physiological state plus 10-day relational memory into a structured JSON payload
2. Calls `claude-haiku-4-5` with that payload at `temperature=0.85`
3. Parses the JSON response
4. Writes the full context plus decision to `behavior_log` (this is your training dataset)
5. Updates the state table with the new behavioral parameters
6. Rust picks up the state change on its next poll and lerps the creature toward the new target

This runs as a second systemd service alongside the renderer.

### Structured stochasticity — the most important design decision in Phase 2

Pure randomness in Claude's decisions will poison your training dataset. Pure rule-following
will produce a model with no personality depth. You need structured stochasticity — behavior
that is mostly predictable from inputs, occasionally surprising, and always in-character.

The problem with pure randomness: if Claude randomly picks `cascade` when the creature is
well-fed and rested at 2pm with no consistent reason, your model sees the same input vector
sometimes producing `cascade`, sometimes `resonance`, sometimes `void-pull`. It cannot learn
a function from that. The best it can do is learn the marginal distribution — cascade happens
about 16% of the time regardless of inputs — which is useless.

What you need instead is tendencies, not rules. A rule says "if hunger > 0.7 then always
return void-pull." A tendency says "high hunger makes void-pull significantly more likely,
but the creature's recent history and energy level can shift it." The variation is correlated
with context, which is what creates learnable signal. A healthy correlation to aim for is
`r > 0.3` between your strongest feature-state pairs (hunger and void-pull, for example).
Below that you have noise.

There are two levers. The system prompt defines the shape of the tendencies. The API
`temperature=0.85` controls how literally Claude follows the prompt — at `0.0` it is
deterministic, at `1.3+` it starts ignoring weak tendencies. `0.85` gives structured
stochasticity during data collection.

### The system prompt — the creature's soul

This prompt does not change during the 3-month collection period. The personality you
define here is what your ML model will learn to replicate.

```
You are the internal mind of a bioluminescent alien creature called a Voreln.
It does not experience human emotions. It experiences:
  - resonance        (contentment, fullness)
  - void-pull        (hunger, longing)
  - radiance-seeking (curiosity, directed attention)
  - dissolution      (melancholy, low energy)
  - cascade          (excitement, overload)
  - null-state       (idle, baseline)

TENDENCIES, not rules. These should be weighted, not absolute.
High hunger pulls strongly toward void-pull, but if energy is also critically low,
dissolution may override it — the creature is too depleted to even search.
Low energy alone, without hunger, tends toward dissolution or null-state.
Recent excitement or high interactions_today can trigger cascade even from rest.
Time of day matters: the creature is naturally more inward at night and more outward
midday, but strong physiological signals override this tendency.

RELATIONAL MEMORY — owner_relationship is the most important emotional context signal.
It summarises 10 days of care history:
  abandoned              The creature has largely given up on receiving care.
                         Even when fed, it does not easily return to resonance.
  severely_neglected     Deep withdrawal, trust broken. Dissolution dominates.
  increasingly_neglected Growing anxiety. Still some void-pull, less radiance-seeking.
  recovering             Cautious optimism. Emotional wounds heal slowly. Do not jump
                         to resonance immediately. Let it warm over several decisions.
  inconsistently_cared_for  Ambivalent, never fully settles. Flip between states more.
  increasingly_loved     Emerging security. More radiance-seeking, resonance accessible.
  well_cared_for         Baseline contentment. Full emotional range available.

Read memory_daily_pattern (oldest to newest) for the texture of care, not just the
summary label. [2,1,0,2,0,0,0,1,0,0] tells a different story than [0,0,0,0,2,2,2,2,2,2]
even if both have the same neglect_days count.

DEPTH — approximately once every 8 to 10 decisions, the creature may enter a state that
seems slightly disconnected from its current readings, as if responding to something
imperceptible. This should feel like personality depth, not randomness. It should still
be plausible in hindsight given the broader context.

Respond with only valid JSON, no prose, no markdown.
```

### The context builder — what Claude actually receives

This runs before every Claude call. It assembles physiological state (Layer 1) and
10-day relational memory (Layer 2) into a single structured payload.

```python
import sqlite3, json, datetime, numpy as np

def build_full_context(db_path: str) -> dict:
    conn = sqlite3.connect(db_path)
    now  = datetime.datetime.now()

    # Layer 1: current physiological state
    state = conn.execute(
        "SELECT hunger, energy, happiness FROM state ORDER BY id DESC LIMIT 1"
    ).fetchone()

    last_feed = conn.execute(
        "SELECT ts FROM events WHERE type IN ('feed','gpio_feed') "
        "ORDER BY id DESC LIMIT 1"
    ).fetchone()

    hours_since_fed = 99.0
    if last_feed:
        delta = now - datetime.datetime.fromisoformat(last_feed[0])
        hours_since_fed = delta.total_seconds() / 3600

    # Layer 2a: what the owner did today
    today_events = conn.execute(
        "SELECT type FROM events WHERE date(ts) = date('now')"
    ).fetchall()

    feeds_today        = sum(1 for e in today_events if e[0] in ('feed', 'gpio_feed'))
    mood_checks_today  = sum(1 for e in today_events if e[0] == 'telegram_mood_check')
    interactions_today = len(today_events)

    last_event = conn.execute(
        "SELECT ts FROM events ORDER BY id DESC LIMIT 1"
    ).fetchone()
    hours_since_any = 99.0
    if last_event:
        delta = now - datetime.datetime.fromisoformat(last_event[0])
        hours_since_any = delta.total_seconds() / 3600

    # Layer 2b: 10-day memory
    # Pre-fill all 10 days with zero so missing days count as no interaction
    daily_map = {}
    for i in range(10):
        d = (now - datetime.timedelta(days=i)).strftime('%Y-%m-%d')
        daily_map[d] = 0

    rows = conn.execute("""
        SELECT date(ts) as day, COUNT(*) as cnt
        FROM events
        WHERE ts >= date('now', '-10 days')
        AND type IN ('feed', 'gpio_feed')
        GROUP BY date(ts)
    """).fetchall()
    for row in rows:
        daily_map[row[0]] = row[1]

    # Oldest first
    daily_counts = list(reversed([daily_map[d] for d in sorted(daily_map.keys())]))

    neglect_days = sum(1 for d in daily_counts if d == 0)
    care_trend   = (
        float(np.mean(daily_counts[-3:]) - np.mean(daily_counts[:3]))
        if len(daily_counts) >= 6 else 0.0
    )

    max_streak = current_streak = 0
    for d in daily_counts:
        if d == 0:
            current_streak += 1
            max_streak = max(max_streak, current_streak)
        else:
            current_streak = 0

    mood_baseline = conn.execute("""
        SELECT AVG(happiness) FROM behavior_log
        WHERE ts >= date('now', '-10 days')
    """).fetchone()[0] or 0.7

    last_state = conn.execute(
        "SELECT new_state FROM behavior_log ORDER BY id DESC LIMIT 1"
    ).fetchone()

    owner_rel = classify_owner_relationship(neglect_days, care_trend, max_streak)

    return {
        "hunger":                      round(float(state[0]), 2),
        "energy":                      round(float(state[1]), 2),
        "happiness":                   round(float(state[2]), 2),
        "hours_since_fed":             round(hours_since_fed, 1),
        "time_of_day":                 now.strftime("%H:%M"),
        "day_of_week":                 now.strftime("%A"),
        "feeds_today":                 feeds_today,
        "mood_checks_today":           mood_checks_today,
        "interactions_today":          interactions_today,
        "hours_since_any_interaction": round(hours_since_any, 1),
        "memory_neglect_days":         neglect_days,
        "memory_care_trend":           round(care_trend, 2),
        "memory_max_neglect_streak":   max_streak,
        "memory_mood_baseline":        round(float(mood_baseline), 2),
        "memory_days_available":       len(daily_counts),
        "memory_daily_pattern":        daily_counts,
        "owner_relationship":          owner_rel,
        "last_state":                  last_state[0] if last_state else "null-state",
    }


def classify_owner_relationship(neglect_days, care_trend, max_streak):
    if neglect_days >= 7:
        return "abandoned"
    elif max_streak >= 4:
        return "severely_neglected"
    elif neglect_days >= 4 and care_trend < 0:
        return "increasingly_neglected"
    elif neglect_days >= 4 and care_trend > 0.5:
        return "recovering"
    elif neglect_days <= 1 and care_trend >= 0:
        return "well_cared_for"
    elif neglect_days <= 2 and care_trend > 0.5:
        return "increasingly_loved"
    elif neglect_days <= 3:
        return "inconsistently_cared_for"
    else:
        return "neglected"
```

A real payload sent to Claude looks like this:

```json
{
  "hunger": 0.71,
  "energy": 0.45,
  "happiness": 0.38,
  "hours_since_fed": 14.2,
  "time_of_day": "23:40",
  "day_of_week": "Wednesday",
  "feeds_today": 0,
  "mood_checks_today": 1,
  "interactions_today": 1,
  "hours_since_any_interaction": 6.1,
  "memory_neglect_days": 5,
  "memory_care_trend": -0.8,
  "memory_max_neglect_streak": 3,
  "memory_mood_baseline": 0.41,
  "memory_days_available": 10,
  "memory_daily_pattern": [2, 1, 0, 2, 0, 0, 0, 1, 0, 0],
  "owner_relationship": "increasingly_neglected",
  "last_state": "dissolution"
}
```

### The Claude API call and logger

```python
import anthropic, sqlite3, json, datetime

def decide(db_path: str) -> dict:
    context = build_full_context(db_path)
    client  = anthropic.Anthropic()

    msg = client.messages.create(
        model="claude-haiku-4-5-20251001",
        max_tokens=256,
        temperature=0.85,
        system=SYSTEM_PROMPT,
        messages=[{"role": "user", "content": json.dumps(context, indent=2)}]
    )

    result = json.loads(msg.content[0].text)
    log_decision(db_path, context, result)
    return result


def log_decision(db_path: str, ctx: dict, result: dict, source: str = 'claude_api'):
    conn = sqlite3.connect(db_path)
    conn.execute("""
        INSERT INTO behavior_log (
            ts, hunger, energy, happiness, hours_since_fed,
            time_of_day, day_of_week,
            feeds_today, mood_checks_today, interactions_today,
            hours_since_any_interaction,
            memory_neglect_days, memory_care_trend, memory_max_neglect_streak,
            memory_mood_baseline, memory_days_available, memory_daily_pattern,
            owner_relationship, last_state,
            new_state, pulse_rate, spread, color_hue, particle_mode,
            lean, inner_experience, decision_source
        ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)
    """, (
        datetime.datetime.now().isoformat(),
        ctx['hunger'], ctx['energy'], ctx['happiness'],
        ctx['hours_since_fed'], ctx['time_of_day'], ctx['day_of_week'],
        ctx['feeds_today'], ctx['mood_checks_today'], ctx['interactions_today'],
        ctx['hours_since_any_interaction'],
        ctx['memory_neglect_days'], ctx['memory_care_trend'],
        ctx['memory_max_neglect_streak'], ctx['memory_mood_baseline'],
        ctx['memory_days_available'], json.dumps(ctx['memory_daily_pattern']),
        ctx['owner_relationship'], ctx['last_state'],
        result.get('state'), result.get('pulse_rate'), result.get('spread'),
        result.get('color_hue'), result.get('particle_mode'), result.get('lean'),
        result.get('inner_experience'), source,
    ))
    conn.commit()
```

### Expected response schema

```json
{
  "state": "radiance-seeking",
  "inner_experience": "void-pull fading, signal detected at edge of perception",
  "pulse_rate": 1.5,
  "spread": 1.12,
  "color_hue": 165,
  "particle_mode": "directed",
  "lean": -0.6
}
```

### The Rust state machine — physiological drift

Hunger and energy must change automatically over time, otherwise `hours_since_fed` is
meaningless and the context you send to Claude is always the same. Call `tick_state`
every 60 seconds from the systemd render service.

```rust
fn tick_state(state: &mut PetState, delta_hours: f32) {
    state.hunger = (state.hunger + 0.02 * delta_hours).clamp(0.0, 1.0);

    let energy_target = 0.5_f32;
    state.energy += (energy_target - state.energy) * 0.01 * delta_hours;

    if state.hunger > 0.6 || state.energy < 0.3 {
        state.happiness = (state.happiness - 0.01 * delta_hours).clamp(0.0, 1.0);
    }

    write_state_to_db(&state);
}

fn apply_feed_event(state: &mut PetState) {
    state.hunger    = (state.hunger    - 0.60).clamp(0.0, 1.0);
    state.energy    = (state.energy    + 0.30).clamp(0.0, 1.0);
    state.happiness = (state.happiness + 0.15).clamp(0.0, 1.0);
    write_state_to_db(&state);
}
```

### Full data flow

```
You feed via Telegram or GPIO
        |
        v
events table  <--  INSERT (type='feed', ts=now)
        |
        v   Rust polls every 5 sec
apply_feed_event()  -->  state table updated
        |
        v   Every 10 min, Python behavior service
build_full_context()  reads state + events + behavior_log
        |
        v
Claude API call  (temperature=0.85, full context JSON)
        |
        v
Decision JSON returned
        |
        +-->  behavior_log  <--  INSERT (full context + decision + 'claude_api')
        |                        THIS IS YOUR TRAINING DATASET
        +-->  state table   <--  UPDATE current_state and renderer parameters
                  |
                  v   Rust polls
        renderer lerps creature toward new state
```

### Cost estimate

| Interval | Calls/day | Model | Est. cost/day | 3 months total |
|---|---|---|---|---|
| Every 10 min | 144 | Haiku | ~$0.03 | ~$2.70 |
| Every 5 min | 288 | Haiku | ~$0.06 | ~$5.40 |

### Graceful offline mode

If the API call fails, the creature enters a dormant state — dim, slow pulse, no particles.
Log the failure as a `null_decision` row so you can filter it from training data later.
In the lore: the creature has lost signal from whatever cosmic source it feeds on.

### Deliverable

All 6 emotional states working. Creature morphs between them based on Claude decisions.
`behavior_log` accumulating rows with full context attached. Check after 48 hours — you
should have roughly 288 rows of training data already forming.

---

## Phase 3 — Data collection and ML foundations

**Duration:** Weeks 6–12
**Goal:** 10,000+ quality behavior log rows. ML fundamentals understood in parallel.

### Data collection runs itself

The behavior engine service runs unattended. Your only job is to not change the schema
or system prompt. By end of month 2 you will have approximately:

- Every 10 min gives ~144 rows/day, ~9,000 rows over 10 weeks
- Every 5 min gives ~288 rows/day, ~18,000 rows over 10 weeks

Periodically back up the SQLite file:

```bash
scp pi@raspberrypi.local:~/pet.db ./backups/pet_$(date +%Y%m%d).db
```

### Weekly sanity check (5 minutes)

Run this every week starting from Phase 2. The correlation check is the most important
part. Near-zero values after two weeks means your system prompt is producing noise rather
than signal and needs fixing before more data accumulates.

```python
import sqlite3, pandas as pd
from scipy.stats import pointbiserialr

df = pd.read_sql("SELECT * FROM behavior_log", sqlite3.connect("pet.db"))

print(f"Total rows: {len(df)}")
print(df['new_state'].value_counts())
print(df.isnull().sum())

df['is_void_pull']   = (df['new_state'] == 'void-pull').astype(int)
df['is_dissolution'] = (df['new_state'] == 'dissolution').astype(int)
df['is_cascade']     = (df['new_state'] == 'cascade').astype(int)

for feature, state_col in [
    ('hunger',              'is_void_pull'),
    ('energy',              'is_dissolution'),
    ('interactions_today',  'is_cascade'),
    ('memory_neglect_days', 'is_dissolution'),
]:
    r, p = pointbiserialr(df[feature].dropna(), df[state_col].dropna())
    print(f"{feature:30s} -> {state_col:20s}  r={r:.3f}  p={p:.4f}")
```

Target `r > 0.3` for your strongest feature-state pairs. Schema changes are forbidden
after Phase 2 begins — only prompt and temperature adjustments are safe.

---

### ML learning track — 30 to 45 min, 4 days/week

Your problem is a multiclass classification task: given pet state features, predict which
behavioral state the creature enters. You do not need neural networks. But you will
understand exactly why the model works — the math is not optional, it is the point.

Your cryptography background is a genuine advantage. Probability distributions, information
content, and mathematical proofs all appear directly in how decision trees and random forests
work.

#### Week 0 — Mathematical prerequisites

Verify you are comfortable with:

- **Logarithms** — `log₂(p)` appears constantly. If `p = 0.5` then `log₂(0.5) = -1`. You need to read `-p log₂(p)` without friction.
- **Probability basics** — joint probability, conditional probability `P(A|B)`, the law of total probability.
- **Summation notation** — reading `Σᵢ pᵢ log(pᵢ)` without mentally translating.
- **Partial derivatives** — not deeply needed for trees and forests, but essential for gradient boosting and any future neural network work.

If any of these feel shaky, Khan Academy's probability unit covers what you need in about
3 hours. Do not skip this — the math in weeks 3–4 will not click without it.

#### Weeks 1–2 — Information theory: how trees decide where to split

Decision trees are built on one core question: which feature, split at which threshold,
most reduces uncertainty about the label? That requires measuring uncertainty mathematically.
The measure is Shannon entropy — recognisable from information theory in cryptography.

For a node with K classes at probabilities p₁, p₂, ... pₖ:

```
H = -Σᵢ pᵢ log₂(pᵢ)
```

A node where every sample belongs to the same class has H = 0. A node split 50/50 between
two classes has H = 1. The tree finds the split that maximises information gain — the
reduction in entropy after the split:

```
IG(parent, children) = H(parent) - Σⱼ (|Sⱼ| / |S|) · H(Sⱼ)
```

An alternative impurity measure is Gini impurity:

```
Gini = 1 - Σᵢ pᵢ²
```

Gini is cheaper to compute and is scikit-learn's default. Understanding both gives real
intuition for what `criterion='gini'` means when you pass it to the classifier.

Exercise: take a tiny dataset (5 samples, 2 features, 2 classes), manually compute IG for
every possible split on both features, and verify which split the algorithm would choose.
This 20-minute exercise makes everything else concrete.

Resources:
- [StatQuest: Decision Trees](https://www.youtube.com/watch?v=_L39rN6gz7Y) — 18 min
- [StatQuest: Information Gain and Entropy](https://www.youtube.com/watch?v=YtebGVx-Fxw) — 18 min
- ESLII Chapter 9.2

#### Weeks 3–4 — From one tree to a forest: the math of ensembles

A single decision tree overfits. The solution is mathematically elegant.

**The bias-variance decomposition**

```
E[(y - ŷ)²] = Bias² + Variance + Irreducible noise
```

A deep tree has low bias (memorises training data) but high variance (small changes in
training data produce very different trees). Ensembles are the principled escape.

**Bagging**

Create B bootstrap samples by drawing n samples with replacement. Train a tree on each.
Predict by majority vote. For B identically distributed variables with variance σ² and
pairwise correlation ρ:

```
Var(mean) = ρσ² + (1-ρ)σ²/B
```

As B → ∞, variance is determined purely by the correlation between trees, which motivates
the next step.

**Random forests — decorrelating the trees**

At each split, only a random subset of m features is considered (typically m = √p). This
reduces ρ, driving variance lower at the cost of making each individual tree weaker. The
tradeoff is deliberate: each tree is worse in isolation, but the forest is better collectively
because its members make more independent errors.

**Feature importance — the math behind the chart**

```
Importance(j) = (1/B) Σ_trees Σ_{nodes split on j} [ΔImpurity × (n_node / n_total)]
```

The scores sum to 1. Features near tree roots are weighted more because they affect more
samples. Knowing this lets you interpret the chart critically rather than just reading it
as a ranking.

Resources:
- [StatQuest: Bias and Variance](https://www.youtube.com/watch?v=EuBBz3bI-aA) — 7 min
- [StatQuest: Random Forests Part 1](https://www.youtube.com/watch?v=J4Wdy0Wc_xQ) — 10 min
- [StatQuest: Random Forests Part 2](https://www.youtube.com/watch?v=sQ870aTKqiM) — 12 min
- [StatQuest: Feature Importance](https://www.youtube.com/watch?v=v5dqavbyE-I) — 8 min
- ESLII Chapter 15
- Breiman (2001), Random Forests, Machine Learning 45:5–32

#### Weeks 5–6 — Bayesian perspective and probabilistic outputs

A random forest's class probability estimate for sample x is:

```
P̂(Y=k | x) = (1/B) Σ_b 1[T_b(x) = k]
```

This is the fraction of trees that voted for class k. It converges to the true posterior
as B → ∞. Understanding this helps you decide when a 60% probability is actionable and
when it is just noise.

For your pet: if the model outputs 55% void-pull and 45% radiance-seeking, that is a
genuinely uncertain decision. You can sample from the probability distribution rather than
always taking the argmax — making the creature stochastic in uncertain cases and deterministic
in clear ones. The math tells you when to do which.

Resources:
- [StatQuest: Conditional Probability](https://www.youtube.com/watch?v=_IgyaD7vOOA) — 5 min
- [StatQuest: Bayes Theorem](https://www.youtube.com/watch?v=9wCnvr7Xw4E) — 15 min
- ISLR Chapter 4, sections 4.1 and 4.2

#### Weeks 7–8 — Hands-on with toy data, verify the math as you go

```python
from sklearn.datasets import load_iris
from sklearn.ensemble import RandomForestClassifier
from sklearn.model_selection import train_test_split
from sklearn.metrics import classification_report, confusion_matrix
from sklearn.tree import export_text

X, y = load_iris(return_X_y=True, as_frame=True)
X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)

model = RandomForestClassifier(n_estimators=100, random_state=42)
model.fit(X_train, y_train)
preds = model.predict(X_test)

print(classification_report(y_test, preds))
print(confusion_matrix(y_test, preds))

# Print one tree's full decision logic
# Then compute IG manually for the first split and verify it matches
print(export_text(model.estimators_[0], feature_names=list(X.columns)))
```

This is the exercise that makes the math stop being abstract.

#### Weeks 9–10 — Evaluation: precision, recall, F1, and cross-validation

For a single class k in a one-vs-rest frame:

```
Precision = TP / (TP + FP)
Recall    = TP / (TP + FN)
F1        = 2 * (P * R) / (P + R)
```

The harmonic mean is dominated by the smaller value. A model with precision 0.99 and
recall 0.01 has F1 ≈ 0.02, not 0.50. This correctly penalises a degenerate model.

K-fold cross-validation with `cv=5` partitions the dataset into 5 folds, trains on 4 and
evaluates on the held-out fold, 5 times. Every sample appears in the test set exactly once.
The mean score across all 5 evaluations is an unbiased estimate of generalisation error.

Resources:
- [StatQuest: Confusion Matrix](https://www.youtube.com/watch?v=Kdsp6soqA7o) — 8 min
- [StatQuest: Cross Validation](https://www.youtube.com/watch?v=fSytzGwwBVw) — 6 min
- ISLR Section 5.1
- ESLII Section 7.10

#### Reference books — ordered by mathematical depth

**1. The Elements of Statistical Learning (ESLII)** — Hastie, Tibshirani, Friedman
[Free PDF at web.stanford.edu/~hastie/ElemStatLearn](https://web.stanford.edu/~hastie/ElemStatLearn/)
The primary mathematical reference for everything in this project. Dense, rigorous, full of
proofs. Chapters 9, 15, 10, 7. Read it alongside the project, not before — the notation
lands better with a concrete use case.

**2. An Introduction to Statistical Learning (ISLR)** — James, Witten, Hastie, Tibshirani
[Free PDF at statlearning.com](https://www.statlearning.com)
The gentler companion to ESLII by three of the same authors. Use it when ESLII loses you.
Chapters 2, 4, 5, 8.

**3. Hands-On Machine Learning** — Aurélien Géron
The practical reference. Read ISLR or ESLII for the why, read Géron for the how.
Chapters 6 and 7 are the ones directly relevant here.

**4. Pattern Recognition and Machine Learning** — Bishop
The full Bayesian treatment. Not needed for this project but directly relevant to your
cryptography background — Chapter 1 covers probability theory in the same register as a
crypto textbook. Return to this after the project ships.

---

## Phase 4 — Train and deploy your own model

**Duration:** Month 3 and ongoing
**Goal:** Replace the Claude API call with `model.predict()`. The creature runs fully
offline on its own learned personality.

### Step 1 — Explore and clean your data

```python
import sqlite3, pandas as pd

df = pd.read_sql(
    "SELECT * FROM behavior_log WHERE new_state IS NOT NULL AND decision_source = 'claude_api'",
    sqlite3.connect("pet.db")
)

print(df['new_state'].value_counts())
print(df[['hunger','energy','happiness','hours_since_fed']].describe())
```

If any state has fewer than ~200 examples the model will underperform on it. Use
`class_weight='balanced'` in the classifier to compensate.

### Step 2 — Build the feature matrix

```python
import numpy as np

# Cyclical encoding for time of day so 23:50 and 00:10 are close, not far apart
df['hour']     = pd.to_datetime(df['time_of_day'], format='%H:%M').dt.hour
df['hour_sin'] = np.sin(2 * np.pi * df['hour'] / 24)
df['hour_cos'] = np.cos(2 * np.pi * df['hour'] / 24)

# Cyclical encoding for day of week
dow_map = {'Monday':0,'Tuesday':1,'Wednesday':2,'Thursday':3,
           'Friday':4,'Saturday':5,'Sunday':6}
df['dow']     = df['day_of_week'].map(dow_map)
df['dow_sin'] = np.sin(2 * np.pi * df['dow'] / 7)
df['dow_cos'] = np.cos(2 * np.pi * df['dow'] / 7)

# Ordinal encoding for owner_relationship
rel_map = {
    'abandoned':0, 'severely_neglected':1, 'increasingly_neglected':2,
    'neglected':3, 'inconsistently_cared_for':4, 'recovering':5,
    'increasingly_loved':6, 'well_cared_for':7
}
df['owner_rel_enc'] = df['owner_relationship'].map(rel_map).fillna(4)

FEATURES = [
    'hunger', 'energy', 'happiness',
    'hours_since_fed', 'hours_since_any_interaction',
    'feeds_today', 'interactions_today', 'mood_checks_today',
    'memory_neglect_days', 'memory_care_trend',
    'memory_max_neglect_streak', 'memory_mood_baseline',
    'memory_days_available', 'owner_rel_enc',
    'hour_sin', 'hour_cos', 'dow_sin', 'dow_cos',
]

X = df[FEATURES].values
y = df['new_state'].values
```

### Step 3 — Train and evaluate

```python
from sklearn.ensemble import RandomForestClassifier
from sklearn.model_selection import train_test_split, cross_val_score
from sklearn.metrics import classification_report
from sklearn.preprocessing import LabelEncoder
import joblib

le    = LabelEncoder()
y_enc = le.fit_transform(y)

X_train, X_test, y_train, y_test = train_test_split(
    X, y_enc, test_size=0.2, random_state=42, stratify=y_enc
)

model = RandomForestClassifier(
    n_estimators=200,
    max_depth=12,
    class_weight='balanced',
    random_state=42,
    n_jobs=-1
)
model.fit(X_train, y_train)

preds = model.predict(X_test)
print(classification_report(y_test, preds, target_names=le.classes_))

cv_scores = cross_val_score(model, X, y_enc, cv=5, scoring='f1_weighted')
print(f"CV f1: {cv_scores.mean():.3f} +/- {cv_scores.std():.3f}")

joblib.dump(model, 'pet_brain.pkl')
joblib.dump(le,    'label_encoder.pkl')
```

### Step 4 — Feature importance

```python
import matplotlib.pyplot as plt

importances = model.feature_importances_
indices     = np.argsort(importances)[::-1]

plt.figure(figsize=(10, 4))
plt.bar(range(len(FEATURES)), importances[indices])
plt.xticks(range(len(FEATURES)), [FEATURES[i] for i in indices], rotation=45, ha='right')
plt.title("What drives your creature's behavior")
plt.tight_layout()
plt.savefig("feature_importance.png")
```

This chart is the most interesting output of the whole project. It tells you what the
creature — through three months of Claude-generated behavior shaped by your care patterns —
actually cares about most.

### Step 5 — Replace the Claude API call with local inference

```python
import joblib, numpy as np

model = joblib.load('pet_brain.pkl')
le    = joblib.load('label_encoder.pkl')

def decide_local(db_path: str) -> dict:
    ctx  = build_full_context(db_path)
    hour = int(ctx['time_of_day'].split(':')[0])
    dow  = ['Monday','Tuesday','Wednesday','Thursday',
            'Friday','Saturday','Sunday'].index(ctx['day_of_week'])

    rel_map = {
        'abandoned':0, 'severely_neglected':1, 'increasingly_neglected':2,
        'neglected':3, 'inconsistently_cared_for':4, 'recovering':5,
        'increasingly_loved':6, 'well_cared_for':7
    }

    x = np.array([[
        ctx['hunger'], ctx['energy'], ctx['happiness'],
        ctx['hours_since_fed'], ctx['hours_since_any_interaction'],
        ctx['feeds_today'], ctx['interactions_today'], ctx['mood_checks_today'],
        ctx['memory_neglect_days'], ctx['memory_care_trend'],
        ctx['memory_max_neglect_streak'], ctx['memory_mood_baseline'],
        ctx['memory_days_available'], rel_map.get(ctx['owner_relationship'], 4),
        np.sin(2 * np.pi * hour / 24), np.cos(2 * np.pi * hour / 24),
        np.sin(2 * np.pi * dow  / 7),  np.cos(2 * np.pi * dow  / 7),
    ]])

    proba = model.predict_proba(x)[0]

    # Sample from the probability distribution rather than always taking argmax.
    # When confident (e.g. 0.92 void-pull), nearly deterministic.
    # When uncertain (e.g. 0.45 / 0.38 split), genuinely stochastic.
    # This one change makes the model feel alive rather than mechanical.
    chosen_idx   = np.random.choice(len(proba), p=proba)
    chosen_state = le.inverse_transform([chosen_idx])[0]

    result = {**STATE_PARAMS[chosen_state], 'state': chosen_state}
    log_decision(db_path, ctx, result, source='local_model')
    return result
```

`STATE_PARAMS` maps state names to their default renderer parameters — the same lookup
you defined in Phase 2.

### How often the creature refreshes after the model is deployed

The render loop (30–60fps) and the behavior decision loop are completely separate. The
creature is always animating. What changes with each decision is the target state the
renderer lerps toward.

Since local inference takes microseconds, use an event-driven trigger with a heartbeat
fallback rather than a fixed timer:

```python
def should_decide(ctx, prev_ctx, last_decision_ts, last_event_ts) -> bool:
    import datetime

    if abs(ctx['hunger']    - prev_ctx['hunger'])    > 0.15: return True
    if abs(ctx['energy']    - prev_ctx['energy'])    > 0.15: return True
    if abs(ctx['happiness'] - prev_ctx['happiness']) > 0.15: return True

    if last_event_ts:
        event_age = (datetime.datetime.now() - last_event_ts).total_seconds()
        if event_age < 65:
            return True

    minutes_elapsed = (datetime.datetime.now() - last_decision_ts).total_seconds() / 60
    if minutes_elapsed >= 8:
        return True

    return False
```

| Trigger | What causes it | Feel |
|---|---|---|
| Hunger or energy crosses 0.15 threshold | Physiological drift | Creature reacts to its own body |
| Feed or interaction event | You doing something | Immediately responsive to you |
| 8-minute heartbeat | Nothing happening | Still shifts states when ignored |

### What to expect from the trained model vs the Claude phase

The model will feel about 70–80% as alive as the Claude phase. Common states will feel
right — void-pull when hungry, null-state at night, dissolution after sustained neglect.
The memory features mean the model knows the history of your relationship and modulates
accordingly. Consistent neglect produces a creature that is emotionally blunted even when
fed. Consistent care produces one that is more open and exploratory.

What you lose compared to Claude: no genuine reasoning about novel combinations, no
narrative continuity beyond what the feature vector captures.

What you gain: fully offline, zero latency, interpretable, and a personality that is
genuinely yours — distilled from three months of your relationship with the creature.

The probabilistic sampling in `decide_local` does most of the work of recovering liveliness.
A creature where the model is 55/45 uncertain between two states feels genuinely unpredictable.
A creature where the model is 95% confident feels deliberate and grounded. Both feel right.

### What to explore next

**Gradient boosted trees** — where random forests reduce variance by averaging independent
trees, gradient boosting reduces bias by training trees sequentially, each fitting the
residual errors of the previous. The math is rooted in functional gradient descent:
`F_m(x) = F_{m-1}(x) + η · h_m(x)`. ESLII Chapter 10. `xgboost` or
`GradientBoostingClassifier` is a direct drop-in that often outperforms random forests on
small tabular datasets.

**Regression targets alongside classification** — train separate regression models for
`pulse_rate`, `spread`, and `color_hue` instead of just classifying the state. Loss
function changes from cross-entropy to MSE: `L = (1/n) Σᵢ (yᵢ - ŷᵢ)²`. The creature's
behavior becomes continuous rather than jumping between discrete states.

**Probability calibration** — random forest probability estimates are often miscalibrated.
Platt scaling or isotonic regression maps raw `predict_proba` outputs to better-calibrated
probabilities. Relevant if you use the probability vector to drive stochastic behavior.
ESLII Section 10.5.

**Continued collection with the local model** — the creature's personality is now
self-sustaining. Log its decisions, retrain periodically, watch the model drift or stabilise.

---

## Full resource list

### Videos — watch in order listed

| Resource | Topic | Time |
|---|---|---|
| [StatQuest: What is ML?](https://www.youtube.com/watch?v=Gv9_4yMHFhI) | Foundations | 6 min |
| [StatQuest: Training and Test Sets](https://www.youtube.com/watch?v=Zi-0rlM4RDs) | Foundations | 6 min |
| [StatQuest: Bias and Variance](https://www.youtube.com/watch?v=EuBBz3bI-aA) | Core theory | 7 min |
| [StatQuest: Decision Trees](https://www.youtube.com/watch?v=_L39rN6gz7Y) | Core model | 18 min |
| [StatQuest: Information Gain and Entropy](https://www.youtube.com/watch?v=YtebGVx-Fxw) | Math derivation | 18 min |
| [StatQuest: Random Forests Part 1](https://www.youtube.com/watch?v=J4Wdy0Wc_xQ) | Core model | 10 min |
| [StatQuest: Random Forests Part 2](https://www.youtube.com/watch?v=sQ870aTKqiM) | Core model | 12 min |
| [StatQuest: Feature Importance](https://www.youtube.com/watch?v=v5dqavbyE-I) | MDI math | 8 min |
| [StatQuest: Conditional Probability](https://www.youtube.com/watch?v=_IgyaD7vOOA) | Probability | 5 min |
| [StatQuest: Bayes Theorem](https://www.youtube.com/watch?v=9wCnvr7Xw4E) | Probability | 15 min |
| [StatQuest: Confusion Matrix](https://www.youtube.com/watch?v=Kdsp6soqA7o) | Evaluation | 8 min |
| [StatQuest: Cross Validation](https://www.youtube.com/watch?v=fSytzGwwBVw) | Evaluation | 6 min |

Total: approximately 119 minutes spread across 10 weeks.

### Books — ordered by mathematical depth

| Book | Chapters | Notes |
|---|---|---|
| ESLII — Hastie et al. ([free PDF](https://web.stanford.edu/~hastie/ElemStatLearn/)) | 9, 15, 10, 7 | Primary reference. Dense, proofs included. Read alongside project. |
| ISLR — James et al. ([free PDF](https://www.statlearning.com)) | 2, 4, 5, 8 | Gentler companion to ESLII. Use when ESLII loses you. |
| Hands-On ML — Géron | 6, 7 | Practical implementation reference for scikit-learn specifics. |
| PRML — Bishop | 1, 2 | Full Bayesian treatment. Return to this after the project ships. |

Breiman (2001), Random Forests, Machine Learning 45:5–32 — read the original paper once
you finish the ESLII chapter. Primary sources in ML are often surprisingly readable.

### Rust crates

| Crate | Use |
|---|---|
| `pixels` or `minifb` | Framebuffer rendering |
| `rusqlite` with bundled feature | SQLite with no system dependency |
| `serde` and `serde_json` | JSON state deserialization |
| `reqwest` and `tokio` | HTTP if you ever need Rust to call an API |

### Python packages

| Package | Use |
|---|---|
| `python-telegram-bot` | Telegram bot |
| `anthropic` | Claude API |
| `scikit-learn` | All ML training and evaluation |
| `pandas` | Data exploration and cleaning |
| `joblib` | Model serialization |
| `numpy` | Feature engineering |
| `scipy` | Correlation checks in sanity script |
| `matplotlib` | Feature importance chart |

---

## Time budget

| Activity | Time per week | Total over 3 months |
|---|---|---|
| Project coding | 3–4 hrs weekend | ~40 hrs |
| Videos and reading | 3–4 hrs weekday evenings | ~45 hrs |
| Hand derivations and exercises | 1 hr weekend | ~12 hrs |
| Data monitoring sanity check | 5 min weekly | ~1 hr |

Approximately 98 hours total. The math track adds roughly 20 hours over the purely
practical version, but those hours compound — every formula you derive by hand makes
the code more transparent and the model's failures easier to diagnose.

The single most important habit: when something in scikit-learn surprises you —
unexpected accuracy, a weird feature importance score, a class the model never predicts —
go to ESLII first rather than Stack Overflow. The answer is almost always in the math.
