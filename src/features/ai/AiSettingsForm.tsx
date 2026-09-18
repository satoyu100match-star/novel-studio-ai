import { useEffect, useState } from "react";
import { AI_PROVIDER_LABELS } from "@/types/tauriCommands";
import * as aiService from "@/services/aiService";
import * as settingsService from "@/services/settingsService";

const SETTING_KEYS = {
  provider: "ai.provider",
  model: "ai.model",
  baseUrl: "ai.base_url",
  temperature: "ai.temperature",
  maxOutputTokens: "ai.max_output_tokens",
};

/**
 * AI設定画面(仕様#45)。Provider / Model / API Key / Base URL / Temperature
 * / Max Output。APIキーはこのフォームから送信するだけで、フロントに
 * 戻ってくることは二度とない(Phase17: ai::keystoreがOSの資格情報ストア
 * 経由で保存する方式。docs/AI.md 3章参照)。
 */
export function AiSettingsForm() {
  const [providerKinds, setProviderKinds] = useState<string[]>([]);
  const [provider, setProvider] = useState("anthropic");
  const [model, setModel] = useState("");
  const [baseUrl, setBaseUrl] = useState("");
  const [temperature, setTemperature] = useState("");
  const [maxOutputTokens, setMaxOutputTokens] = useState("");
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [hasKey, setHasKey] = useState(false);
  const [savedMessage, setSavedMessage] = useState("");

  useEffect(() => {
    (async () => {
      const kinds = await aiService.listAiProviderKinds();
      setProviderKinds(kinds);
      const [savedProvider, savedModel, savedBaseUrl, savedTemp, savedMax] = await Promise.all([
        settingsService.readSetting(SETTING_KEYS.provider),
        settingsService.readSetting(SETTING_KEYS.model),
        settingsService.readSetting(SETTING_KEYS.baseUrl),
        settingsService.readSetting(SETTING_KEYS.temperature),
        settingsService.readSetting(SETTING_KEYS.maxOutputTokens),
      ]);
      const resolvedProvider = savedProvider ?? kinds[0] ?? "anthropic";
      setProvider(resolvedProvider);
      setModel(savedModel ?? "");
      setBaseUrl(savedBaseUrl ?? "");
      setTemperature(savedTemp ?? "");
      setMaxOutputTokens(savedMax ?? "");
      setHasKey(await aiService.hasAiApiKey(resolvedProvider));
    })();
  }, []);

  useEffect(() => {
    aiService.hasAiApiKey(provider).then(setHasKey);
    setApiKeyInput("");
  }, [provider]);

  async function handleSave(e: React.FormEvent) {
    e.preventDefault();
    await Promise.all([
      settingsService.writeSetting(SETTING_KEYS.provider, provider),
      settingsService.writeSetting(SETTING_KEYS.model, model),
      settingsService.writeSetting(SETTING_KEYS.baseUrl, baseUrl),
      settingsService.writeSetting(SETTING_KEYS.temperature, temperature),
      settingsService.writeSetting(SETTING_KEYS.maxOutputTokens, maxOutputTokens),
    ]);
    if (apiKeyInput.trim()) {
      await aiService.setAiApiKey(provider, apiKeyInput.trim());
      setApiKeyInput("");
      setHasKey(true);
    }
    setSavedMessage("保存しました。");
    setTimeout(() => setSavedMessage(""), 2000);
  }

  async function handleClearKey() {
    await aiService.clearAiApiKey(provider);
    setHasKey(false);
  }

  return (
    <form className="detail-form" onSubmit={handleSave}>
      <div className="status-line" style={{ maxWidth: 640 }}>
        AI機能は完全に任意です。APIキーを設定しなくても通常の執筆機能は
        すべて利用できます。APIキーはこのアプリのファイルやデータベース
        には一切保存されず、お使いのOSの資格情報ストア(Windowsの資格情報
        マネージャー等)に保存されます。一度設定すればアプリを再起動しても
        再入力は不要です(その環境でOSの資格情報ストアが利用できない場合は、
        今回の起動中のみの一時的な保持になります)。
      </div>
      <label className="form-label">
        AI Provider
        <select value={provider} onChange={(e) => setProvider(e.target.value)}>
          {providerKinds.map((kind) => (
            <option key={kind} value={kind}>
              {AI_PROVIDER_LABELS[kind] ?? kind}
            </option>
          ))}
        </select>
      </label>
      <label className="form-label">
        Model
        <input value={model} onChange={(e) => setModel(e.target.value)} placeholder="例: claude-opus-4-6" />
      </label>
      {provider === "openai_compatible" && (
        <label className="form-label">
          Base URL
          <input value={baseUrl} onChange={(e) => setBaseUrl(e.target.value)} placeholder="https://.../v1/chat/completions" />
        </label>
      )}
      <label className="form-label">
        API Key {hasKey && <span className="status-badge">設定済み</span>}
        <input
          type="password"
          value={apiKeyInput}
          onChange={(e) => setApiKeyInput(e.target.value)}
          placeholder={hasKey ? "変更する場合のみ入力" : "sk-..."}
        />
      </label>
      {hasKey && (
        <button type="button" className="secondary" onClick={handleClearKey}>
          APIキーを削除
        </button>
      )}
      <label className="form-label">
        Temperature(空欄でプロバイダーの既定値)
        <input value={temperature} onChange={(e) => setTemperature(e.target.value)} placeholder="例: 0.7" />
      </label>
      <label className="form-label">
        Max Output Tokens(空欄でプロバイダーの既定値)
        <input value={maxOutputTokens} onChange={(e) => setMaxOutputTokens(e.target.value)} placeholder="例: 2048" />
      </label>
      <div className="field-row">
        <button className="primary" type="submit">
          保存
        </button>
        {savedMessage && <span className="status-line">{savedMessage}</span>}
      </div>
    </form>
  );
}
