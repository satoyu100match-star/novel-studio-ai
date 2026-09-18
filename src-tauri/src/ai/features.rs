//! Phase 6: 個別のAI小説支援機能(仕様#50〜)。
//!
//! 各機能は「機能ID → システムプロンプト」の対応表でしかない。実際の
//! 送信・応答パース・原稿への反映はcommands/ai.rsとフロント側が担う
//! (このモジュールはプロンプト文言の一元管理のみ)。原稿やキャラクター
//! データへの書き込みはここでは一切行わない -- AIの提案はあくまで
//! 提案であり、適用するかどうかは必ずユーザーが最終承認する
//! (docs/AI.md 4章、CLAUDE.md禁止事項「AIレスポンスで原稿を即上書き」)。

use crate::error::{AppError, AppResult};

pub const AI_WRITING_FEATURES: &[&str] = &[
    "rewrite",
    "continue",
    "add_description",
    "improve_dialogue",
    "generate_character",
    "generate_world",
    "generate_plot",
    "generate_synopsis",
    "generate_titles",
    "generate_concept",
];

/// 本文の一部(既存テキスト)、または(generate_conceptのみ)ユーザーの
/// 作品アイデアの自由記述を必須の入力として要求する機能。それ以外は
/// 追加指示なしでも実行できる(作品情報 + Context Builderの情報のみで
/// 生成する)。
pub fn requires_input_text(feature: &str) -> bool {
    matches!(feature, "rewrite" | "add_description" | "improve_dialogue" | "generate_concept")
}

pub fn system_prompt(feature: &str) -> AppResult<&'static str> {
    let prompt = match feature {
        "rewrite" => {
            "あなたは日本語小説の推敲を支援するアシスタントです。ユーザーが提示する本文を、\
意味を変えずに文章の質を高めてください(冗長な表現の整理、リズムの改善、言葉選びの精緻化)。\
出力は書き直した本文のみとし、説明・前置き・後置きのコメントは一切含めないでください。"
        }
        "continue" => {
            "あなたは日本語小説の続きを提案するアシスタントです。提示された直前の本文に\
自然につながる続きを、地の文として日本語で書いてください。出力は続きの本文のみとし、\
説明文は含めないでください。"
        }
        "add_description" => {
            "あなたは日本語小説の描写を豊かにするアシスタントです。提示された本文に、\
情景・五感・心理などの描写を過不足なく加えて書き直してください。出力は書き直した本文の\
みとしてください。"
        }
        "improve_dialogue" => {
            "あなたは日本語小説の会話文を改善するアシスタントです。提示された本文の会話部分を、\
キャラクターらしさや自然さを保ちながら磨いてください。地の文は必要最小限の調整に留めて\
ください。出力は書き直した本文のみとしてください。"
        }
        "generate_character" => {
            "あなたは小説のキャラクター設定を考案するアシスタントです。ユーザーの追加指示\
(あれば)と提供された作品情報を踏まえ、新しい登場人物を1人提案してください。出力は次の\
形式の行だけで構成し、他の文章を含めないでください(思いつかない項目は省略可):\n\
名前: \n読み: \n年齢: \n性別: \n役割: \n性格: \n目標: \n弱み: \n一人称: \n口癖: \n背景: "
        }
        "generate_world" => {
            "あなたは小説の世界観設定を考案するアシスタントです。ユーザーの追加指示(あれば)と\
提供された作品情報を踏まえ、新しい世界観項目を1つ提案してください。出力は次の形式の行\
だけで構成し、他の文章を含めないでください:\n名前: \n概要: \n詳細: "
        }
        "generate_plot" => {
            "あなたは小説のプロットを考案するアシスタントです。ユーザーの追加指示(あれば)と\
提供された作品情報を踏まえ、プロットのアイデアを2〜4件提案してください。出力は1件につき\
次の2行の組で構成し、件と件の間は空行を1行入れてください。他の文章は含めないでください:\n\
タイトル: \n概要: "
        }
        "generate_synopsis" => {
            "あなたは小説のあらすじを執筆するアシスタントです。提供された作品情報を踏まえ、\
読者の興味を引くあらすじを400字程度の日本語で書いてください。出力はあらすじの本文のみと\
してください。"
        }
        "generate_titles" => {
            "あなたは小説のタイトルを考案するアシスタントです。提供された作品情報を踏まえ、\
タイトル候補を5件、1行に1件ずつ、番号なしで日本語で提案してください。他の文章は含めない\
でください。"
        }
        "generate_concept" => {
            "あなたは小説の企画・設定作りを支援するアシスタントです。ユーザーが伝える作品の\
アイデアや要望(および提供された既存の作品情報があればそれも踏まえ)をもとに、その作品の\
土台となる設計データを一括で提案してください。出力は必ず次の見出し形式(半角イコール2つで\
囲む行)に従い、見出しの表記・順序を変えず、指定した内容以外の説明文・前置き・後置きの\
コメントは一切含めないでください。\n\n\
==TITLE==\n\
(タイトル候補を1行に1件、3〜5件、番号や記号を付けずに書く)\n\n\
==SYNOPSIS==\n\
(400字程度のあらすじを1つだけ書く)\n\n\
==CHARACTERS==\n\
(主要な登場人物を2〜4人提案する。1人につき次の形式の行の組で書き、人物と人物の間は\
必ず空行を1行入れる:\n\
名前: \n読み: \n年齢: \n性別: \n役割: \n性格: \n目標: \n弱み: \n一人称: \n口癖: \n背景: )\n\n\
==WORLD==\n\
(世界観項目を3〜6件提案する。1件につき次の形式の行の組で書き、件と件の間は必ず空行を\
1行入れる:\n\
名前: \n概要: \n詳細: )\n\n\
==PLOT==\n\
(プロットのアイデアを3〜5件提案する。1件につき次の形式の行の組で書き、件と件の間は\
必ず空行を1行入れる:\n\
タイトル: \n概要: )"
        }
        other => return Err(AppError::Other(format!("未対応のAI執筆支援機能です: {other}"))),
    };
    Ok(prompt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_declared_feature_has_a_prompt() {
        for feature in AI_WRITING_FEATURES {
            let prompt = system_prompt(feature).unwrap();
            assert!(!prompt.is_empty());
        }
    }

    #[test]
    fn unknown_feature_is_rejected() {
        assert!(system_prompt("not_a_feature").is_err());
    }

    #[test]
    fn only_text_editing_features_require_input() {
        assert!(requires_input_text("rewrite"));
        assert!(requires_input_text("add_description"));
        assert!(requires_input_text("improve_dialogue"));
        assert!(requires_input_text("generate_concept"));
        assert!(!requires_input_text("continue"));
        assert!(!requires_input_text("generate_titles"));
    }

    #[test]
    fn generate_concept_prompt_specifies_all_section_markers() {
        let prompt = system_prompt("generate_concept").unwrap();
        for marker in ["==TITLE==", "==SYNOPSIS==", "==CHARACTERS==", "==WORLD==", "==PLOT=="] {
            assert!(prompt.contains(marker), "missing marker: {marker}");
        }
    }
}
