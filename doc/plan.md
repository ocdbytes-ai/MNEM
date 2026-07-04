# Mnem AI Model Plan

This document describes the AI architecture for Mnem: a persistent entity whose behavior
changes during the day and whose personality consolidates nightly based on how it has
been treated over the last few days.

The root `Plan.md` describes the full hardware and application roadmap. This file focuses
only on the AI model design, data schema, training loop, and ML study topics.

## Goal

Mnem should not only react to the current input. It should accumulate a personal history.

If it is neglected for 7 days, its loneliness should increase, trust should fall, and its
future behavior should change even after a single feed event. If it is cared for
consistently, it should become more trusting, curious, and emotionally open.

The clean design is:

```text
daytime state updates = short-term reactions
nightly consolidation = memory becoming personality
model weights/adapter = learned individual response pattern
```

## Core Principle

Do not rely on only one mechanism.

```text
runtime state
  What the entity feels right now.

persistent personality vector
  What this individual entity has become over time.

model weights
  The learned behavior function that maps state + personality + event to action.
```

This separation matters. If memory exists only inside model weights, the system becomes
hard to inspect and easy to destabilize. If memory exists only as hand-written columns,
the system feels rule-based. Mnem should use both:

```text
explicit personality values for inspectable continuity
trainable embedding/adapter weights for learned individuality
```

## Runtime Architecture

```text
Telegram / GPIO / system events
        |
        v
events table
        |
        v
daytime state updater
        |
        v
entity_state table
        |
        v
behavior model inference
        |
        +--> behavior_log table
        |
        v
renderer state update

nightly:
events + behavior_log + entity_state history
        |
        v
daily summary builder
        |
        v
nightly consolidation model
        |
        +--> personality table update
        +--> personality embedding update
        +--> optional personal adapter fine-tune
        +--> training_runs / model_versions log
```

## Model Architecture

Use a small PyTorch MLP with a trainable personality embedding and multiple output heads.

Do not start with a Transformer or LSTM. Mnem's early data is structured tabular data, not
large text or long sequence data. A small MLP is easier to train, inspect, and deploy on a
Raspberry Pi.

### Recommended V1 Model

```text
numeric features, 20-50 dims
categorical event embeddings
personality scalar values
personality embedding, 8-16 dims
        |
        v
concat
        |
        v
Linear(64) -> ReLU -> LayerNorm
Linear(64) -> ReLU
        |
        +--> behavior_state_head
        |      outputs probability over states
        |
        +--> affect_delta_head
        |      outputs changes to mood/affect values
        |
        +--> visual_param_head
               outputs renderer parameters
```

### Inputs

Physiology:

```text
hunger
energy
fatigue
stimulation
```

Current affect:

```text
happiness
loneliness
trust
resentment
frustration
curiosity
attachment
fear
calm
```

Current context:

```text
latest_event_type
event_intensity
hour_sin
hour_cos
day_sin
day_cos
minutes_since_last_interaction
minutes_since_last_feed
last_behavior_state
```

Persistent personality:

```text
baseline_loneliness
baseline_trust
baseline_resentment
baseline_attachment
sensitivity_to_neglect
forgiveness_rate
novelty_seeking
attention_seeking
emotional_volatility
recovery_speed
```

Trainable personality embedding:

```text
entity_embedding[8-16]
```

### Outputs

Behavior classifier:

```text
P(null-state)
P(resonance)
P(void-pull)
P(radiance-seeking)
P(dissolution)
P(cascade)
P(guarded)
P(recovering)
```

Affect delta regressor:

```text
delta_happiness
delta_loneliness
delta_trust
delta_resentment
delta_frustration
delta_curiosity
delta_attachment
delta_fear
delta_calm
```

Visual parameter regressor:

```text
pulse_rate
spread
color_hue
brightness
particle_density
lean
motion_jitter
```

## Why This Model

### Why Not Random Forest First

Random forests are excellent for tabular data, but they do not naturally support smooth
nightly fine-tuning. You can retrain them nightly, but there is no clean notion of a small
personal weight update.

Use a random forest later as a baseline model. It is useful for checking whether the MLP is
learning anything meaningful.

### Why Not A Large Neural Net

The dataset will be small at first. A large model will overfit, drift, and become hard to
debug. Mnem needs stable personality accumulation, not raw capacity.

### Why A Small MLP

A small MLP gives:

