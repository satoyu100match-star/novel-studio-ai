//! APIキーの保管(仕様#45, docs/AI.md 3章)。
//!
//! 方針: ソースコードへのハードコード禁止、SQLiteへの平文保存禁止
//! (CLAUDE.md禁止事項)。本来はOSの資格情報ストア(Windows Credential
//! Manager / macOS Keychain / Linux Secret Service)を`keyring`クレート
//! 経由で使う設計にしたいが、Phase 5実装時点でこの開発環境から
//! crates.ioへの新規クレート取得がネットワーク障害でできなかった
//! (`CLAUDE.md`の既知の問題を参照)。
//!
//! 生半可な自前暗号化(同じフォルダに鍵と暗号文を並べて置くだけの
//! XOR等)は、ディスクへアクセスできる相手に対しては実質平文と同じ
//! 安全性しかなく、かえって「暗号化されているから安全」という誤った
//! 安心感を与えてしまう。そのためPhase 5では、ディスクへの永続化を
//! 一切行わず、アプリ実行中のみメモリ上(Rust側、Tauriのstate)に
//! 保持する方式にする。この制約により、アプリを再起動するたびに
//! APIキーの再入力が必要になる(既知の問題としてCLAUDE.md/AI.mdに
//! 明記し、ネットワーク制約解消後にkeyringクレート導入で解消する)。
//!
//! フロントエンドへは一切キーを返さない(設定済みかどうかの真偽値の
//! みを返す)。ログにも出力しない。

use std::collections::HashMap;
use std::sync::Mutex;

pub struct AiKeyStore {
    keys: Mutex<HashMap<String, String>>,
}

impl AiKeyStore {
    pub fn new() -> Self {
        Self { keys: Mutex::new(HashMap::new()) }
    }

    pub fn set(&self, provider: &str, api_key: String) {
        let mut keys = self.keys.lock().expect("ai key store mutex poisoned");
        if api_key.is_empty() {
            keys.remove(provider);
        } else {
            keys.insert(provider.to_string(), api_key);
        }
    }

    pub fn get(&self, provider: &str) -> Option<String> {
        let keys = self.keys.lock().expect("ai key store mutex poisoned");
        keys.get(provider).cloned()
    }

    pub fn has(&self, provider: &str) -> bool {
        let keys = self.keys.lock().expect("ai key store mutex poisoned");
        keys.contains_key(provider)
    }

    pub fn clear(&self, provider: &str) {
        let mut keys = self.keys.lock().expect("ai key store mutex poisoned");
        keys.remove(provider);
    }
}

impl Default for AiKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_has_get_and_clear_roundtrip() {
        let store = AiKeyStore::new();
        assert!(!store.has("anthropic"));
        store.set("anthropic", "sk-test-123".into());
        assert!(store.has("anthropic"));
        assert_eq!(store.get("anthropic").as_deref(), Some("sk-test-123"));
        store.clear("anthropic");
        assert!(!store.has("anthropic"));
    }

    #[test]
    fn setting_empty_string_clears_key() {
        let store = AiKeyStore::new();
        store.set("openai", "sk-abc".into());
        store.set("openai", "".into());
        assert!(!store.has("openai"));
    }
}
