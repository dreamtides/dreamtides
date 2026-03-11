"""Localhost web server for the quest simulator.

Runs the existing quest flow on a background thread and exposes
a browser UI via a minimal HTTP server (zero extra dependencies).
"""

import http.server
import json
import mimetypes
import queue
import random
import sys
import threading
import time
import webbrowser
from pathlib import Path
from typing import Any

# Ensure quest_simulator siblings are importable (quest_sim.py adds draft_simulator
# to sys.path before importing this module, so those imports also work here).
_QS_DIR = str(Path(__file__).resolve().parent.parent)
if _QS_DIR not in sys.path:
    sys.path.insert(0, _QS_DIR)

_DRAFT_SIM_DIR = str(Path(__file__).resolve().parent.parent.parent / "draft_simulator")
if _DRAFT_SIM_DIR not in sys.path:
    sys.path.insert(0, _DRAFT_SIM_DIR)

import agents
import card_generator
import cube_manager
import flow
import image_cache
import input_handler
from draft_models import CubeConsumptionMode
import atlas
import data_loader
from jsonl_log import SessionLogger
from quest_sim import _build_draft_config
from quest_state import QuestState
from site_dispatch import SiteData

_STATIC_DIR = Path(__file__).parent / "static"


def _serialize_deck_card(dc: Any) -> dict:
    design = dc.instance.design
    image_hash = None
    if design.image_number is not None:
        image_hash = image_cache.get_image_cache_key(design.image_number)
    return {
        "name": design.name,
        "image_hash": image_hash,
        "energy_cost": design.energy_cost,
        "card_type": design.card_type,
        "rules_text": design.rules_text,
        "spark": design.spark,
        "resonance": list(design.resonance),
    }


def _make_handler(
    prompt_q: queue.Queue,
    response_q: queue.Queue,
    game_thread: threading.Thread,
    total_battles: int,
) -> type[http.server.BaseHTTPRequestHandler]:
    """Return a request handler class closed over the shared state.

    The current prompt is held in `_pending` (a one-element list used as a
    mutable cell) so that multiple concurrent poll requests all see it, and
    the game thread only advances once a choice is submitted.
    """
    _pending: list[Any] = [None]  # _pending[0] = current prompt dict or None
    _lock = threading.Lock()

    class Handler(http.server.BaseHTTPRequestHandler):
        def log_message(self, format: str, *args: Any) -> None:
            pass  # suppress access log noise

        def do_GET(self) -> None:
            if self.path in ("/", "/index.html"):
                self._serve_file(_STATIC_DIR / "index.html", "text/html")
            elif self.path.startswith("/static/"):
                rel = self.path[len("/static/") :]
                self._serve_file(_STATIC_DIR / rel)
            elif self.path == "/api/prompt":
                self._handle_prompt()
            elif self.path.startswith("/api/images/"):
                hash_key = self.path[len("/api/images/") :]
                self._serve_image(hash_key)
            else:
                self.send_response(404)
                self.end_headers()

        def do_POST(self) -> None:
            if self.path == "/api/choice":
                self._handle_choice()
            else:
                self.send_response(404)
                self.end_headers()

        def _serve_file(self, path: Path, content_type: str | None = None) -> None:
            try:
                data = path.read_bytes()
            except (FileNotFoundError, OSError):
                self.send_response(404)
                self.end_headers()
                return
            if content_type is None:
                guessed, _ = mimetypes.guess_type(str(path))
                content_type = guessed or "application/octet-stream"
            self.send_response(200)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def _handle_prompt(self) -> None:
            # Return the pending prompt immediately if the game is waiting for a choice.
            with _lock:
                if _pending[0] is not None:
                    self._json(_pending[0])
                    return

            # Otherwise long-poll up to 30 s for the next prompt.
            deadline = time.monotonic() + 30
            while time.monotonic() < deadline:
                try:
                    prompt = prompt_q.get(timeout=0.5)
                    with _lock:
                        _pending[0] = prompt
                    self._json(prompt)
                    return
                except queue.Empty:
                    with _lock:
                        if _pending[0] is not None:
                            self._json(_pending[0])
                            return
                    if not game_thread.is_alive():
                        self._json(
                            {"type": "game_over", "total_battles": total_battles}
                        )
                        return
            self._json({"type": "waiting"})

        def _handle_choice(self) -> None:
            length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(length)
            try:
                data = json.loads(body)
                choice = data["choice"]
            except (json.JSONDecodeError, KeyError):
                self.send_response(400)
                self.end_headers()
                return
            with _lock:
                _pending[0] = None
            response_q.put(choice)
            self._json({"ok": True})

        def _serve_image(self, hash_key: str) -> None:
            image_path = image_cache.get_cache_dir() / hash_key
            if not image_path.exists():
                self.send_response(404)
                self.end_headers()
                return
            data = image_path.read_bytes()
            self.send_response(200)
            self.send_header("Content-Type", "image/jpeg")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def _json(self, obj: Any) -> None:
            data = json.dumps(obj).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

    return Handler