```text
fast inference on Raspberry Pi
nightly fine-tuning support
multi-output prediction
trainable personality embedding
simple deployment
simple debugging
```

## Nightly Consolidation

Nightly consolidation is the important part of this design.

The model should not merely predict behavior. It should slowly update what this individual
entity is becoming.

### Nightly Inputs

Build a daily or rolling summary from the last 1, 3, and 7 days:

```text
feeds_1d
feeds_3d
feeds_7d
interactions_1d
interactions_3d
interactions_7d
negative_events_7d
neglect_streak_days
avg_hunger_7d
avg_happiness_7d
avg_loneliness_7d
avg_frustration_7d
avg_trust_7d
dominant_behavior_7d
```

These summaries are not the entity's memory by themselves. They are the night's evidence.
The result of consolidation is written into the persistent personality values and the
trainable embedding.

### Nightly Outputs

```text
delta_baseline_loneliness
delta_baseline_trust
delta_baseline_resentment
delta_baseline_attachment
delta_sensitivity_to_neglect
delta_forgiveness_rate
delta_novelty_seeking
delta_attention_seeking
delta_emotional_volatility
delta_recovery_speed
```

Apply the deltas slowly:

```text
personality = clamp(personality + nightly_delta * consolidation_rate, 0.0, 1.0)
```

Start with:

```text
consolidation_rate = 0.15
```

This prevents one strange day from rewriting the creature.

### Nightly Weight Update

Use a personal adapter strategy:

```text
base model
  Mostly fixed. Represents species-level behavior.

personal adapter
  Small final layer or low-rank adapter updated nightly.

personality embedding
  Updated nightly. Represents this individual entity.
```

At first, update only:

```text
personality embedding
personal adapter head
```

Avoid updating the full model every night until the data and evaluation are solid.

### Replay Buffer

Nightly fine-tuning must avoid forgetting.

Training mix:

```text
70 percent historical replay samples
30 percent recent samples from last few days
```

Sample weights:

```text
recent 1 day: 3.0
recent 3 days: 2.0
older data: 1.0
synthetic seed data: 0.5
```

This lets recent treatment matter without destroying the original personality structure.

## Proposed SQLite Schema

The schema should preserve raw events, current state, personality, model versions, and
training provenance. Raw events should never be overwritten.

### entity_state

Current runtime condition.

```sql
CREATE TABLE entity_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    hunger REAL NOT NULL DEFAULT 0.0,
    energy REAL NOT NULL DEFAULT 1.0,
    fatigue REAL NOT NULL DEFAULT 0.0,
    stimulation REAL NOT NULL DEFAULT 0.0,

    happiness REAL NOT NULL DEFAULT 0.6,
    loneliness REAL NOT NULL DEFAULT 0.2,
    trust REAL NOT NULL DEFAULT 0.5,
    resentment REAL NOT NULL DEFAULT 0.0,
    frustration REAL NOT NULL DEFAULT 0.0,
    curiosity REAL NOT NULL DEFAULT 0.5,
    attachment REAL NOT NULL DEFAULT 0.3,
    fear REAL NOT NULL DEFAULT 0.0,
    calm REAL NOT NULL DEFAULT 0.5,

    current_behavior TEXT NOT NULL DEFAULT 'null-state',
    updated_at TEXT NOT NULL
);
```

### personality

Slow-moving individual traits.

```sql
CREATE TABLE personality (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    baseline_loneliness REAL NOT NULL DEFAULT 0.2,
    baseline_trust REAL NOT NULL DEFAULT 0.5,
    baseline_resentment REAL NOT NULL DEFAULT 0.0,
    baseline_attachment REAL NOT NULL DEFAULT 0.3,
    sensitivity_to_neglect REAL NOT NULL DEFAULT 0.5,
    forgiveness_rate REAL NOT NULL DEFAULT 0.5,
    novelty_seeking REAL NOT NULL DEFAULT 0.5,
    attention_seeking REAL NOT NULL DEFAULT 0.5,
    emotional_volatility REAL NOT NULL DEFAULT 0.4,
    recovery_speed REAL NOT NULL DEFAULT 0.5,

    embedding_json TEXT NOT NULL DEFAULT '[0,0,0,0,0,0,0,0]',
    model_version_id INTEGER,
    updated_at TEXT NOT NULL
);
```

### events

Raw interaction log.

