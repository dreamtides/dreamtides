"""Tests for raw terminal input handler."""

import io
import sys
from typing import Optional
from unittest.mock import MagicMock, patch

from input_handler import (
    KEY_DOWN,
    KEY_ENTER,
    KEY_LEFT,
    KEY_QUIT,
    KEY_RIGHT,
    KEY_SPACE,
    KEY_UNKNOWN,
    KEY_UP,
    _read_key,
    confirm_decline,
    multi_select,
    single_select,
    wait_for_continue,
)


class TestNonInteractiveFallbacks:
    """Non-TTY paths should return safe defaults without crashing."""

    @patch("input_handler._is_interactive", return_value=False)
    def test_single_select_returns_initial(self, _mock: object) -> None:
        assert single_select(["a", "b", "c"]) == 0
        assert single_select(["a", "b", "c"], initial=2) == 2

    @patch("input_handler._is_interactive", return_value=False)
    def test_single_select_clamps_initial(self, _mock: object) -> None:
        assert single_select(["a", "b"], initial=99) == 1

    @patch("input_handler._is_interactive", return_value=False)
    def test_single_select_empty_options(self, _mock: object) -> None:
        assert single_select([]) == 0

    @patch("input_handler._is_interactive", return_value=False)
    def test_multi_select_returns_empty(self, _mock: object) -> None:
        assert multi_select(["a", "b", "c"]) == []

    @patch("input_handler._is_interactive", return_value=False)
    def test_multi_select_preserves_initial_toggled(self, _mock: object) -> None:
        result = multi_select(["a", "b", "c"], initial_toggled={0, 2})
        assert result == [0, 2]

    @patch("input_handler._is_interactive", return_value=False)
    def test_multi_select_filters_out_of_range_initial(self, _mock: object) -> None:
        result = multi_select(["a", "b"], initial_toggled={0, 5})
        assert result == [0]

    @patch("input_handler._is_interactive", return_value=False)
    def test_multi_select_empty_options(self, _mock: object) -> None:
        assert multi_select([]) == []

    @patch("input_handler._is_interactive", return_value=False)
    def test_confirm_decline_returns_true(self, _mock: object) -> None:
        assert confirm_decline() is True

    @patch("input_handler._is_interactive", return_value=False)
    def test_wait_for_continue_returns(self, _mock: object) -> None:
        wait_for_continue()


class TestReadKey:
    """Key parsing from raw stdin bytes."""

    def _make_stdin(self, data: str) -> io.StringIO:
        return io.StringIO(data)

    @patch("input_handler.select.select", return_value=([], [], []))
    def test_enter_cr(self, _sel: object) -> None:
        with patch("input_handler.sys.stdin", self._make_stdin("\r")):
            assert _read_key() == KEY_ENTER

    @patch("input_handler.select.select", return_value=([], [], []))
    def test_enter_lf(self, _sel: object) -> None:
        with patch("input_handler.sys.stdin", self._make_stdin("\n")):
            assert _read_key() == KEY_ENTER

    @patch("input_handler.select.select", return_value=([], [], []))
    def test_space(self, _sel: object) -> None:
        with patch("input_handler.sys.stdin", self._make_stdin(" ")):
            assert _read_key() == KEY_SPACE

    @patch("input_handler.select.select", return_value=([], [], []))
    def test_ctrl_c(self, _sel: object) -> None:
        with patch("input_handler.sys.stdin", self._make_stdin("\x03")):
            assert _read_key() == KEY_QUIT

    @patch("input_handler.select.select", return_value=([], [], []))
    def test_eof(self, _sel: object) -> None:
        with patch("input_handler.sys.stdin", self._make_stdin("")):
            assert _read_key() == KEY_QUIT

    @patch("input_handler.select.select", return_value=([], [], []))
    def test_unknown_char(self, _sel: object) -> None:
        with patch("input_handler.sys.stdin", self._make_stdin("x")):
            assert _read_key() == KEY_UNKNOWN

    def test_arrow_up(self) -> None:
        stdin = self._make_stdin("\x1b[A")
        with patch("input_handler.sys.stdin", stdin):
            with patch(
                "input_handler.select.select", return_value=([stdin], [], [])
            ):
                assert _read_key() == KEY_UP

    def test_arrow_down(self) -> None:
        stdin = self._make_stdin("\x1b[B")
        with patch("input_handler.sys.stdin", stdin):
            with patch(
                "input_handler.select.select", return_value=([stdin], [], [])
            ):
                assert _read_key() == KEY_DOWN

    def test_arrow_right(self) -> None:
        stdin = self._make_stdin("\x1b[C")
        with patch("input_handler.sys.stdin", stdin):
            with patch(
                "input_handler.select.select", return_value=([stdin], [], [])
            ):
                assert _read_key() == KEY_RIGHT

    def test_arrow_left(self) -> None:
        stdin = self._make_stdin("\x1b[D")
        with patch("input_handler.sys.stdin", stdin):
            with patch(
                "input_handler.select.select", return_value=([stdin], [], [])
            ):
                assert _read_key() == KEY_LEFT

    def test_bare_escape(self) -> None:
        stdin = self._make_stdin("\x1b")
        with patch("input_handler.sys.stdin", stdin):
            with patch(
                "input_handler.select.select", return_value=([], [], [])
            ):
                assert _read_key() == KEY_QUIT