def run_web_server(args: Any) -> None:
    """Initialize the quest and serve the browser UI."""
    seed: int = args.seed if args.seed is not None else random.randint(0, 2**32)
    rng = random.Random(seed)

    cfg = _build_draft_config(synthetic=args.synthetic, real_only=args.real_only)

    cards = card_generator.generate_cards(cfg, rng)
    copies_per_card: int | dict[str, int] = cube_manager.build_copies_map(
        cards, cfg.rarity
    )
    cube = cube_manager.CubeManager(
        designs=cards,
        copies_per_card=copies_per_card,
        consumption_mode=CubeConsumptionMode.WITH_REPLACEMENT,
    )

    human_agent = agents.create_agent(archetype_count=cfg.cards.archetype_count)
    ai_agents = [
        agents.create_agent(archetype_count=cfg.cards.archetype_count)
        for _ in range(cfg.draft.seat_count - 1)
    ]

    config = data_loader.load_config()
    dreamcallers = data_loader.load_dreamcallers()
    dreamsigns = data_loader.load_dreamsigns()
    journeys = data_loader.load_journeys()
    offers = data_loader.load_offers()
    banes = data_loader.load_banes()
    bosses = data_loader.load_bosses()

    quest_config: dict[str, Any] = config.get("quest", {})
    starting_essence: int = int(quest_config.get("starting_essence", 250))
    max_deck: int = int(quest_config.get("max_deck", 50))
    min_deck: int = int(quest_config.get("min_deck", 25))
    max_dreamsigns: int = int(quest_config.get("max_dreamsigns", 12))
    total_battles: int = int(quest_config.get("total_battles", 7))

    state = QuestState(
        essence=starting_essence,
        rng=rng,
        human_agent=human_agent,
        ai_agents=ai_agents,
        cube=cube,
        draft_cfg=cfg,
        packs=None,
        max_deck=max_deck,
        min_deck=min_deck,
        max_dreamsigns=max_dreamsigns,
        debug=args.debug,
    )

    data = SiteData(
        dreamcallers=dreamcallers,
        dreamsigns=dreamsigns,
        journeys=journeys,
        offers=offers,
        banes=banes,
        bosses=bosses,
        config=config,
    )

    nodes = atlas.initialize_atlas(rng)
    logger = SessionLogger(seed)

    prompt_q: queue.Queue = queue.Queue()
    response_q: queue.Queue = queue.Queue()

    def state_callback() -> dict:
        return {
            "essence": state.essence,
            "completion_level": state.completion_level,
            "total_battles": total_battles,
            "deck": [_serialize_deck_card(dc) for dc in state.deck],
            "dreamsigns": [ds.name for ds in state.dreamsigns],
            "dreamcaller": state.dreamcaller.name if state.dreamcaller else None,
            "dreamcaller_archetype": (
                state.dreamcaller.archetype if state.dreamcaller else None
            ),
            "deck_count": state.deck_count(),
        }

    # Build card name → image hash map and prefetch missing images in background.
    card_image_map: dict[str, str | None] = {
        d.name: (
            image_cache.get_image_cache_key(d.image_number)
            if d.image_number is not None
            else None
        )
        for d in cards
    }
    card_spark_map: dict[str, int | None] = {d.name: d.spark for d in cards}
    card_resonance_map: dict[str, tuple[str, ...]] = {
        d.name: d.resonance for d in cards
    }
    input_handler.set_card_name_image_map(card_image_map)
    input_handler.set_card_name_spark_map(card_spark_map)
    input_handler.set_card_name_resonance_map(card_resonance_map)

    def _prefetch_images() -> None:
        for d in cards:
            if d.image_number is not None and card_image_map.get(d.name) is None:
                result = image_cache.ensure_image_cached(d.image_number)
                if result:
                    card_image_map[d.name] = result

    threading.Thread(target=_prefetch_images, daemon=True).start()

    input_handler.install_output_capture()
    input_handler.set_web_mode(prompt_q, response_q, state_callback)

    def run_game() -> None:
        try:
            flow.run_quest(
                state=state,
                nodes=nodes,
                data=data,
                total_battles=total_battles,
                logger=logger,
            )
        finally:
            logger.close()

    game_thread = threading.Thread(target=run_game, daemon=True)
    game_thread.start()

    handler_class = _make_handler(prompt_q, response_q, game_thread, total_battles)
    server = http.server.ThreadingHTTPServer(("localhost", args.port), handler_class)
    print(f"  Quest simulator web UI: http://localhost:{args.port}")
    webbrowser.open(f"http://localhost:{args.port}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n  Web server stopped.")
