//! 可読性分析(仕様の文体・可読性分析のうちAI不要な部分)。
//!
//! 意図的に `ai` モジュールの外に置いている: 平均文長・会話文比率・
//! 文字種比率などは単なる文字列集計であり、AI機能が未設定/オフラインの
//! ユーザーでも常に使えるべきだからである(CLAUDE.md「AI機能が利用不能
//! でも通常の小説執筆機能は動作すること」)。AIによる定性的な講評
//! (`ai::analysis`の"style"分析)は、ここで計算した統計値をContext
//! Builder経由でAIに渡す形で連携する。

use crate::models::ReadabilityStats;

/// 100字を超える文を「長文」として数える(目安。仕様上の厳密な閾値定義
/// はないため、一般的な可読性ガイドラインの目安値を採用)。
const LONG_SENTENCE_THRESHOLD: usize = 100;

fn is_kanji(c: char) -> bool {
    matches!(c, '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}')
}

fn is_hiragana(c: char) -> bool {
    matches!(c, '\u{3040}'..='\u{309F}')
}

fn is_katakana(c: char) -> bool {
    matches!(c, '\u{30A0}'..='\u{30FF}')
}

/// 全角/半角カギ括弧「」『』の中にある文字数を数える(会話文の簡易判定)。
/// ネストは考慮しない(小説本文で二重鉤括弧が閉じないまま次の発話に
/// 入るケースは稀という前提の簡易実装)。
fn dialogue_char_count(text: &str) -> i64 {
    let mut depth = 0i32;
    let mut count = 0i64;
    for c in text.chars() {
        match c {
            '「' | '『' => {
                depth += 1;
            }
            '」' | '』' => {
                depth = (depth - 1).max(0);
            }
            _ => {
                if depth > 0 {
                    count += 1;
                }
            }
        }
    }
    count
}

/// 句点(。/!/?/！/？)で文を分割する。改行のみで終わる断片(空行等)は
/// 文としてカウントしない。
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for c in text.chars() {
        current.push(c);
        if matches!(c, '。' | '！' | '？' | '!' | '?') {
            sentences.push(std::mem::take(&mut current));
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        sentences.push(current);
    }
    sentences.into_iter().filter(|s| !s.trim().is_empty()).collect()
}

pub fn compute(text: &str) -> ReadabilityStats {
    let total_chars = text.chars().count() as i64;
    if total_chars == 0 {
        return ReadabilityStats::default();
    }

    let sentences = split_sentences(text);
    let sentence_count = sentences.len() as i64;

    let mut max_sentence_length = 0i64;
    let mut long_sentence_count = 0i64;
    let mut total_sentence_chars = 0i64;
    let mut total_touten = 0i64;
    for s in &sentences {
        let len = s.chars().count();
        total_sentence_chars += len as i64;
        if len as i64 > max_sentence_length {
            max_sentence_length = len as i64;
        }
        if len > LONG_SENTENCE_THRESHOLD {
            long_sentence_count += 1;
        }
        total_touten += s.chars().filter(|&c| c == '、').count() as i64;
    }
    let avg_sentence_length = if sentence_count > 0 { total_sentence_chars as f64 / sentence_count as f64 } else { 0.0 };
    let avg_touten_per_sentence = if sentence_count > 0 { total_touten as f64 / sentence_count as f64 } else { 0.0 };

    let kanji = text.chars().filter(|&c| is_kanji(c)).count() as i64;
    let hiragana = text.chars().filter(|&c| is_hiragana(c)).count() as i64;
    let katakana = text.chars().filter(|&c| is_katakana(c)).count() as i64;
    let dialogue = dialogue_char_count(text);

    ReadabilityStats {
        total_chars,
        sentence_count,
        avg_sentence_length,
        max_sentence_length,
        long_sentence_count,
        dialogue_ratio: dialogue as f64 / total_chars as f64,
        kanji_ratio: kanji as f64 / total_chars as f64,
        hiragana_ratio: hiragana as f64 / total_chars as f64,
        katakana_ratio: katakana as f64 / total_chars as f64,
        avg_touten_per_sentence,
    }
}

/// AIへの参考情報として渡す、統計値の日本語サマリー文字列。
pub fn summarize_for_ai(stats: &ReadabilityStats) -> String {
    format!(
        "総文字数: {}字\n文数: {}文\n平均文長: {:.1}字\n最長文: {}字\n100字超の長文数: {}文\n会話文比率: {:.1}%\n漢字比率: {:.1}%\nひらがな比率: {:.1}%\nカタカナ比率: {:.1}%\n一文あたりの読点数(平均): {:.1}個",
        stats.total_chars,
        stats.sentence_count,
        stats.avg_sentence_length,
        stats.max_sentence_length,
        stats.long_sentence_count,
        stats.dialogue_ratio * 100.0,
        stats.kanji_ratio * 100.0,
        stats.hiragana_ratio * 100.0,
        stats.katakana_ratio * 100.0,
        stats.avg_touten_per_sentence,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_yields_zeroed_stats() {
        let stats = compute("");
        assert_eq!(stats.total_chars, 0);
        assert_eq!(stats.sentence_count, 0);
    }

    #[test]
    fn counts_sentences_dialogue_and_char_kinds() {
        let text = "「おはよう」と彼女は言った。空はとても青かった。";
        let stats = compute(text);
        assert_eq!(stats.sentence_count, 2);
        assert!(stats.total_chars > 0);
        assert!(stats.dialogue_ratio > 0.0);
        assert!(stats.kanji_ratio > 0.0);
        assert!(stats.hiragana_ratio > 0.0);
    }

    #[test]
    fn flags_long_sentences() {
        let long = "あ".repeat(150) + "。";
        let stats = compute(&long);
        assert_eq!(stats.sentence_count, 1);
        assert_eq!(stats.long_sentence_count, 1);
        assert_eq!(stats.max_sentence_length, 151);
    }
}
