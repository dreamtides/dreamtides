// @vitest-environment jsdom

import { act } from "react";
import type { HTMLAttributes, ReactElement, ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { CardSourceOverlay } from "./CardSourceOverlay";
import type { CardSourceDebugState } from "../types/quest";

vi.mock("framer-motion", () => ({
  AnimatePresence: ({ children }: { children: ReactNode }) => <>{children}</>,
  motion: {
    aside: ({
      animate: _animate,
      children,
      exit: _exit,
      initial: _initial,
      transition: _transition,
      ...props
    }: {
      animate?: unknown;
      children: ReactNode;
      exit?: unknown;
      initial?: unknown;
      transition?: unknown;
    } & HTMLAttributes<HTMLElement>) => <aside {...props}>{children}</aside>,
  },
}));

function makeOverlayState(): CardSourceDebugState {
  return {
    screenLabel: "Shop Offers",
    surface: "Shop",
    entries: [
      {
        cardNumber: 11,
        cardName: "Lantern Broker",
        cardTides: ["core", "support-a"],
        matchedMandatoryTides: ["core"],
        matchedOptionalTides: ["support-a"],
        isFallback: false,
      },
      {
        cardNumber: 12,
        cardName: "Driftbound Relic",
        cardTides: ["outsider"],
        matchedMandatoryTides: [],
        matchedOptionalTides: [],
        isFallback: true,
      },
    ],
  };
}

function mount(element: ReactElement): {
  container: HTMLDivElement;
  root: Root;
} {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  act(() => {
    root.render(element);
  });
  return { container, root };
}

beforeEach(() => {
  vi.clearAllMocks();
  (
    globalThis as typeof globalThis & {
      IS_REACT_ACT_ENVIRONMENT?: boolean;
    }
  ).IS_REACT_ACT_ENVIRONMENT = true;
});

afterEach(() => {
  document.body.innerHTML = "";
});

describe("CardSourceOverlay", () => {
  it("renders matched tides and fallback explanations for visible cards", () => {
    const { container, root } = mount(
      <CardSourceOverlay
        cardSourceDebug={makeOverlayState()}
        isOpen
        onClose={vi.fn()}
      />,
    );

    expect(container.textContent).toContain("Why am I seeing these cards?");
    expect(container.textContent).toContain("Shop Offers");
    expect(container.textContent).toContain("Lantern Broker");
    expect(container.textContent).toContain("core");
    expect(container.textContent).toContain("support-a");
    expect(container.textContent).toContain("Driftbound Relic");
    expect(container.textContent).toContain("broader-pool fallback");

    act(() => {
      root.unmount();
    });
  });
});
