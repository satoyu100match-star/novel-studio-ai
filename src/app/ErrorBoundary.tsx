import { Component, type ErrorInfo, type ReactNode } from "react";
import { logger } from "@/utils/logger";

interface Props {
  children: ReactNode;
}

interface State {
  error: Error | null;
}

/**
 * Top-level crash barrier.
 *
 * Rule (spec #91 / CLAUDE.md): never show a bare "何か失敗しました" with no
 * explanation and no next step. This still isn't a substitute for the
 * real crash-recovery flow (Phase 8, "Working Draft" restore) -- it's the
 * last-resort net so a render crash doesn't blank the whole window before
 * that exists.
 */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    logger.error("unhandled render error", {
      message: error.message,
      componentStack: info.componentStack ?? undefined,
    });
  }

  private handleReload = () => {
    this.setState({ error: null });
    window.location.reload();
  };

  render() {
    const { error } = this.state;
    if (!error) return this.props.children;

    return (
      <div className="error-boundary" role="alert">
        <h1>画面の表示中に問題が発生しました</h1>
        <p>
          原稿データそのものは失われていない可能性があります。アプリを再読み込みしてください。
          問題が繰り返される場合は、直前の操作内容を控えてご報告ください。
        </p>
        <pre>{error.message}</pre>
        <button className="primary" onClick={this.handleReload}>
          再読み込み
        </button>
      </div>
    );
  }
}
