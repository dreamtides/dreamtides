import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import type {
  BattleView,
  CardView,
  DisplayPlayer,
  GameAction,
  TestDeckName,
} from "../types/battle";
import * as api from "../api/client";
import { resetUserId } from "../api/client";
import {
  parseCommandSequence,
  type ParsedBattleCommands,
} from "../util/command-parser";
import * as log from "../api/logger";

function stripTags(text: string): string {
  return text.replace(/<[^>]*>/g, "").trim();
}

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

function hasStackCards(battle: BattleView): boolean {
  return battle.cards.some((c) => {
    const pos = c.position.position;
    return typeof pos !== "string" && "OnStack" in pos;
  });
}

interface GenerateEventsResult {
  events: string[];
  /** True when judgment happened AND something meaningful occurred (score change or combat) */
  judgmentPause: boolean;
}

interface DreamwellReveal {
  card: CardView;
  player: DisplayPlayer;
}

function generateEvents(oldBattle: BattleView, newBattle: BattleView): GenerateEventsResult {
  const events: string[] = [];
  const oldMap = new Map(oldBattle.cards.map((c) => [c.id, c]));
  const judgmentOccurred = newBattle.turn_number !== oldBattle.turn_number;
  let judgmentHadAction = false;

  // Generate detailed judgment log if turn changed.
  // In the combat model, the *non-active* player's front-rank characters are
  // the attackers during judgment. The active player is the one whose turn just
  // ended — they had can_act=true in oldBattle.
  if (judgmentOccurred) {
    const activePlayer = oldBattle.user.can_act ? "User" : "Enemy";
    const attackingPlayer = activePlayer === "User" ? "Enemy" : "User";
    const oldFront = getFrontRankCards(oldBattle);
    const activeCards = oldFront.filter((c) => c.player === activePlayer);
    const attackerCards = oldFront.filter((c) => c.player === attackingPlayer);
    const activeBySlot = new Map(activeCards.map((c) => [c.slot, c]));
    const attackerBySlot = new Map(attackerCards.map((c) => [c.slot, c]));
    const allSlots = new Set([...activeBySlot.keys(), ...attackerBySlot.keys()]);

    if (allSlots.size > 0) {
      events.push("--- Judgment Phase ---");
    }

    for (const slot of [...allSlots].sort((a, b) => a - b)) {
      const attacker = attackerBySlot.get(slot);
      const blocker = activeBySlot.get(slot);
      if (attacker && blocker) {
        // Both players have a front-rank character: spark comparison
        judgmentHadAction = true;
        const aSpark = parseInt(attacker.spark) || 0;
        const bSpark = parseInt(blocker.spark) || 0;
        if (aSpark > bSpark) {
          events.push(`Slot ${slot}: ${playerLabel(attacker.player)} ${attacker.name} (${aSpark}) defeated ${playerLabel(blocker.player)} ${blocker.name} (${bSpark})`);
        } else if (bSpark > aSpark) {
          events.push(`Slot ${slot}: ${playerLabel(blocker.player)} ${blocker.name} (${bSpark}) defeated ${playerLabel(attacker.player)} ${attacker.name} (${aSpark})`);
        } else {
          events.push(`Slot ${slot}: ${playerLabel(attacker.player)} ${attacker.name} and ${playerLabel(blocker.player)} ${blocker.name} clashed at ${aSpark} spark — both dissolved`);
        }
      } else if (attacker) {
        // Attacker with no blocker: scores spark as points
        judgmentHadAction = true;
        events.push(`Slot ${slot}: ${playerLabel(attacker.player)} ${attacker.name} (${attacker.spark} spark) attacked unblocked — scored ${attacker.spark} points`);
      }
      // Active player's unblocked characters do nothing during judgment
    }
  }

  const scoreChanged = newBattle.enemy.score !== oldBattle.enemy.score ||
    newBattle.user.score !== oldBattle.user.score;

  // Build arrow target lookup: source card ID → list of target card names.
  // Check arrows from both old and new battle states — during polling, we may
  // miss the intermediate stack state where arrows are present.
  const allCardMap = new Map([
    ...oldBattle.cards.map((c) => [c.id, c] as const),
    ...newBattle.cards.map((c) => [c.id, c] as const),
  ]);
  const arrowTargets = new Map<string, string[]>();
  const allArrows = [...oldBattle.arrows, ...newBattle.arrows];
  for (const arrow of allArrows) {
    const src = arrow.source as Record<string, unknown>;
    const tgt = arrow.target as Record<string, unknown>;
    const srcId = typeof src === "object" && src !== null && "CardId" in src ? src["CardId"] as string : null;
    const tgtId = typeof tgt === "object" && tgt !== null && "CardId" in tgt ? tgt["CardId"] as string : null;
    if (srcId && tgtId) {
      const tgtCard = allCardMap.get(tgtId);
      const tgtName = tgtCard?.revealed?.name;
      if (tgtName) {
        const existing = arrowTargets.get(srcId);
        if (existing) {
          if (!existing.includes(tgtName)) existing.push(tgtName);
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
    // Card played from hand (to stack, or resolved past the stack during a poll gap)
    else if (oldPos === "InHand" && (newPos === "OnStack" || newPos === "InVoid")) {
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
    // Card moved to void from non-hand, non-battlefield source
    else if (newPos === "InVoid" && oldPos !== "OnBattlefield" && oldPos !== "InVoid" && oldPos !== "InHand") {
      events.push(`${name} sent to ${possessiveLabel(newPlayer)} void`);
    }
    // Dreamwell activated (card moved to DreamwellActivation position).
    // The dreamwell is shared, so all cards render as "User". Determine the
    // actual owner from whose turn it is (dreamwell fires at turn start).
    else if (newPos === "DreamwellActivation" && oldPos !== "DreamwellActivation") {
      const rulesText = card.revealed?.rules_text ?? old.revealed?.rules_text;
      const desc = rulesText ? ` — ${stripTags(rulesText)}` : "";
      const owner = newBattle.user.can_act ? "User" : "Enemy";
      events.push(`${possessiveLabel(owner)} dreamwell: ${name}${desc}`);
    }
    // Card placed in dreamwell
    else if (newPos === "InDreamwell" && oldPos !== "InDreamwell") {
      events.push(`${name} placed in ${possessiveLabel(newPlayer)} dreamwell`);
    }
    // Card left dreamwell (to somewhere other than activation)
    else if (oldPos === "InDreamwell" && newPos !== "InDreamwell" && newPos !== "DreamwellActivation") {
      events.push(`${name} left ${possessiveLabel(oldPlayer)} dreamwell`);
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

function resolveDreamwellReveals(parsed: ParsedBattleCommands): DreamwellReveal[] {
  if (!parsed.battle) return [];
  const cards = new Map(parsed.battle.cards.map((card) => [card.id, card]));
  return parsed.dreamwellActivations.flatMap((activation) => {
    const card = cards.get(activation.card_id);
    return card ? [{ card, player: activation.player }] : [];
  });
}

interface BattleContextValue {
  battle: BattleView | null;
  isPolling: boolean;
  error: string | null;
  events: string[];
  yourTurnCounter: number;
  judgmentPause: boolean;
  dreamwellReveal: DreamwellReveal | null;
  continueFromJudgment: () => void;
  sendAction: (action: GameAction) => void;
  sendDebugAction: (action: GameAction) => void;
  reconnect: (deck?: TestDeckName, userGoesSecond?: boolean) => void;
}

const BattleContext = createContext<BattleContextValue | null>(null);

export function useBattle(): BattleContextValue {
  const ctx = useContext(BattleContext);
  if (!ctx) throw new Error("useBattle must be used within BattleProvider");
  return ctx;
}

const POLL_INTERVAL_MS = 200;
const STACK_PAUSE_MS = 500;
const DREAMWELL_REVEAL_MS = 3000;

export function BattleProvider({ children }: { children: ReactNode }) {
  const [battle, setBattle] = useState<BattleView | null>(null);
  const [isPolling, setIsPolling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [events, setEvents] = useState<string[]>([]);
  const [yourTurnCounter, setYourTurnCounter] = useState(0);
  const [judgmentPause, setJudgmentPause] = useState(false);
  const [dreamwellReveal, setDreamwellReveal] = useState<DreamwellReveal | null>(null);
  const [dreamwellRevealQueue, setDreamwellRevealQueue] = useState<DreamwellReveal[]>([]);
  const [pendingResumePolling, setPendingResumePolling] = useState(false);
  const responseVersionRef = useRef<string | undefined>(undefined);
  const pollIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const prevBattleRef = useRef<BattleView | null>(null);
  const wasPollingRef = useRef(false);
  const judgmentPauseRef = useRef(false);
  const dreamwellRevealRef = useRef<DreamwellReveal | null>(null);
  // Generation counter to invalidate stale in-flight polls
  const pollGenerationRef = useRef(0);
  // Timestamp until which polling should pause (for stack visibility)
  const stackPauseUntilRef = useRef(0);

  const stopPolling = useCallback((reason?: string) => {
    pollGenerationRef.current++;
    if (pollIntervalRef.current != null) {
      clearInterval(pollIntervalRef.current);
      pollIntervalRef.current = null;
    }
    setIsPolling(false);
    log.logPollingStop(reason ?? "unknown");
  }, []);

  useEffect(() => {
    judgmentPauseRef.current = judgmentPause;
  }, [judgmentPause]);

  useEffect(() => {
    dreamwellRevealRef.current = dreamwellReveal;
  }, [dreamwellReveal]);

  const enqueueDreamwellReveals = useCallback((reveals: DreamwellReveal[]) => {
    if (reveals.length === 0) return;
    if (judgmentPauseRef.current || dreamwellRevealRef.current != null) {
      setDreamwellRevealQueue((current) => [...current, ...reveals]);
      return;
    }
    const [nextReveal, ...rest] = reveals;
    dreamwellRevealRef.current = nextReveal;
    setDreamwellReveal(nextReveal);
    if (rest.length > 0) {
      setDreamwellRevealQueue((current) => [...current, ...rest]);
    }
  }, []);

  useEffect(() => {
    if (judgmentPause || dreamwellReveal || dreamwellRevealQueue.length === 0) return;
    const [nextReveal, ...rest] = dreamwellRevealQueue;
    setDreamwellReveal(nextReveal);
    setDreamwellRevealQueue(rest);
  }, [dreamwellReveal, dreamwellRevealQueue, judgmentPause]);

  useEffect(() => {
    if (!dreamwellReveal) return;
    const timer = window.setTimeout(() => {
      setDreamwellReveal(null);
    }, DREAMWELL_REVEAL_MS);
    return () => window.clearTimeout(timer);
  }, [dreamwellReveal]);

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
      // Skip this poll tick if we're in a stack pause
      if (performance.now() < stackPauseUntilRef.current) return;
      pollInFlight = true;
      void (async () => {
        const pollStart = performance.now();
        try {
          const pollRes = await api.poll();
          const pollMs = performance.now() - pollStart;
          // Check if this poll generation is still current
          if (pollGenerationRef.current !== myGeneration) return;
          const parsed = pollRes.commands ? parseCommandSequence(pollRes.commands) : null;
          const view = parsed?.battle ?? null;
          const dreamwellReveals = parsed ? resolveDreamwellReveals(parsed) : [];
          log.logPollResult(
            pollRes.response_type,
            pollRes.response_version,
            !!view,
            view?.user.can_act ?? null,
            pollMs,
          );
          enqueueDreamwellReveals(dreamwellReveals);
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
                if (dreamwellReveals.length > 0 && !view.user.can_act) {
                  setPendingResumePolling(true);
                }
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
            // Pause polling while cards are on the stack for visibility
            if (hasStackCards(view)) {
              stackPauseUntilRef.current = performance.now() + STACK_PAUSE_MS;
            }
            log.logStateUpdate("poll", view.turn_number, view.user.score, view.enemy.score, view.user.can_act, view.user.energy, view.game_over);
            if (dreamwellReveals.length > 0 && !view.user.can_act && !view.game_over) {
              setPendingResumePolling(true);
              if (pollRes.response_version) {
                log.logResponseVersionUpdate("poll_dreamwell", responseVersionRef.current, pollRes.response_version);
                responseVersionRef.current = pollRes.response_version;
              }
              stopPolling("dreamwell_reveal");
              return;
            }
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
  }, [enqueueDreamwellReveals, stopPolling]);

  useEffect(() => {
    if (!pendingResumePolling || judgmentPause || dreamwellReveal || dreamwellRevealQueue.length > 0) {
      return;
    }
    if (!battle || battle.user.can_act || battle.game_over) {
      setPendingResumePolling(false);
      return;
    }
    setPendingResumePolling(false);
    startPolling();
  }, [
    battle,
    dreamwellReveal,
    dreamwellRevealQueue.length,
    judgmentPause,
    pendingResumePolling,
    startPolling,
  ]);

  const sendAction = useCallback(
    (action: GameAction) => {
      if (isPolling || dreamwellReveal || dreamwellRevealQueue.length > 0) return;
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
          const parsed = parseCommandSequence(res.commands);
          const view = parsed.battle;
          const dreamwellReveals = resolveDreamwellReveals(parsed);
          log.logActionResult(action, durationMs, !!view, view?.user.can_act ?? null, null);
          enqueueDreamwellReveals(dreamwellReveals);
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
                if (dreamwellReveals.length > 0 && !view.user.can_act) {
                  setPendingResumePolling(true);
                }
                setJudgmentPause(true);
                log.logJudgmentPause(view.turn_number, view.user.score, view.enemy.score);
                return;
              }
            }
            prevBattleRef.current = view;
            setBattle(view);
            if (hasStackCards(view)) {
              stackPauseUntilRef.current = performance.now() + STACK_PAUSE_MS;
            }
            log.logStateUpdate("action", view.turn_number, view.user.score, view.enemy.score, view.user.can_act, view.user.energy, view.game_over);
            if (dreamwellReveals.length > 0) {
              if (!view.user.can_act && !view.game_over) {
                setPendingResumePolling(true);
              }
              return;
            }
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
    [dreamwellReveal, dreamwellRevealQueue.length, enqueueDreamwellReveals, isPolling, startPolling],
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
          const parsed = parseCommandSequence(res.commands);
          const initialDreamwellReveals = resolveDreamwellReveals(parsed);
          let deferPollingForDreamwell = false;
          enqueueDreamwellReveals(initialDreamwellReveals);
          let view = parsed.battle;
          if (view) {
            setBattle(view);
          }
          if (view && initialDreamwellReveals.length > 0 && !view.user.can_act && !view.game_over) {
            setPendingResumePolling(true);
            deferPollingForDreamwell = true;
            return;
          }
          // Poll with retries to get the updated state.
          for (let attempt = 0; attempt < 8; attempt++) {
            await new Promise((r) => setTimeout(r, 150 + attempt * 100));
            const pollStart = performance.now();
            const pollRes = await api.poll();
            const parsedPoll = pollRes.commands ? parseCommandSequence(pollRes.commands) : null;
            const pollView = parsedPoll?.battle ?? null;
            const dreamwellReveals = parsedPoll ? resolveDreamwellReveals(parsedPoll) : [];
            log.logPollResult(pollRes.response_type, pollRes.response_version, !!pollView, pollView?.user.can_act ?? null, performance.now() - pollStart);
            enqueueDreamwellReveals(dreamwellReveals);
            if (pollView) {
              setBattle(pollView);
              view = pollView;
            }
            if (pollRes.response_version) {
              log.logResponseVersionUpdate("debug_poll", responseVersionRef.current, pollRes.response_version);
              responseVersionRef.current = pollRes.response_version;
            }
            if (pollView && dreamwellReveals.length > 0 && !pollView.user.can_act && !pollView.game_over) {
              setPendingResumePolling(true);
              deferPollingForDreamwell = true;
              break;
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
          if (view && !view.user.can_act && !deferPollingForDreamwell) {
            startPolling();
          }
        } catch (e) {
          const msg = e instanceof Error ? e.message : "Debug action failed";
          log.logError("debug_action", msg);
          setError(msg);
        }
      })();
    },
    [enqueueDreamwellReveals, startPolling, stopPolling],
  );

  const continueFromJudgment = useCallback(() => {
    log.logJudgmentContinue();
    setJudgmentPause(false);
    if (dreamwellReveal || dreamwellRevealQueue.length > 0 || pendingResumePolling) {
      return;
    }
    if (battle && !battle.user.can_act && !battle.game_over) {
      startPolling();
    } else if (battle && battle.user.can_act) {
      setYourTurnCounter((c) => c + 1);
      wasPollingRef.current = false;
    }
  }, [battle, dreamwellReveal, dreamwellRevealQueue.length, pendingResumePolling, startPolling]);

  const reconnect = useCallback(
    (deck?: TestDeckName, userGoesSecond?: boolean) => {
      log.logReconnect(deck);
      void (async () => {
        const connectStart = performance.now();
        try {
          setError(null);
          setEvents([]);
          setDreamwellReveal(null);
          setDreamwellRevealQueue([]);
          setPendingResumePolling(false);
          setJudgmentPause(false);
          stopPolling("reconnect");
          setBattle(null);
          resetUserId();
          const res = await api.connect(deck, userGoesSecond);
          const durationMs = performance.now() - connectStart;
          log.logResponseVersionUpdate("connect", responseVersionRef.current, res.response_version);
          responseVersionRef.current = res.response_version;
          const parsed = parseCommandSequence(res.commands);
          const view = parsed.battle;
          enqueueDreamwellReveals(resolveDreamwellReveals(parsed));
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
    [enqueueDreamwellReveals, stopPolling],
  );

  return (
    <BattleContext.Provider
      value={{ battle, isPolling, error, events, yourTurnCounter, judgmentPause, dreamwellReveal, continueFromJudgment, sendAction, sendDebugAction, reconnect }}
    >
      {children}
    </BattleContext.Provider>
  );
}
