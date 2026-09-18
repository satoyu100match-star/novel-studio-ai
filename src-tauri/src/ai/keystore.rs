//! APIキーの保管(仕様#45, docs/AI.md 3章)。
//!
//! 方針: ソースコードへのハードコード禁止、SQLiteへの平文保存禁止
//! (CLAUDE.md禁止事項)。
//!
//! Phase 5実装時点では、この開発環境からcrates.ioへの新規クレート取得が
//! ネットワーク障害でできず、`keyring`クレート(OSの資格情報ストア:
//! Windows Credential Manager / macOS Keychain / Linux Secret Service)を
//! 導入できなかった。生半可な自前暗号化(同じフォルダに鍵と暗号文を
//! 並べて置くだけのXOR等)は、ディスクへアクセスできる相手に対しては
//! 実質平文と同じ安全性しかなく、かえって「暗号化されているから安全」
//! という誤った安心感を与えてしまうため、Phase 5ではディスクへの永続化を
//! 一切行わず、アプリ実行中のみメモリ上に保持する方式にしていた
//! (アプリを再起動するたびにAPIキーの再入力が必要、という制約として
//! CLAUDE.md/AI.mdに明記していた)。
//!
//! Phase 17: ネットワーク制約が解消されたため、`keyring`クレート経由で
//! OS標準の資格情報ストアに保存するよう切り替えた(自前の暗号化コードは
//! 一切書かず、各OSが提供する既存の安全な仕組みにそのまま委ねる)。
//! これにより、一度入力したAPIキーはアプリを再起動しても再入力不要になる。
//!
//! ただし、資格情報ストア自体が使えない環境(例: D-Bus Secret Serviceが
//! 起動していないLinux環境)もあり得るため、保存・読み込みに失敗しても
//! アプリの起動やAI機能そのものは止めない: 失敗はログに警告として記録
//! した上で、その場のセッション中だけはメモリ上のキャッシュで動作を
//! 継続する(CLAUDE.md「AI機能が利用不能でも通常の小説執筆機能は動作
//! すること」の精神を踏襲し、エラーの握りつぶしはしない)。
//!
//! フロントエンドへは一切キーを返さない(設定済みかどうかの真偽値の
//! みを返す)。ログにもキーの値自体は出力しない。

use std::collections::HashMap;
use std::sync::Mutex;

/// OS資格情報ストア上でのサービス名。プロバイダーごとに
/// `Entry::new(SERVICE, provider)`でエントリを分ける。
const SERVICE: &str = "com.novelstudioai.app.ai-api-key";

fn entry_for(provider: &str) -> keyring::Result<keyring::Entry> {
    keyring::Entry::new(SERVICE, provider)
}

pub struct AiKeyStore {
    /// OSストアへのアクセスに失敗した場合のフォールバック、および
    /// 読み込み結果のセッション内キャッシュ(毎回OS APIを叩かないため)。
    cache: Mutex<HashMap<String, String>>,
}

impl AiKeyStore {
    pub fn new() -> Self {
        Self { cache: Mutex::new(HashMap::new()) }
    }

    pub fn set(&self, provider: &str, api_key: String) {
        if api_key.is_empty() {
            self.clear(provider);
            return;
        }
        {
            let mut cache = self.cache.lock().expect("ai key store mutex poisoned");
            cache.insert(provider.to_string(), api_key.clone());
        }
        match entry_for(provider) {
            Ok(entry) => {
                if let Err(e) = entry.set_password(&api_key) {
                    log::warn!(
                        "APIキーをOSの資格情報ストアへ保存できませんでした({provider}): {e}。\
                         今回のアプリ実行中はメモリ上でのみ利用できますが、次回起動時は再入力が必要です。"
                    );
                }
            }
            Err(e) => {
                log::warn!(
                    "この環境ではOSの資格情報ストアが利用できません({provider}): {e}。\
                     今回のアプリ実行中はメモリ上でのみ利用できますが、次回起動時は再入力が必要です。"
                );
            }
        }
    }

    pub fn get(&self, provider: &str) -> Option<String> {
        {
            let cache = self.cache.lock().expect("ai key store mutex poisoned");
            if let Some(key) = cache.get(provider) {
                return Some(key.clone());
            }
        }
        // メモリに無ければOSストアからの読み込みを試みる。前回起動時に
        // 保存できていれば、再起動後もここで見つかる(再入力不要)。
        let entry = match entry_for(provider) {
            Ok(entry) => entry,
            Err(_) => return None,
        };
        match entry.get_password() {
            Ok(key) => {
                let mut cache = self.cache.lock().expect("ai key store mutex poisoned");
                cache.insert(provider.to_string(), key.clone());
                Some(key)
            }
            Err(keyring::Error::NoEntry) => None,
            Err(e) => {
                log::warn!("OSの資格情報ストアからのAPIキー読み込みに失敗しました({provider}): {e}");
                None
            }
        }
    }

    pub fn has(&self, provider: &str) -> bool {
        self.get(provider).is_some()
    }

    pub fn clear(&self, provider: &str) {
        {
            let mut cache = self.cache.lock().expect("ai key store mutex poisoned");
            cache.remove(provider);
        }
        if let Ok(entry) = entry_for(provider) {
            match entry.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => {}
                Err(e) => log::warn!("OSの資格情報ストアからのAPIキー削除に失敗しました({provider}): {e}"),
            }
        }
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

    // 実際のOS資格情報ストア(Windows Credential Manager等)にはCI/CD
    // 環境からアクセスできないことが多いため、ユニットテストでは
    // メモリキャッシュの挙動(OSストアが使えない場合のフォールバック
    // 経路と同じコードパス)のみを検証する。実際のOSストアとの往復動作
    // はWindows実機での確認が必要(CLAUDE.md既知の問題参照)。

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

    #[test]
    fn different_providers_do_not_share_a_key() {
        let store = AiKeyStore::new();
        store.set("anthropic", "sk-anthropic".into());
        store.set("openai", "sk-openai".into());
        assert_eq!(store.get("anthropic").as_deref(), Some("sk-anthropic"));
        assert_eq!(store.get("openai").as_deref(), Some("sk-openai"));
        store.clear("anthropic");
        assert!(!store.has("anthropic"));
        assert!(store.has("openai"));
    }
}