```sql
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TEXT NOT NULL,
    type TEXT NOT NULL,
    source TEXT NOT NULL,
    intensity REAL NOT NULL DEFAULT 1.0,
    metadata_json TEXT
);
```

Example event types:

```text
feed
gpio_feed
mood_check
vitals_check
touch
ignore_tick
system_tick
error
```

### behavior_log

Inference log and training dataset.

```sql
CREATE TABLE behavior_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TEXT NOT NULL,

    input_json TEXT NOT NULL,
    personality_json TEXT NOT NULL,
    model_version_id INTEGER,

    chosen_behavior TEXT NOT NULL,
    behavior_prob_json TEXT NOT NULL,

    predicted_affect_delta_json TEXT NOT NULL,
    applied_affect_delta_json TEXT NOT NULL,
    visual_params_json TEXT NOT NULL,

    decision_source TEXT NOT NULL,
    training_eligible INTEGER NOT NULL DEFAULT 1
);
```

`decision_source` values:

```text
llm_teacher
local_model
rule_fallback
manual_label
```

### daily_summary

Nightly aggregation input.

```sql
CREATE TABLE daily_summary (
    day TEXT PRIMARY KEY,
    feeds INTEGER NOT NULL,
    interactions INTEGER NOT NULL,
    negative_events INTEGER NOT NULL,
    neglect_score REAL NOT NULL,
    avg_hunger REAL NOT NULL,
    avg_happiness REAL NOT NULL,
    avg_loneliness REAL NOT NULL,
    avg_frustration REAL NOT NULL,
    avg_trust REAL NOT NULL,
    dominant_behavior TEXT,
    summary_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);
```

### nightly_consolidation_log

Records personality changes.

```sql
CREATE TABLE nightly_consolidation_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TEXT NOT NULL,
    window_start TEXT NOT NULL,
    window_end TEXT NOT NULL,

    personality_before_json TEXT NOT NULL,
    summary_input_json TEXT NOT NULL,
    predicted_delta_json TEXT NOT NULL,
    personality_after_json TEXT NOT NULL,

    model_version_before INTEGER,
    model_version_after INTEGER,
    notes TEXT
);
```

### model_versions

Track every deployed model.

```sql
CREATE TABLE model_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT NOT NULL,
    model_type TEXT NOT NULL,
    path TEXT NOT NULL,
    train_rows INTEGER NOT NULL,
    validation_loss REAL,
    behavior_f1 REAL,
    affect_mse REAL,
    visual_mse REAL,
    metadata_json TEXT
);
```

### training_runs

Track training provenance.

```sql
CREATE TABLE training_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    run_type TEXT NOT NULL,
    dataset_query TEXT NOT NULL,
    train_rows INTEGER NOT NULL,
    recent_weight REAL NOT NULL,
    replay_weight REAL NOT NULL,
    status TEXT NOT NULL,
    metrics_json TEXT,
    error TEXT
);
```

## Training Phases

### Phase A: Synthetic Teacher Data

Use an LLM to generate labeled examples before real data exists.

Generate scenarios across:

```text
well cared for
ignored for 1 day
ignored for 3 days
ignored for 7 days
fed after neglect
overfed
frequent mood checks without feeding
high interaction day
late night care
inconsistent care
recovery after neglect
```

The LLM should output:

```text
behavior probabilities
chosen behavior
affect deltas
visual parameters
nightly personality deltas
```

### Phase B: Supervised Pretraining

Train the base MLP on synthetic and early LLM-teacher rows.

Loss:

```text
total_loss =
    behavior_cross_entropy
  + 0.5 * affect_delta_mse
  + 0.25 * visual_param_mse
```

### Phase C: Live Collection

Run the LLM teacher or local model during normal use and log every decision. Keep all raw
events and all behavior logs.

### Phase D: Nightly Personalization

Each night:

```text
1. create daily_summary
2. compute rolling 1d/3d/7d evidence
3. update persistent personality values
4. fine-tune personality embedding and personal adapter
5. evaluate against holdout/replay data
6. save a new model version only if metrics do not regress badly
```

### Phase E: Offline Mode

Once local behavior is stable, stop relying on LLM inference. Keep optional LLM generation
only for experiments and data augmentation.

## Evaluation

Track these metrics after every nightly run:

```text
behavior accuracy
macro F1
per-class F1
affect delta MSE
visual parameter MSE
probability calibration error
personality drift magnitude
```

