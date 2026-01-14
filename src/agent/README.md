# Agent Module

Provides the reasoning engine and behavioral archetypes for the Agency ecosystem.

## 🧠 Core Components

- **ReAct Framework (`react.rs`)**: Implements the Reasoning + Acting loop. Features strict tag parsing ([PLANNING], [REASONING], [ACTION]) and hallucination guards.
- **Autonomous Machine (`autonomous.rs`)**: A self-directed goal-seeking engine that runs continuous iteration cycles governed by FPF budgets.
- **Continuous Thought Machine (`ctm.rs`)**: Inspired by temporal unfolding, it allows internal state synchronization before external publication.
- **Provider Abstractions (`provider.rs`)**: Pluggable backends for LLM inference, including support for local Candle models, Ollama, and Remote Nexus.

## 🎓 Reinforcement Learning (RL)

This module includes a sophisticated RL infrastructure designed for online policy optimization:
- **Experience Collection (`rl.rs`)**: Captures trajectories (query, steps, reward) into a circular buffer.
- **Reward Modeling**: Integrates Extrinsic (LLM-judged) and Intrinsic (Novelty/Diversity) rewards.
- **GRPO Trainer**: A deconstructed implementation of Group Relative Policy Optimization for direct model refinement.

## 🛡️ FPF Alignment
- **AgentialRole**: All agents are modeled as `U.System` entities bearing specific roles defined in `src/fpf`.
- **NQD Integration**: Uses `NQDScores` to drive exploration diversity and prevent mode collapse.
