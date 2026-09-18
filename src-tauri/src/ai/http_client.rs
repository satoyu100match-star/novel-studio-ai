//! 同期HTTPクライアント(AI Provider呼び出し専用)。
//!
//! 本来は `ureq` や `reqwest` のような専用クレートを追加したいところだが、
//! Phase 5実装時点でこの開発環境から crates.io へ新規クレートを取得
//! できなかった(index.crates.io への新規パッケージ問い合わせが継続的に
//! タイムアウトする状態が続いた。詳細は `CLAUDE.md` の「既知の問題」を
//! 参照)。そのため、OSに標準搭載されている `curl` をサブプロセスとして
//! 起動しHTTP(S)通信を行うフォールバック実装にしている。
//! Windows 10 (1803以降) / macOS / Linuxのいずれも `curl` を標準搭載して
//! いるため、対象プラットフォームでは追加インストールなしに動作する想定。
//! ネットワーク制約が解消され次第、専用クレートへの置き換えを検討する。
//!
//! セキュリティ上の配慮: APIキーを含むヘッダーは一切コマンドライン引数に
//! 載せない。`curl -K -` (stdin経由の設定ファイル)でのみ渡すことで、同じ
//! マシン上の他プロセスが `ps`/タスクマネージャー等でコマンドライン引数を
//! 一覧してもAPIキーが見えないようにする。リクエストボディも一時ファイル
//! 経由で渡し、シェル解釈やcurl configの複雑なエスケープを避ける。

use crate::error::{AppError, AppResult};
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

pub struct HttpRequest<'a> {
    pub method: &'a str,
    pub url: &'a str,
    /// 機微情報を含みうるヘッダー(Authorization, x-api-key等)。stdin経由の
    /// curl configにのみ書き込まれ、argvには一切現れない。
    pub headers: &'a [(&'a str, String)],
    pub body_json: Option<String>,
    pub timeout: Duration,
}

pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

fn escape_config_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn send(req: HttpRequest) -> AppResult<HttpResponse> {
    // Content-Typeなど機微でないヘッダーもまとめてstdin config側に寄せて
    // 実装を単純化する(argvにはURL/メソッド/タイムアウトのみを渡す)。
    let mut body_file = None;
    let mut config = String::new();
    for (name, value) in req.headers {
        config.push_str(&format!("header = \"{}: {}\"\n", name, escape_config_value(value)));
    }
    if let Some(body) = &req.body_json {
        let mut tmp = tempfile::NamedTempFile::new()
            .map_err(|e| AppError::Other(format!("一時ファイルを作成できませんでした: {e}")))?;
        tmp.write_all(body.as_bytes())
            .map_err(|e| AppError::Other(format!("リクエスト本文の書き込みに失敗しました: {e}")))?;
        tmp.flush().ok();
        let path = tmp.path().to_string_lossy().to_string();
        config.push_str(&format!("data-binary = \"@{}\"\n", escape_config_value(&path)));
        body_file = Some(tmp); // keep alive until the curl process reads it
    }

    let timeout_secs = req.timeout.as_secs().max(1).to_string();
    let mut child = Command::new("curl")
        .arg("-sS")
        .arg("-X")
        .arg(req.method)
        .arg(req.url)
        .arg("--max-time")
        .arg(&timeout_secs)
        .arg("-w")
        .arg("\n__NOVEL_STUDIO_HTTP_STATUS__%{http_code}")
        .arg("-K")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            AppError::Other(format!(
                "curlコマンドを起動できませんでした。AI機能の利用にはcurlが必要です: {e}"
            ))
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(config.as_bytes())
            .map_err(|e| AppError::Other(format!("curlへの入力書き込みに失敗しました: {e}")))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| AppError::Other(format!("curlの実行に失敗しました: {e}")))?;
    drop(body_file);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Other(format!("AIプロバイダーへの通信に失敗しました: {stderr}")));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let (body, status) = match stdout.rsplit_once("__NOVEL_STUDIO_HTTP_STATUS__") {
        Some((body, status)) => (body.trim_end().to_string(), status.trim().parse::<u16>().unwrap_or(0)),
        None => (stdout, 0),
    };
    Ok(HttpResponse { status, body })
}
