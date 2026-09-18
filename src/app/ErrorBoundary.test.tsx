import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ErrorBoundary } from "@/app/ErrorBoundary";

function Bomb(): never {
  throw new Error("boom");
}

describe("ErrorBoundary", () => {
  it("renders children when there is no error", () => {
    render(
      <ErrorBoundary>
        <div>正常表示</div>
      </ErrorBoundary>,
    );
    expect(screen.getByText("正常表示")).toBeInTheDocument();
  });

  it("renders a non-generic fallback UI when a child throws", () => {
    // React logs the error to the console during the throw; keep the
    // test output clean.
    const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    render(
      <ErrorBoundary>
        <Bomb />
      </ErrorBoundary>,
    );

    expect(screen.getByRole("alert")).toBeInTheDocument();
    expect(screen.getByText("boom")).toBeInTheDocument();
    expect(screen.getByText("再読み込み")).toBeInTheDocument();

    consoleErrorSpy.mockRestore();
  });

  it("clicking reload clears the error state", () => {
    const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const reloadSpy = vi.fn();
    Object.defineProperty(window, "location", {
      value: { reload: reloadSpy },
      writable: true,
    });

    render(
      <ErrorBoundary>
        <Bomb />
      </ErrorBoundary>,
    );

    fireEvent.click(screen.getByText("再読み込み"));
    expect(reloadSpy).toHaveBeenCalled();

    consoleErrorSpy.mockRestore();
  });
});
