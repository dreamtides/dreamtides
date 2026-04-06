import {
  createContext,
  useCallback,
  useContext,
  useRef,
  useState,
  type ReactNode,
} from "react";
import type {
  BattleView,
  CardView,
  GameAction,
  TestDeckName,
} from "../types/battle";
import * as api from "../api/client";
import { resetUserId } from "../api/client";
import { extractBattleView } from "../util/command-parser";
import * as log from "../api/logger";

function getPosition(card: CardView): [string, string | null] {
  const pos = card.position.position;
  if (typeof pos === "string") return [pos, null];
  const key = Object.keys(pos)[0];
  const val = (pos as Record<string, unknown>)[key];
  // OnBattlefield is now a tuple [player, rank, slot]
  if (Array.isArray(val)) return [key, typeof val[0] === "string" ? val[0] : null];
  return [key, typeof val === "string" ? val : null];
}

function playerLabel(player: string | null): string {
  if (player === "Enemy") return "Enemy";
  if (player === "User") return "You";
  return "???";
}

function possessiveLabel(player: string | null): string {
  if (player === "Enemy") return "Enemy's";
  if (player === "User") return "Your";
  return "???'s";
}

interface FrontRankCard {
  name: string;
  spark: string;
  player: string | null;
  slot: number;
}

function getFrontRankCards(battle: BattleView): FrontRankCard[] {
  const result: FrontRankCard[] = [];
  for (const card of battle.cards) {
    const pos = card.position.position;
    if (typeof pos !== "string" && "OnBattlefield" in pos) {
      const val = (pos as Record<string, unknown>)["OnBattlefield"];
      if (Array.isArray(val) && val[1] === "Front" && card.revealed) {
        result.push({
          name: card.revealed.name,
          spark: card.revealed.spark ?? "0",
          player: typeof val[0] === "string" ? val[0] : null,
          slot: typeof val[2] === "number" ? val[2] : 0,
        });
      }
    }
  }
  return result;
}

interface GenerateEventsResult {
  events: string[];
  /** True when judgment happened AND something meaningful occurred (score change or combat) */
  judgmentPause: boolean;
}