class TestTermiosStateManagement:
    """Verify save/restore/release state transitions."""

    def test_restore_does_not_clear_saved_state(self) -> None:
        import input_handler

        original = object()
        input_handler._saved_termios = [original]  # type: ignore[list-item]
        with patch("input_handler._HAS_TERMIOS", True):
            mock_fd = MagicMock(return_value=0)
            with patch("input_handler.sys.stdin") as mock_stdin:
                mock_stdin.fileno = mock_fd
                with patch("input_handler.termios") as mock_termios:
                    input_handler._restore_termios()
                    # State should NOT be cleared after restore
                    assert input_handler._saved_termios is not None
                    mock_termios.tcsetattr.assert_called_once()

        input_handler._saved_termios = None

    def test_release_clears_saved_state(self) -> None:
        import input_handler

        original = object()
        input_handler._saved_termios = [original]  # type: ignore[list-item]
        with patch("input_handler._HAS_TERMIOS", True):
            mock_fd = MagicMock(return_value=0)
            with patch("input_handler.sys.stdin") as mock_stdin:
                mock_stdin.fileno = mock_fd
                with patch("input_handler.termios"):
                    input_handler._release_termios()
                    # State SHOULD be cleared after release
                    assert input_handler._saved_termios is None

    def test_restore_noop_when_no_saved_state(self) -> None:
        import input_handler

        input_handler._saved_termios = None
        with patch("input_handler._HAS_TERMIOS", True):
            # Should not raise
            input_handler._restore_termios()
            assert input_handler._saved_termios is None


class TestRenderCallbacks:
    """Verify render_fn parameters are accepted."""

    @patch("input_handler._is_interactive", return_value=False)
    def test_single_select_accepts_render_fn(self, _mock: object) -> None:
        calls: list[tuple[int, str, bool]] = []

        def render(idx: int, opt: str, selected: bool) -> str:
            calls.append((idx, opt, selected))
            return f"{'*' if selected else ' '} {opt}"

        single_select(["a"], render_fn=render)

    @patch("input_handler._is_interactive", return_value=False)
    def test_multi_select_accepts_render_fn(self, _mock: object) -> None:
        def render(idx: int, opt: str, highlighted: bool, checked: bool) -> str:
            return f"{opt}"

        multi_select(["a"], render_fn=render)

    @patch("input_handler._is_interactive", return_value=False)
    def test_confirm_decline_accepts_render_fn(self, _mock: object) -> None:
        def render(accept: str, decline: str, accepted: bool) -> str:
            return f"{accept if accepted else decline}"

        confirm_decline(render_fn=render)

    @patch("input_handler._is_interactive", return_value=False)
    def test_wait_for_continue_accepts_render_fn(self, _mock: object) -> None:
        def render() -> str:
            return "Continue..."

        wait_for_continue(render_fn=render)
