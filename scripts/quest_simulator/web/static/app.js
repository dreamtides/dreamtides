"use strict";

// ── State ──────────────────────────────────────────────────────────────────

let currentPrompt = null;  // last prompt received from server
let focusIndex = 0;        // keyboard cursor position
let checkedIndices = new Set();  // multi-select checked items

// ── DOM refs ───────────────────────────────────────────────────────────────

const elContext     = document.getElementById("context");
const elOptions     = document.getElementById("options");
const elActionBar   = document.getElementById("action-bar");
const elStatusEss   = document.getElementById("status-essence");
const elStatusComp  = document.getElementById("status-completion");
const elStatusDeck  = document.getElementById("status-deck");
const elStatusCaller= document.getElementById("status-dreamcaller");
const elDeckSidebar = document.getElementById("deck-sidebar");
const elDeckList    = document.getElementById("deck-list");
const elDeckToggle  = document.getElementById("deck-toggle");
const elGameOver    = document.getElementById("game-over");
const elGameOverMsg = document.getElementById("game-over-message");

// ── Deck sidebar toggle ────────────────────────────────────────────────────

let sidebarOpen = false;
elDeckToggle.addEventListener("click", () => {
  sidebarOpen = !sidebarOpen;
  elDeckSidebar.classList.toggle("hidden", !sidebarOpen);
  elDeckToggle.textContent = sidebarOpen ? "Deck ◀" : "Deck ▶";
});

// ── Status bar ─────────────────────────────────────────────────────────────

function updateStatus(state) {
  if (!state) return;
  elStatusEss.textContent  = `Essence: ${state.essence}`;
  elStatusComp.textContent = `Completion: ${state.completion_level}/${state.total_battles}`;
  elStatusDeck.textContent = `Deck: ${state.deck_count}`;
  elStatusCaller.textContent = state.dreamcaller ? `☆ ${state.dreamcaller}` : "";
}

// ── Deck sidebar ───────────────────────────────────────────────────────────

function renderDeckSidebar(state) {
  if (!state) return;
  elDeckList.innerHTML = "";
  for (const card of state.deck) {
    const li = document.createElement("li");

    // Art zone
    const art = document.createElement("div");
    art.className = "tcg-art";
    if (card.image_hash) {
      const img = document.createElement("img");
      img.src = `/api/images/${card.image_hash}`;
      img.alt = card.name;
      art.appendChild(img);
    }
    if (card.energy_cost !== null) {
      const badge = document.createElement("span");
      badge.className = "tcg-cost";
      badge.textContent = card.energy_cost;
      art.appendChild(badge);
    }
    li.appendChild(art);

    // Name
    const name = document.createElement("div");
    name.className = "tcg-name";
    name.textContent = card.name;
    li.appendChild(name);

    // Type line
    const type = document.createElement("div");
    type.className = "tcg-type";
    type.textContent = card.card_type;
    li.appendChild(type);

    // Rules text
    if (card.rules_text) {
      const rules = document.createElement("div");
      rules.className = "tcg-rules";
      rules.textContent = card.rules_text;
      li.appendChild(rules);
    }

    elDeckList.appendChild(li);
  }
}

// ── Image lookup ───────────────────────────────────────────────────────────

// Pre-computed map of card name → image hash for the current state.
let _cardImageMap = {};

function _buildCardImageMap(state) {
  _cardImageMap = {};
  if (!state || !state.deck) return;
  for (const card of state.deck) {
    if (card.image_hash) _cardImageMap[card.name] = card.image_hash;
  }
}

function cardImageHash(optionText) {
  for (const [name, hash] of Object.entries(_cardImageMap)) {
    if (optionText.includes(name)) return hash;
  }
  return null;
}

// ── Render prompt ──────────────────────────────────────────────────────────