function generateEvents(oldBattle: BattleView, newBattle: BattleView): GenerateEventsResult {
  const events: string[] = [];
  const oldMap = new Map(oldBattle.cards.map((c) => [c.id, c]));
  const judgmentOccurred = newBattle.turn_number !== oldBattle.turn_number;
  let judgmentHadAction = false;

  // Generate detailed judgment log if turn changed
  if (judgmentOccurred) {
    const oldFront = getFrontRankCards(oldBattle);
    const userCards = oldFront.filter((c) => c.player === "User");
    const enemyCards = oldFront.filter((c) => c.player === "Enemy");
    const userBySlot = new Map(userCards.map((c) => [c.slot, c]));
    const enemyBySlot = new Map(enemyCards.map((c) => [c.slot, c]));
    const allSlots = new Set([...userBySlot.keys(), ...enemyBySlot.keys()]);

    if (allSlots.size > 0) {
      events.push("--- Judgment Phase ---");
    }

    for (const slot of [...allSlots].sort((a, b) => a - b)) {
      const u = userBySlot.get(slot);
      const e = enemyBySlot.get(slot);
      if (u && e) {
        judgmentHadAction = true;
        const uSpark = parseInt(u.spark) || 0;
        const eSpark = parseInt(e.spark) || 0;
        if (uSpark > eSpark) {
          events.push(`Slot ${slot}: Your ${u.name} (${uSpark} spark) defeated Enemy ${e.name} (${eSpark} spark)`);
        } else if (eSpark > uSpark) {
          events.push(`Slot ${slot}: Enemy ${e.name} (${eSpark} spark) defeated Your ${u.name} (${uSpark} spark)`);
        } else {
          events.push(`Slot ${slot}: Your ${u.name} and Enemy ${e.name} clashed at ${uSpark} spark — both dissolved`);
        }
      } else if (u) {
        judgmentHadAction = true;
        events.push(`Slot ${slot}: Your ${u.name} (${u.spark} spark) attacked unblocked — scored ${u.spark} points`);
      } else if (e) {
        judgmentHadAction = true;
        events.push(`Slot ${slot}: Enemy ${e.name} (${e.spark} spark) attacked unblocked — scored ${e.spark} points`);
      }
    }
  }

  const scoreChanged = newBattle.enemy.score !== oldBattle.enemy.score ||
    newBattle.user.score !== oldBattle.user.score;

  // Build arrow target lookup: source card ID → list of target card names
  const newCardMap = new Map(newBattle.cards.map((c) => [c.id, c]));
  const arrowTargets = new Map<string, string[]>();
  for (const arrow of newBattle.arrows) {
    const src = arrow.source as Record<string, unknown>;
    const tgt = arrow.target as Record<string, unknown>;
    const srcId = typeof src === "object" && src !== null && "CardId" in src ? src["CardId"] as string : null;
    const tgtId = typeof tgt === "object" && tgt !== null && "CardId" in tgt ? tgt["CardId"] as string : null;
    if (srcId && tgtId) {
      const tgtCard = newCardMap.get(tgtId);
      const tgtName = tgtCard?.revealed?.name;
      if (tgtName) {
        const existing = arrowTargets.get(srcId);
        if (existing) {
          existing.push(tgtName);
        } else {
          arrowTargets.set(srcId, [tgtName]);
        }
      }
    }
  }

  for (const card of newBattle.cards) {
    const old = oldMap.get(card.id);
    if (!old) continue;
    const [oldPos, oldPlayer] = getPosition(old);
    const [newPos, newPlayer] = getPosition(card);
    if (oldPos === newPos) continue;
    const name = card.revealed?.name ?? old.revealed?.name;
    if (!name) continue;

    // Card materialized (appeared on battlefield from any non-battlefield zone)
    if (newPos === "OnBattlefield" && oldPos !== "OnBattlefield") {
      events.push(`${playerLabel(newPlayer)} materialized ${name}`);
    }
    // Card dissolved (battlefield -> void) — skip during judgment (already logged above)
    else if (oldPos === "OnBattlefield" && newPos === "InVoid") {
      if (!judgmentOccurred) {
        events.push(`${playerLabel(oldPlayer)}: ${name} dissolved`);
      }
    }
    // Card banished (battlefield -> banished)
    else if (oldPos === "OnBattlefield" && newPos === "InBanished") {
      events.push(`${playerLabel(oldPlayer)}: ${name} banished`);
    }
    // Card played from hand (to stack)
    else if (oldPos === "InHand" && newPos === "OnStack") {
      const targets = arrowTargets.get(card.id);
      if (targets && targets.length > 0) {
        events.push(`${playerLabel(oldPlayer)} played ${name} targeting ${targets.join(", ")}`);
      } else {
        events.push(`${playerLabel(oldPlayer)} played ${name}`);
      }
    }
    // Card drawn (deck -> hand)
    else if (oldPos === "InDeck" && newPos === "InHand") {
      events.push(`${playerLabel(newPlayer)} drew a card`);
    }
    // Card returned to hand
    else if (oldPos === "OnBattlefield" && newPos === "InHand") {
      events.push(`${name} returned to ${possessiveLabel(newPlayer)} hand`);
    }
    // Card moved to void from hand or deck
    else if (newPos === "InVoid" && oldPos !== "OnBattlefield" && oldPos !== "InVoid") {
      events.push(`${name} sent to ${possessiveLabel(newPlayer)} void`);
    }
  }

  if (newBattle.enemy.score !== oldBattle.enemy.score) {
    events.push(`Enemy score: ${oldBattle.enemy.score} \u2192 ${newBattle.enemy.score}`);
  }
  if (newBattle.user.score !== oldBattle.user.score) {
    events.push(`Your score: ${oldBattle.user.score} \u2192 ${newBattle.user.score}`);
  }

  return { events, judgmentPause: judgmentOccurred && (judgmentHadAction || scoreChanged) };
}

interface BattleContextValue {
  battle: BattleView | null;
  isPolling: boolean;
  error: string | null;
  events: string[];
  yourTurnCounter: number;
  judgmentPause: boolean;
  continueFromJudgment: () => void;
  sendAction: (action: GameAction) => void;
  sendDebugAction: (action: GameAction) => void;
  reconnect: (deck?: TestDeckName) => void;
}

const BattleContext = createContext<BattleContextValue | null>(null);

export function useBattle(): BattleContextValue {
  const ctx = useContext(BattleContext);
  if (!ctx) throw new Error("useBattle must be used within BattleProvider");
  return ctx;
}

const POLL_INTERVAL_MS = 200;

