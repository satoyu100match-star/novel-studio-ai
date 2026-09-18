/**
 * AIが提案した本文と元の本文の差分を表示するための、依存ライブラリなしの
 * 簡易diff実装(仕様#43「差分表示」)。日本語の文章には単語区切りの空白が
 * ないため、単語単位ではなく文字単位(短文向け)または文単位(長文向け、
 * 性能のため)でLCSベースの差分を取る。
 */

export type DiffOp = "equal" | "insert" | "delete";
export type DiffToken = { op: DiffOp; text: string };

const CHAR_DIFF_LIMIT = 4000; // これを超える長さの入力は文単位にフォールバックする

function lcsDiff(a: string[], b: string[]): DiffToken[] {
  const n = a.length;
  const m = b.length;
  // dp[i][j] = a[i..]とb[j..]のLCS長。Uint32Arrayのフラット配列で確保する。
  const dp = new Uint32Array((n + 1) * (m + 1));
  const idx = (i: number, j: number) => i * (m + 1) + j;
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      dp[idx(i, j)] =
        a[i] === b[j] ? dp[idx(i + 1, j + 1)] + 1 : Math.max(dp[idx(i + 1, j)], dp[idx(i, j + 1)]);
    }
  }

  const tokens: DiffToken[] = [];
  let i = 0;
  let j = 0;
  function push(op: DiffOp, text: string) {
    const last = tokens[tokens.length - 1];
    if (last && last.op === op) {
      last.text += text;
    } else {
      tokens.push({ op, text });
    }
  }
  while (i < n && j < m) {
    if (a[i] === b[j]) {
      push("equal", a[i]);
      i++;
      j++;
    } else if (dp[idx(i + 1, j)] >= dp[idx(i, j + 1)]) {
      push("delete", a[i]);
      i++;
    } else {
      push("insert", b[j]);
      j++;
    }
  }
  while (i < n) {
    push("delete", a[i]);
    i++;
  }
  while (j < m) {
    push("insert", b[j]);
    j++;
  }
  return tokens;
}

/** 文単位に分割する(。！？と改行の直後で区切り、区切り文字は前の文に含める)。 */
function splitIntoSentences(text: string): string[] {
  const parts = text.split(/(?<=[。！？\n])/);
  return parts.filter((p) => p.length > 0);
}

export function computeDiff(before: string, after: string): DiffToken[] {
  if (before === after) return [{ op: "equal", text: before }];
  if (before.length <= CHAR_DIFF_LIMIT && after.length <= CHAR_DIFF_LIMIT) {
    return lcsDiff(Array.from(before), Array.from(after));
  }
  return lcsDiff(splitIntoSentences(before), splitIntoSentences(after));
}