function renderPrompt(data) {
  window._renderGen = (window._renderGen || 0) + 1;
  currentPrompt = data;
  focusIndex = 0;
  checkedIndices = new Set();

  // Update context text (append, don't replace)
  if (data.context) {
    elContext.textContent += data.context;
    elContext.scrollTop = elContext.scrollHeight;
  }

  updateStatus(data.state);
  _buildCardImageMap(data.state);
  renderDeckSidebar(data.state);
  elOptions.innerHTML = "";
  elOptions.classList.remove("card-grid");
  elActionBar.innerHTML = "";

  switch (data.type) {
    case "single_select":
      renderSingleSelect(data);
      break;
    case "multi_select":
      renderMultiSelect(data);
      break;
    case "confirm_decline":
      renderConfirmDecline(data);
      break;
    case "wait_for_continue":
      renderWaitForContinue(data);
      break;
    default:
      break;
  }

  updateFocus();
}

function renderSingleSelect(data) {
  if (data.options.some(opt => cardImageHash(opt))) elOptions.classList.add("card-grid");
  data.options.forEach((opt, i) => {
    const li = createOptionLi(opt, i);
    li.setAttribute("role", "option");
    li.addEventListener("click", () => submitChoice(i));
    elOptions.appendChild(li);
  });
}

function renderMultiSelect(data) {
  window._currentMaxSelections = data.max_selections;
  const hasImages = data.options.some(opt => cardImageHash(opt));
  if (hasImages) elOptions.classList.add("card-grid");
  data.options.forEach((opt, i) => {
    const li = document.createElement("li");
    li.setAttribute("role", "option");

    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.id = `cb-${i}`;
    cb.addEventListener("change", () => {
      if (cb.checked) {
        if (data.max_selections !== null && checkedIndices.size >= data.max_selections) {
          cb.checked = false;
          return;
        }
        checkedIndices.add(i);
      } else {
        checkedIndices.delete(i);
      }
      li.classList.toggle("selected", cb.checked);
    });

    const hash = cardImageHash(opt);
    if (hasImages) {
      // Card-grid mode: portrait card with checkbox in art zone
      const art = document.createElement("div");
      art.className = "tcg-art";
      if (hash) {
        const img = document.createElement("img");
        img.src = `/api/images/${hash}`;
        img.alt = opt;
        art.appendChild(img);
      }
      art.appendChild(cb);
      li.appendChild(art);
      const name = document.createElement("div");
      name.className = "tcg-name";
      name.textContent = opt;
      li.appendChild(name);
      const idx = document.createElement("span");
      idx.className = "option-index";
      idx.textContent = i + 1;
      li.appendChild(idx);
    } else {
      // List mode: checkbox + label + index
      const label = document.createElement("label");
      label.htmlFor = `cb-${i}`;
      label.className = "option-text";
      label.textContent = opt;
      const idx = document.createElement("span");
      idx.className = "option-index";
      idx.textContent = i + 1;
      li.appendChild(cb);
      li.appendChild(label);
      li.appendChild(idx);
    }

    li.addEventListener("click", (e) => {
      if (e.target !== cb) cb.click();
      focusIndex = i;
      updateFocus();
    });
    elOptions.appendChild(li);
  });

  const confirmBtn = document.createElement("button");
  confirmBtn.className = "primary";
  confirmBtn.textContent = "Confirm";
  confirmBtn.addEventListener("click", () => submitChoice(Array.from(checkedIndices).sort((a, b) => a - b)));
  elActionBar.appendChild(confirmBtn);
}

function renderConfirmDecline(data) {
  const [acceptLabel, declineLabel] = data.options.length >= 2
    ? [data.options[0], data.options[1]]
    : ["Accept", "Decline"];

  const acceptBtn = document.createElement("button");
  acceptBtn.className = "primary";
  acceptBtn.textContent = acceptLabel;
  acceptBtn.addEventListener("click", () => submitChoice(true));

  const declineBtn = document.createElement("button");
  declineBtn.className = "danger";
  declineBtn.textContent = declineLabel;
  declineBtn.addEventListener("click", () => submitChoice(false));

  elActionBar.appendChild(acceptBtn);
  elActionBar.appendChild(declineBtn);
}

function renderWaitForContinue(_data) {
  const btn = document.createElement("button");
  btn.className = "primary solo";
  btn.textContent = "Continue";
  btn.addEventListener("click", () => submitChoice(null));
  elActionBar.appendChild(btn);
  btn.focus();
}

