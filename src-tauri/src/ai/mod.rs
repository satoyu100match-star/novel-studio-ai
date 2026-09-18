//! AI基盤(Phase 5, docs/AI.md)。
//!
//! 他レイヤー(repositories/commands以外)からこのモジュールへ直接依存
//! させない(CLAUDE.md「AI関連は独立レイヤー」方針)。AI機能が利用不能
//! でも通常の執筆機能が動作することは、このモジュールを一切importしない
//! Phase 0〜4のコードパスがそのまま残っていることで保証される。

pub mod analysis;
pub mod anthropic;
pub mod context_builder;
pub mod features;
pub mod gemini;
pub mod http_client;
pub mod keystore;
pub mod openai_compatible;
pub mod provider;
