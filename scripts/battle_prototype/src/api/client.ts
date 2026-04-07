import type {
  ConnectResponse,
  PerformActionResponse,
  PollResponse,
  GameAction,
  TestDeckName,
  Metadata,
} from "../types/battle";

function generateUserId(): string {
  return "10000000-1000-4000-8000-100000000000".replace(/[018]/g, (c) =>
    (Number(c) ^ (crypto.getRandomValues(new Uint8Array(1))[0] & (15 >> (Number(c) / 4)))).toString(16),
  );
}

let currentUserId: string =
  new URLSearchParams(window.location.search).get("user") ??
  generateUserId();

function metadata(): Metadata {
  return { user_id: currentUserId };
}

export function resetUserId(): void {
  currentUserId = generateUserId();
}

export async function connect(
  deckOverride?: TestDeckName,
  userGoesSecond?: boolean,
): Promise<ConnectResponse> {
  const debugConfig = deckOverride || userGoesSecond
    ? {
        deck_override: deckOverride ?? undefined,
        user_goes_second: userGoesSecond ?? undefined,
      }
    : undefined;
  const body = JSON.stringify({
    metadata: metadata(),
    persistent_data_path: "",
    streaming_assets_path: "",
    debug_configuration: debugConfig,
  });
  const res = await fetch("/connect", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body,
  });
  if (!res.ok) {
    throw new Error(`connect failed: ${res.status} ${await res.text()}`);
  }
  return (await res.json()) as ConnectResponse;
}

export async function performAction(
  action: GameAction,
  lastResponseVersion?: string,
): Promise<PerformActionResponse> {
  const body = JSON.stringify({
    metadata: metadata(),
    action,
    last_response_version: lastResponseVersion,
  });
  const res = await fetch("/perform_action", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body,
  });
  if (!res.ok) {
    throw new Error(
      `perform_action failed: ${res.status} ${await res.text()}`,
    );
  }
  return (await res.json()) as PerformActionResponse;
}

export async function poll(): Promise<PollResponse> {
  const body = JSON.stringify({ metadata: metadata() });
  const res = await fetch("/poll", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body,
  });
  if (!res.ok) {
    throw new Error(`poll failed: ${res.status} ${await res.text()}`);
  }
  return (await res.json()) as PollResponse;
}