Important checks:

```text
The model should not always choose the most common behavior.
Neglect should increase loneliness/trust-related outputs.
Care after neglect should not instantly reset personality.
Repeated care should gradually improve trust and attachment.
Nightly updates should be small but visible over days.
```

## First Implementation Milestone

Build the simplest working version:

```text
1. Add SQLite schema.
2. Add daytime state updater.
3. Add behavior_log writer.
4. Generate 5000 synthetic LLM teacher rows.
5. Train small PyTorch MLP.
6. Run local inference from current SQLite state.
7. Add nightly consolidation that updates only personality scalar values.
8. Add embedding/adapter fine-tuning after scalar consolidation works.
```

Do not start with full online learning. First make the data, schema, and evaluation solid.

## ML Study Topics

Study in this order.

### 1. Data Representation

You need to understand:

```text
feature vectors
normalization
categorical encoding
embeddings
train/validation/test split
data leakage
```

Why it matters: Mnem's behavior depends on mixed numeric and categorical inputs.

### 2. Probability And Classification

You need to understand:

```text
softmax
cross-entropy loss
class imbalance
precision
recall
F1 score
confusion matrix
calibrated probabilities
```

Why it matters: the behavior head outputs probabilities, not just one hard state.

### 3. Regression

You need to understand:

```text
mean squared error
mean absolute error
multi-output regression
scaling targets
clamping outputs
```

Why it matters: affect deltas and visual parameters are continuous values.

### 4. Neural Network Basics

You need to understand:

```text
linear layers
activation functions
ReLU
backpropagation
gradient descent
learning rate
batch size
epochs
overfitting
regularization
```

Why it matters: nightly personality updates are weight updates.

### 5. PyTorch Fundamentals

You need to be comfortable with:

```text
Dataset
DataLoader
nn.Module
forward()
loss.backward()
optimizer.step()
model.eval()
torch.no_grad()
saving and loading state_dict
```

Why it matters: PyTorch will own the behavior model and nightly training.

### 6. Embeddings And Personalization

You need to understand:

```text
embedding vectors
trainable parameters
freezing layers
fine-tuning only part of a model
adapter layers
catastrophic forgetting
replay buffers
```

Why it matters: this is the core of the personality system.

### 7. Multi-Task Learning

You need to understand:

```text
shared trunk
multiple heads
weighted losses
loss balancing
task interference
```

Why it matters: one model predicts behavior, affect changes, and visual parameters.

### 8. Time And Memory Modeling

You need to understand:

```text
rolling summaries
exponential decay
state-space thinking
short-term vs long-term memory
nightly consolidation
```

Why it matters: Mnem's personality changes over days, not just per event.

### 9. Model Evaluation And Safety Rails

You need to understand:

```text
holdout sets
replay evaluation
regression tests for models
drift detection
rollback to previous model version
metric thresholds
```

Why it matters: nightly training can make the model worse unless every run is measured.

## Suggested Learning Path

Week 1:

```text
features, normalization, train/test split, confusion matrix
```

Week 2:

```text
softmax, cross-entropy, regression losses
```

Week 3:

```text
PyTorch basics, simple MLP on toy data
```

Week 4:

```text
multi-head MLP: classification + regression
```

Week 5:

```text
embeddings, freezing layers, fine-tuning
```

Week 6:

```text
replay buffers, catastrophic forgetting, weighted sampling
```

Week 7:

```text
nightly consolidation prototype
```

Week 8:

```text
evaluation, rollback, model versioning
```

## Design Rules

Keep these constraints throughout the project:

```text
Raw events are immutable.
Never overwrite training logs.
Every model decision must be logged with model_version_id.
Every nightly personality change must be explainable from stored inputs.
Do not fine-tune the whole model until adapter training is stable.
Do not deploy a nightly model if validation metrics regress badly.
Keep personality deltas small.
Prefer inspectable state plus small learned updates over opaque large models.
```

## Summary

The recommended AI architecture is:

```text
small PyTorch MLP
multi-head outputs
persistent personality vector
trainable personality embedding
personal adapter updated nightly
replay buffer to prevent forgetting
SQLite logs for all state, events, decisions, training runs, and model versions
```

This gives Mnem a practical version of memory consolidation: daily experiences update
short-term state, and nightly training slowly turns repeated treatment patterns into
personality.