function createOptionLi(text, index) {
  const li = document.createElement("li");
  const hash = cardImageHash(text);

  if (hash) {
    // Portrait card layout
    const art = document.createElement("div");
    art.className = "tcg-art";
    const img = document.createElement("img");
    img.src = `/api/images/${hash}`;
    img.alt = text;
    art.appendChild(img);
    li.appendChild(art);
    const name = document.createElement("div");
    name.className = "tcg-name";
    name.textContent = text;
    li.appendChild(name);
  } else {
    const span = document.createElement("span");
    span.className = "option-text";
    span.textContent = text;
    li.appendChild(span);
  }

  const idx = document.createElement("span");
  idx.className = "option-index";
  idx.textContent = index + 1;
  li.appendChild(idx);
  return li;
}

// ── Focus management ───────────────────────────────────────────────────────

function updateFocus() {
  const items = elOptions.querySelectorAll("li");
  items.forEach((li, i) => li.classList.toggle("focused", i === focusIndex));
  if (items[focusIndex]) {
    items[focusIndex].scrollIntoView({ block: "nearest" });
  }
}

// ── Keyboard navigation ────────────────────────────────────────────────────

document.addEventListener("keydown", (e) => {
  if (!currentPrompt) return;

  const items = elOptions.querySelectorAll("li");
  const type = currentPrompt.type;

  if (e.key === "ArrowDown") {
    e.preventDefault();
    if (items.length > 0) {
      focusIndex = (focusIndex + 1) % items.length;
      updateFocus();
    }
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    if (items.length > 0) {
      focusIndex = (focusIndex - 1 + items.length) % items.length;
      updateFocus();
    }
  } else if (e.key === "Enter") {
    e.preventDefault();
    if (type === "single_select" && items.length > 0) {
      submitChoice(focusIndex);
    } else if (type === "multi_select") {
      const confirmBtn = elActionBar.querySelector("button");
      if (confirmBtn) confirmBtn.click();
    } else if (type === "confirm_decline") {
      const buttons = elActionBar.querySelectorAll("button");
      if (focusIndex === 0 && buttons[0]) buttons[0].click();
      else if (buttons[1]) buttons[1].click();
    } else if (type === "wait_for_continue") {
      submitChoice(null);
    }
  } else if (e.key === " " && type === "multi_select") {
    e.preventDefault();
    const cb = items[focusIndex]?.querySelector("input[type='checkbox']");
    if (cb) cb.click();
  } else if (e.key === "ArrowLeft" && type === "confirm_decline") {
    e.preventDefault();
    focusIndex = 0;
    updateFocus();
  } else if (e.key === "ArrowRight" && type === "confirm_decline") {
    e.preventDefault();
    focusIndex = 1;
    updateFocus();
  }
});

// ── Submit choice ──────────────────────────────────────────────────────────

async function submitChoice(choice) {
  currentPrompt = null;
  try {
    await fetch("/api/choice", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ choice }),
    });
  } catch (_) {
    // ignore network errors; pollForPrompt will handle recovery
  }
  pollForPrompt();
}

// ── Polling loop ───────────────────────────────────────────────────────────

async function pollForPrompt() {
  while (true) {
    let data;
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 35_000);
      const resp = await fetch("/api/prompt", { signal: controller.signal });
      clearTimeout(timeoutId);
      data = await resp.json();
    } catch (_) {
      // Network error or timeout — retry after short delay
      await sleep(500);
      continue;
    }

    if (data.type === "waiting") {
      await sleep(100);
      continue;
    }

    if (data.type === "game_over") {
      showGameOver(data);
      return;
    }

    renderPrompt(data);
    return;  // wait for user to submitChoice, which calls pollForPrompt again
  }
}

function showGameOver(data) {
  elGameOverMsg.textContent = `Quest finished after ${data.total_battles || "?"} battles.`;
  elGameOver.classList.remove("hidden");
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// ── Boot ───────────────────────────────────────────────────────────────────

pollForPrompt();