export function BattleProvider({ children }: { children: ReactNode }) {
  const [battle, setBattle] = useState<BattleView | null>(null);
  const [isPolling, setIsPolling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [events, setEvents] = useState<string[]>([]);
  const [yourTurnCounter, setYourTurnCounter] = useState(0);
  const [judgmentPause, setJudgmentPause] = useState(false);
  const responseVersionRef = useRef<string | undefined>(undefined);
  const pollIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const prevBattleRef = useRef<BattleView | null>(null);
  const wasPollingRef = useRef(false);
  // Generation counter to invalidate stale in-flight polls
  const pollGenerationRef = useRef(0);

  const stopPolling = useCallback((reason?: string) => {
    pollGenerationRef.current++;
    if (pollIntervalRef.current != null) {
      clearInterval(pollIntervalRef.current);
      pollIntervalRef.current = null;
    }
    setIsPolling(false);
    log.logPollingStop(reason ?? "unknown");
  }, []);

  const startPolling = useCallback(() => {
    // Clear any existing polling interval before starting a new one
    if (pollIntervalRef.current != null) {
      clearInterval(pollIntervalRef.current);
      pollIntervalRef.current = null;
    }
    pollGenerationRef.current++;
    const myGeneration = pollGenerationRef.current;
    setIsPolling(true);
    wasPollingRef.current = true;
    log.logPollingStart();
    let pollInFlight = false;
    const interval = setInterval(() => {
      if (pollInFlight) return;
      if (pollGenerationRef.current !== myGeneration) {
        clearInterval(interval);
        return;
      }
      pollInFlight = true;
      void (async () => {
        const pollStart = performance.now();
        try {
          const pollRes = await api.poll();
          const pollMs = performance.now() - pollStart;
          // Check if this poll generation is still current
          if (pollGenerationRef.current !== myGeneration) return;
          const view = pollRes.commands ? extractBattleView(pollRes.commands) : null;
          log.logPollResult(
            pollRes.response_type,
            pollRes.response_version,
            !!view,
            view?.user.can_act ?? null,
            pollMs,
          );
          if (view) {
            if (prevBattleRef.current) {
              const result = generateEvents(prevBattleRef.current, view);
              if (result.events.length > 0) {
                setEvents((prev) => [...prev, ...result.events]);
                for (const evt of result.events) log.logBattleEvent(evt);
              }
              if (result.judgmentPause && !view.game_over) {
                prevBattleRef.current = view;
                setBattle(view);
                setJudgmentPause(true);
                log.logJudgmentPause(view.turn_number, view.user.score, view.enemy.score);
                if (pollRes.response_version) {
                  log.logResponseVersionUpdate("poll_judgment", responseVersionRef.current, pollRes.response_version);
                  responseVersionRef.current = pollRes.response_version;
                }
                stopPolling("judgment_pause");
                return;
              }
            }
            prevBattleRef.current = view;
            setBattle(view);
            log.logStateUpdate("poll", view.turn_number, view.user.score, view.enemy.score, view.user.can_act, view.user.energy, view.game_over);
            if (view.user.can_act || view.game_over) {
              if (wasPollingRef.current && !view.game_over) {
                setYourTurnCounter((c) => c + 1);
                wasPollingRef.current = false;
              }
              stopPolling(view.game_over ? "game_over" : "can_act");
            }
          }
          if (pollRes.response_version) {
            log.logResponseVersionUpdate("poll", responseVersionRef.current, pollRes.response_version);
            responseVersionRef.current = pollRes.response_version;
          }
          if (pollRes.response_type === "Final") {
            stopPolling("final");
          }
        } catch (e) {
          const msg = e instanceof Error ? e.message : "Poll failed";
          log.logError("poll", msg);
          stopPolling("error");
          setError(msg);
        } finally {
          pollInFlight = false;
        }
      })();
    }, POLL_INTERVAL_MS);
    pollIntervalRef.current = interval;
  }, [stopPolling]);

  const sendAction = useCallback(
    (action: GameAction) => {
      if (isPolling) return;
      log.logAction(action, responseVersionRef.current);
      void (async () => {
        const actionStart = performance.now();
        try {
          setError(null);
          const res = await api.performAction(
            action,
            responseVersionRef.current,
          );
          const durationMs = performance.now() - actionStart;
          const view = extractBattleView(res.commands);
          log.logActionResult(action, durationMs, !!view, view?.user.can_act ?? null, null);
          if (view) {
            if (prevBattleRef.current) {
              const result = generateEvents(prevBattleRef.current, view);
              if (result.events.length > 0) {
                setEvents((prev) => [...prev, ...result.events]);
                for (const evt of result.events) log.logBattleEvent(evt);
              }
              if (result.judgmentPause && !view.game_over) {
                prevBattleRef.current = view;
                setBattle(view);
                setJudgmentPause(true);
                log.logJudgmentPause(view.turn_number, view.user.score, view.enemy.score);
                return;
              }
            }
            prevBattleRef.current = view;
            setBattle(view);
            log.logStateUpdate("action", view.turn_number, view.user.score, view.enemy.score, view.user.can_act, view.user.energy, view.game_over);
            if (view.user.can_act || view.game_over) return;
          }
          startPolling();
        } catch (e) {
          const msg = e instanceof Error ? e.message : "Action failed";
          log.logError("send_action", msg);
          setError(msg);
        }
      })();
    },
    [isPolling, startPolling],
  );

  const sendDebugAction = useCallback(
    (action: GameAction) => {
      // Stop any background polling and invalidate in-flight polls.
      stopPolling("debug_action");
      log.logAction(action, undefined);
      void (async () => {
        const actionStart = performance.now();
        try {
          setError(null);
          const res = await api.performAction(action, undefined);
          log.logActionResult(action, performance.now() - actionStart, false, null, null);
          let view = extractBattleView(res.commands);
          if (view) {
            setBattle(view);
          }
          // Poll with retries to get the updated state.
          for (let attempt = 0; attempt < 8; attempt++) {
            await new Promise((r) => setTimeout(r, 150 + attempt * 100));
            const pollStart = performance.now();
            const pollRes = await api.poll();
            const pollView = pollRes.commands ? extractBattleView(pollRes.commands) : null;
            log.logPollResult(pollRes.response_type, pollRes.response_version, !!pollView, pollView?.user.can_act ?? null, performance.now() - pollStart);
            if (pollView) {
              setBattle(pollView);
              view = pollView;
            }
            if (pollRes.response_version) {
              log.logResponseVersionUpdate("debug_poll", responseVersionRef.current, pollRes.response_version);
              responseVersionRef.current = pollRes.response_version;
            }
            if (pollRes.response_type === "Final") {
              break;
            }
            // If the response was consumed by a stale in-flight poll,
            // re-send the action to generate a new response.
            if (pollRes.response_type === "None" && attempt === 0) {
              await api.performAction(action, undefined);
              continue;
            }
            if (pollRes.response_type === "None") {
              break;
            }
          }
          if (view) {
            log.logStateUpdate("debug_action", view.turn_number, view.user.score, view.enemy.score, view.user.can_act, view.user.energy, view.game_over);
          }
          if (view && !view.user.can_act) {
            startPolling();
          }
        } catch (e) {
          const msg = e instanceof Error ? e.message : "Debug action failed";
          log.logError("debug_action", msg);
          setError(msg);
        }
      })();
    },
    [stopPolling, startPolling],
  );

  const continueFromJudgment = useCallback(() => {
    log.logJudgmentContinue();
    setJudgmentPause(false);
    if (battle && !battle.user.can_act && !battle.game_over) {
      startPolling();
    } else if (battle && battle.user.can_act) {
      setYourTurnCounter((c) => c + 1);
      wasPollingRef.current = false;
    }
  }, [battle, startPolling]);

  const reconnect = useCallback(
    (deck?: TestDeckName) => {
      log.logReconnect(deck);
      void (async () => {
        const connectStart = performance.now();
        try {
          setError(null);
          setEvents([]);
          stopPolling("reconnect");
          setBattle(null);
          resetUserId();
          const res = await api.connect(deck);
          const durationMs = performance.now() - connectStart;
          log.logResponseVersionUpdate("connect", responseVersionRef.current, res.response_version);
          responseVersionRef.current = res.response_version;
          const view = extractBattleView(res.commands);
          if (view) {
            prevBattleRef.current = view;
            setBattle(view);
            log.logConnect(res.metadata.user_id, deck, res.response_version, durationMs);
            log.logSessionStart(res.metadata.user_id);
            log.logStateUpdate("connect", view.turn_number, view.user.score, view.enemy.score, view.user.can_act, view.user.energy, view.game_over);
          }
        } catch (e) {
          const msg = e instanceof Error ? e.message : "Connect failed";
          log.logError("connect", msg);
          setError(msg);
        }
      })();
    },
    [stopPolling],
  );

  return (
    <BattleContext.Provider
      value={{ battle, isPolling, error, events, yourTurnCounter, judgmentPause, continueFromJudgment, sendAction, sendDebugAction, reconnect }}
    >
      {children}
    </BattleContext.Provider>
  );
}
