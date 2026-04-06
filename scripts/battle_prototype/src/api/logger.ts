const sessionId =
  new Date().toISOString().replace(/[:.]/g, "-") +
  "_" +
  Math.random().toString(36).slice(2, 8);

let sequence = 0;

interface LogEntry {
  session_id: string;
  seq: number;
  ts: string;
  elapsed_ms: number;
  type: string;
  [key: string]: unknown;
}

const sessionStart = performance.now();

function send(entry: LogEntry): void {
  const body = JSON.stringify(entry);
  try {
    navigator.sendBeacon("/__log", body);
  } catch {
    // Swallow errors — logging must never break the app.
  }
}

function base(type: string): LogEntry {
  return {
    session_id: sessionId,
    seq: sequence++,
    ts: new Date().toISOString(),
    elapsed_ms: Math.round(performance.now() - sessionStart),
    type,
  };
}

export function logSessionStart(userId: string): void {
  send({ ...base("session_start"), user_id: userId });
}

export function logConnect(
  userId: string,
  deck: string | undefined,
  responseVersion: string,
  durationMs: number,
): void {
  send({
    ...base("connect"),
    user_id: userId,
    deck: deck ?? null,
    response_version: responseVersion,
    duration_ms: durationMs,
  });
}

export function logAction(
  action: unknown,
  responseVersion: string | undefined,
): void {
  send({
    ...base("action_sent"),
    action,
    response_version: responseVersion ?? null,
  });
}

export function logActionResult(
  action: unknown,
  durationMs: number,
  hasView: boolean,
  canAct: boolean | null,
  error: string | null,
): void {
  send({
    ...base("action_result"),
    action,
    duration_ms: durationMs,
    has_view: hasView,
    can_act: canAct,
    error,
  });
}

export function logPollResult(
  responseType: string,
  responseVersion: string | undefined,
  hasView: boolean,
  canAct: boolean | null,
  durationMs: number,
): void {
  send({
    ...base("poll_result"),
    response_type: responseType,
    response_version: responseVersion ?? null,
    has_view: hasView,
    can_act: canAct,
    duration_ms: durationMs,
  });
}

export function logPollingStart(): void {
  send(base("polling_start"));
}

export function logPollingStop(reason: string): void {
  send({ ...base("polling_stop"), reason });
}

export function logJudgmentPause(
  turnNumber: number,
  userScore: number,
  enemyScore: number,
): void {
  send({
    ...base("judgment_pause"),
    turn_number: turnNumber,
    user_score: userScore,
    enemy_score: enemyScore,
  });
}

export function logJudgmentContinue(): void {
  send(base("judgment_continue"));
}

export function logBattleEvent(event: string): void {
  send({ ...base("battle_event"), message: event });
}

export function logStateUpdate(
  source: string,
  turnNumber: number,
  userScore: number,
  enemyScore: number,
  userCanAct: boolean,
  energy: number,
  gameOver: boolean,
): void {
  send({
    ...base("state_update"),
    source,
    turn_number: turnNumber,
    user_score: userScore,
    enemy_score: enemyScore,
    user_can_act: userCanAct,
    energy,
    game_over: gameOver,
  });
}

export function logError(source: string, message: string): void {
  send({ ...base("error"), source, message });
}

export function logReconnect(deck: string | undefined): void {
  send({ ...base("reconnect"), deck: deck ?? null });
}

export function logResponseVersionUpdate(
  source: string,
  oldVersion: string | undefined,
  newVersion: string,
): void {
  send({
    ...base("response_version_update"),
    source,
    old_version: oldVersion ?? null,
    new_version: newVersion,
  });
}
