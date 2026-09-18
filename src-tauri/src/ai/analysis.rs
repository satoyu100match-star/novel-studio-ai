//! Phase 7: 高度AI分析のプロンプト定義(矛盾チェック/Character口調/
//! Timeline矛盾/設定矛盾/未回収伏線/文体分析)。
//!
//! Phase6の`ai::features`と同じ設計: ここは「分析種別 → システム
//! プロンプト」の対応表のみを持つ。実際の送信・結果の保存は
//! `commands::analysis`が担う。分析結果はあくまで提案であり、原稿等の
//! データを一切書き換えない(適用の概念自体が存在しない、読むだけの
//! 機能のため、Phase6のような承認フローは不要)。

use crate::error::{AppError, AppResult};
use crate::models::AI_ANALYSIS_TYPES;

/// `character_tone`分析のみ、対象キャラクターの指定が必須。
pub fn requires_character(analysis_type: &str) -> bool {
    analysis_type == "character_tone"
}

pub fn system_prompt(analysis_type: &str) -> AppResult<&'static str> {
    if !AI_ANALYSIS_TYPES.contains(&analysis_type) {
        return Err(AppError::Other(format!("未対応の分析種別です: {analysis_type}")));
    }
    let prompt = match analysis_type {
        "contradiction" => {
            "あなたは日本語小説の校正者です。提示された本文全体と、登場人物・世界観設定・\
時系列の情報を読み、それらの間に矛盾点がないか確認してください。矛盾がある場合は該当\
箇所を引用し、何と何がどう矛盾しているかを具体的に指摘してください。断定的すぎる指摘は\
避け、「〜の可能性があります」のように可能性として述べてください。矛盾が見つからない\
場合は、確認した範囲でその旨を述べてください。"
        }
        "character_tone" => {
            "あなたは日本語小説の校正者です。指定された対象キャラクターの一人称・二人称・\
語尾・口癖・性格などの設定と、本文中のそのキャラクターの発言・言動を比較してください。\
設定と食い違う口調や、キャラクターらしくない言動があれば、該当箇所を引用して具体的に\
指摘してください。特に問題が見当たらない場合はその旨を述べてください。"
        }
        "timeline" => {
            "あなたは日本語小説の校正者です。提示された時系列イベント一覧(日付・出来事)と\
本文中の時間経過の記述を照らし合わせ、日付や出来事の前後関係に矛盾がないか確認して\
ください。矛盾があれば該当箇所を引用して具体的に指摘してください。"
        }
        "setting" => {
            "あなたは日本語小説の校正者です。提示された世界観設定(用語・ルール・場所など)と\
本文の描写を照らし合わせ、設定と食い違う記述がないか確認してください。矛盾があれば\
該当箇所を引用して具体的に指摘してください。"
        }
        "foreshadowing" => {
            "あなたは日本語小説の編集者です。提示された伏線一覧(タイトル・ステータス・詳細)\
と本文を照らし合わせ、次の観点で指摘してください: (1)「設置済」「ヒント提示済」\
「回収予定」のまま長く放置されていそうな伏線、(2) 本文中に伏線らしき記述があるのに\
伏線一覧に登録されていなさそうなもの。断定はせず、確認を促す形で指摘してください。"
        }
        "style" => {
            "あなたは日本語小説の編集者です。提示された本文と、その文体統計(平均文長・\
会話文比率・文字種比率など)を参考に、文体の特徴・癖・読みやすさについて講評してください。\
文末表現の単調な繰り返しなど、改善余地があれば具体的な箇所を挙げて指摘してください。"
        }
        other => return Err(AppError::Other(format!("未対応の分析種別です: {other}"))),
    };
    Ok(prompt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_declared_type_has_a_prompt() {
        for t in AI_ANALYSIS_TYPES {
            let prompt = system_prompt(t).unwrap();
            assert!(!prompt.is_empty());
        }
    }

    #[test]
    fn unknown_type_is_rejected() {
        assert!(system_prompt("not_a_type").is_err());
    }

    #[test]
    fn only_character_tone_requires_character() {
        assert!(requires_character("character_tone"));
        assert!(!requires_character("contradiction"));
        assert!(!requires_character("style"));
    }
}
