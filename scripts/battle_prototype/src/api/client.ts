import type {
  ConnectResponse,
  PerformActionResponse,
  PollResponse,
  GameAction,
  TestDeckName,
  Metadata,
} from "../types/battle";

const USER_ID = "00000000-0000-0000-0000-000000000001";

function metadata(): Metadata {
  return { user_id: USER_ID };
}

export async function connect(
  deckOverride?: TestDeckName,
): Promise<ConnectResponse> {
  const body = JSON.stringify({
    metadata: metadata(),
    persistent_data_path: "",
    streaming_assets_path: "",
    debug_configuration: deckOverride
      ? { deck_override: deckOverride }
      : undefined,
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
