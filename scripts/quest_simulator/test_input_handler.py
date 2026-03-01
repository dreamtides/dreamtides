"""Tests for raw terminal input handler."""

import os
import signal
import sys
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
    ensure_terminal_restored,
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

    def _pipe_stdin(self, data: bytes) -> tuple[int, MagicMock]:
        """Create a pipe fd with data and a mock stdin pointing to it."""
        r_fd, w_fd = os.pipe()
        os.write(w_fd, data)
        os.close(w_fd)
        mock = MagicMock()
        mock.fileno.return_value = r_fd
        return r_fd, mock

    def test_enter_cr(self) -> None:
        r_fd, mock_stdin = self._pipe_stdin(b"\r")
        try:
            with patch("input_handler.sys.stdin", mock_stdin):
                assert _read_key() == KEY_ENTER
        finally:
            os.close(r_fd)

    def test_enter_lf(self) -> None:
        r_fd, mock_stdin = self._pipe_stdin(b"\n")
        try:
            with patch("input_handler.sys.stdin", mock_stdin):
                assert _read_key() == KEY_ENTER
        finally:
            os.close(r_fd)

    def test_space(self) -> None:
        r_fd, mock_stdin = self._pipe_stdin(b" ")
        try:
            with patch("input_handler.sys.stdin", mock_stdin):
                assert _read_key() == KEY_SPACE
        finally:
            os.close(r_fd)

    def test_ctrl_c(self) -> None:
        r_fd, mock_stdin = self._pipe_stdin(b"\x03")
        try:
            with patch("input_handler.sys.stdin", mock_stdin):
                assert _read_key() == KEY_QUIT
        finally:
            os.close(r_fd)

    def test_eof(self) -> None:
        r_fd, mock_stdin = self._pipe_stdin(b"")
        try:
            with patch("input_handler.sys.stdin", mock_stdin):
                assert _read_key() == KEY_QUIT
        finally:
            os.close(r_fd)

    def test_unknown_char(self) -> None:
        r_fd, mock_stdin = self._pipe_stdin(b"x")
        try:
            with patch("input_handler.sys.stdin", mock_stdin):
                assert _read_key() == KEY_UNKNOWN
        finally:
            os.close(r_fd)

    def test_arrow_up(self) -> None:
        r_fd, mock_stdin = self._pipe_stdin(b"\x1b[A")
        try:
            with patch("input_handler.sys.stdin", mock_stdin):
                assert _read_key() == KEY_UP
        finally:
            os.close(r_fd)

    def test_arrow_down(self) -> None:
        r_fd, mock_stdin = self._pipe_stdin(b"\x1b[B")
        try:
            with patch("input_handler.sys.stdin", mock_stdin):
                assert _read_key() == KEY_DOWN
        finally:
            os.close(r_fd)

    def test_arrow_right(self) -> None:
        r_fd, mock_stdin = self._pipe_stdin(b"\x1b[C")
        try:
            with patch("input_handler.sys.stdin", mock_stdin):
                assert _read_key() == KEY_RIGHT
        finally:
            os.close(r_fd)

    def test_arrow_left(self) -> None:
        r_fd, mock_stdin = self._pipe_stdin(b"\x1b[D")
        try:
            with patch("input_handler.sys.stdin", mock_stdin):
                assert _read_key() == KEY_LEFT
        finally:
            os.close(r_fd)

    def test_bare_escape(self) -> None:
        r_fd, mock_stdin = self._pipe_stdin(b"\x1b")
        try:
            with patch("input_handler.sys.stdin", mock_stdin):
                assert _read_key() == KEY_QUIT
        finally:
            os.close(r_fd)


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


class TestEnsureTerminalRestored:
    """Public API for external callers to force terminal restoration."""

    def test_ensure_terminal_restored_calls_release(self) -> None:
        import input_handler

        original = object()
        input_handler._saved_termios = [original]  # type: ignore[list-item]
        with patch("input_handler._HAS_TERMIOS", True):
            mock_fd = MagicMock(return_value=0)
            with patch("input_handler.sys.stdin") as mock_stdin:
                mock_stdin.fileno = mock_fd
                with patch("input_handler.termios"):
                    ensure_terminal_restored()
                    assert input_handler._saved_termios is None

    def test_ensure_terminal_restored_noop_when_no_state(self) -> None:
        import input_handler

        input_handler._saved_termios = None
        ensure_terminal_restored()
        assert input_handler._saved_termios is None


class TestSigintHandlerMessage:
    """SIGINT handler should print quest abandoned message."""

    def test_sigint_handler_exits_with_130(self) -> None:
        import input_handler

        input_handler._saved_termios = None
        with patch("input_handler._HAS_TERMIOS", True):
            with patch("builtins.print") as mock_print:
                try:
                    input_handler._sigint_handler(signal.SIGINT, None)
                except SystemExit as e:
                    assert e.code == 130
                else:
                    assert False, "Expected SystemExit"

    def test_sigint_handler_prints_abandoned_message(self) -> None:
        import input_handler

        input_handler._saved_termios = None
        with patch("input_handler._HAS_TERMIOS", True):
            with patch("builtins.print") as mock_print:
                try:
                    input_handler._sigint_handler(signal.SIGINT, None)
                except SystemExit:
                    pass
                printed = " ".join(
                    str(a) for c in mock_print.call_args_list for a in c.args
                )
                assert "Quest abandoned" in printed


class TestSigtstpHandler:
    """SIGTSTP (Ctrl+Z) should restore terminal before suspending."""

    def test_sigtstp_handler_restores_terminal(self) -> None:
        import input_handler

        original = [object()]
        input_handler._saved_termios = original  # type: ignore[list-item]
        with patch("input_handler._HAS_TERMIOS", True):
            mock_fd = MagicMock(return_value=0)
            with patch("input_handler.sys.stdin") as mock_stdin:
                mock_stdin.fileno = mock_fd
                with patch("input_handler.termios") as mock_termios:
                    with patch("input_handler.os.kill") as mock_kill:
                        with patch("input_handler.os.getpid", return_value=123):
                            input_handler._sigtstp_handler(signal.SIGTSTP, None)
                            # Should have restored terminal
                            mock_termios.tcsetattr.assert_called()
                            # Should have sent SIGSTOP to self
                            mock_kill.assert_called_with(123, signal.SIGSTOP)

        input_handler._saved_termios = None

    def test_sigtstp_handler_resets_default_handler(self) -> None:
        """After restoring, SIGTSTP should be reset to SIG_DFL so the OS can suspend."""
        import input_handler

        input_handler._saved_termios = None
        with patch("input_handler._HAS_TERMIOS", True):
            with patch("input_handler.os.kill"):
                with patch("input_handler.os.getpid", return_value=1):
                    with patch("input_handler.signal.signal") as mock_signal:
                        input_handler._sigtstp_handler(signal.SIGTSTP, None)
                        mock_signal.assert_called_with(signal.SIGTSTP, signal.SIG_DFL)
