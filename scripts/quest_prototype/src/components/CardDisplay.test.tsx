// @vitest-environment jsdom

import { act } from "react";
import type { ReactElement } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import type { CardData } from "../types/cards";
import { CardDisplay } from "./CardDisplay";

function makeCard(overrides: Partial<CardData>): CardData {
  return {
    name: "Test Card",
    id: "test-card",
    cardNumber: 1,
    cardType: "Character",
    subtype: "",
    isStarter: false,
    energyCost: 3,
    spark: 2,
    isFast: false,
    tides: ["Bloom"],
    renderedText: "Test text.",
    imageNumber: 1,
    artOwned: true,
    ...overrides,
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
  (
    globalThis as typeof globalThis & {
      IS_REACT_ACT_ENVIRONMENT?: boolean;
    }
  ).IS_REACT_ACT_ENVIRONMENT = true;
});

afterEach(() => {
  document.body.innerHTML = "";
});

describe("CardDisplay", () => {
  it("renders neutral Character cards with subdued chrome and white names", () => {
    const { container, root } = mount(
      <CardDisplay card={makeCard({ tides: [] })} />,
    );

    const cardRoot = container.firstElementChild;
    const cardName = container.querySelector("h3");
    if (
      !(cardRoot instanceof HTMLDivElement)
      || !(cardName instanceof HTMLHeadingElement)
    ) {
      throw new Error("Missing card content");
    }

    expect(cardRoot.style.border).toContain("rgba(255, 255, 255, 0.18)");
    expect(cardRoot.style.boxShadow).toBe("");
    expect(cardName.style.color).toBe("rgb(248, 250, 252)");

    act(() => {
      root.unmount();
    });
  });

  it("renders Event cards with purple border chrome and tide-colored names", () => {
    const { container, root } = mount(
      <CardDisplay
        card={makeCard({
          cardType: "Event",
          spark: null,
        })}
      />,
    );

    const cardRoot = container.firstElementChild;
    const cardName = container.querySelector("h3");
    if (
      !(cardRoot instanceof HTMLDivElement)
      || !(cardName instanceof HTMLHeadingElement)
    ) {
      throw new Error("Missing card content");
    }

    expect(cardRoot.style.border).toContain("rgb(192, 132, 252)");
    expect(cardRoot.style.boxShadow).toContain("#c084fc");
    expect(cardName.style.color).toBe("rgb(16, 185, 129)");

    act(() => {
      root.unmount();
    });
  });
});
