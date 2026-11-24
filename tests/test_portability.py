"""Files come out byte for byte the same on every system, and a PNG that is not written says why."""
import base64
import contextlib
import io
import os
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "src"))

from open_diagram import Diagram, Panel, svg_to_png  # noqa: E402
from open_diagram import cli, mcp_server, render  # noqa: E402
from open_diagram.blocks import _data_uri  # noqa: E402

ICON = '<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4">\n<rect width="4" height="4"/>\n</svg>\n'
SOURCE = '{"title": "Test", "columns": [[{"title": "Alpha", "tiles": ["x"]}]]}'
TINY_PNG = base64.b64decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=")


def small_diagram():
    d = Diagram("Test")
    d.add(Panel("Alpha", ["x"]))
    return d


class LineEndingTests(unittest.TestCase):
    """Issues #3 and #4: Windows line endings changed the bytes of saved pictures."""

    def test_svg_icon_with_windows_line_endings_embeds_the_same_bytes(self):
        with tempfile.TemporaryDirectory() as tmp:
            unix, windows = os.path.join(tmp, "unix.svg"), os.path.join(tmp, "windows.svg")
            with open(unix, "wb") as fh:
                fh.write(ICON.encode("utf-8"))
            with open(windows, "wb") as fh:
                fh.write(ICON.replace("\n", "\r\n").encode("utf-8"))
            self.assertEqual(_data_uri(unix), _data_uri(windows))

    def test_png_icon_bytes_are_never_changed(self):
        data = TINY_PNG + b"\r\n\r\n"
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "icon.png")
            with open(path, "wb") as fh:
                fh.write(data)
            self.assertEqual(base64.b64decode(_data_uri(path).split(",", 1)[1]), data)

    def test_saved_svg_is_exactly_what_render_returns(self):
        d = small_diagram()
        svg, _ = d.render()
        with tempfile.TemporaryDirectory() as tmp:
            base = os.path.join(tmp, "saved")
            with contextlib.redirect_stdout(io.StringIO()):
                d.save(base, png=False)
            with open(base + ".svg", "rb") as fh:
                self.assertEqual(fh.read(), svg.encode("utf-8"))
            result = mcp_server.draw_files(SOURCE, "json", "mcp", tmp, png=False)
            with open(result["files"][0], "rb") as fh:
                self.assertEqual(fh.read(), mcp_server.build(SOURCE, "json").render()[0].encode("utf-8"))

    def test_new_writes_the_template_with_unix_line_endings(self):
        with tempfile.TemporaryDirectory() as tmp:
            target = os.path.join(tmp, "start.json")
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(cli.main(["new", "aws_etl", "-o", target]), 0)
            with open(target, "rb") as fh:
                self.assertNotIn(b"\r", fh.read())


class PngTests(unittest.TestCase):
    """Issue #5: a failed browser run reported success over an old PNG, and every failure read 'not found'."""

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.dir = tmp.name
        self.svg = os.path.join(self.dir, "a.svg")
        self.png = os.path.join(self.dir, "a.png")
        with open(self.svg, "w", encoding="utf-8") as fh:
            fh.write('<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"></svg>')
        self.old = b"an older picture"
        with open(self.png, "wb") as fh:
            fh.write(self.old)

    def png_bytes(self):
        with open(self.png, "rb") as fh:
            return fh.read()

    def test_browser_that_fails_leaves_the_old_png_and_says_so(self):
        # Python stands in for the browser: it stops with exit code 2 on the unknown option --headless=new.
        self.assertFalse(svg_to_png(self.svg, self.png, 10, 10, chrome=sys.executable))
        self.assertIn("exit code 2", render.png_error(self.svg, self.png, 10, 10, chrome=sys.executable))
        self.assertEqual(self.png_bytes(), self.old)

    def test_timeout_no_file_and_no_browser_are_told_apart(self):
        with patch("subprocess.run", side_effect=subprocess.TimeoutExpired(["chrome"], 180)):
            self.assertIn("did not finish", render.png_error(self.svg, self.png, 10, 10, chrome=sys.executable))
        with patch("subprocess.run", return_value=subprocess.CompletedProcess([], 0, b"", b"")):
            self.assertIn("without writing a PNG",
                          render.png_error(self.svg, self.png, 10, 10, chrome=sys.executable))
        with patch.object(render, "find_chrome", return_value=None):
            self.assertEqual(render.png_error(self.svg, self.png, 10, 10), render.NOT_FOUND)
        self.assertEqual(self.png_bytes(), self.old)

    def test_new_png_replaces_the_old_one(self):
        def browser(cmd, **kwargs):
            shot = next(a.split("=", 1)[1] for a in cmd if a.startswith("--screenshot="))
            with open(shot, "wb") as fh:
                fh.write(TINY_PNG)
            return subprocess.CompletedProcess(cmd, 0, b"", b"")

        with patch("subprocess.run", side_effect=browser):
            self.assertTrue(svg_to_png(self.svg, self.png, 10, 10, chrome=sys.executable))
        self.assertEqual(self.png_bytes(), TINY_PNG)

    def test_save_png_command_and_mcp_give_the_real_reason(self):
        base = os.path.join(self.dir, "d")
        with patch("subprocess.run", side_effect=subprocess.TimeoutExpired(["chrome"], 180)), \
                patch.object(render, "find_chrome", return_value=sys.executable):
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                report = small_diagram().save(base, png=True)
            self.assertIn("PNG skipped: the browser at", out.getvalue())
            self.assertIn("did not finish", out.getvalue())
            self.assertEqual(report.files, [base + ".svg"])
            err = io.StringIO()
            with contextlib.redirect_stderr(err), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(cli._png(base + ".svg", 2.0), 2)
            self.assertIn("did not finish", err.getvalue())
            result = mcp_server.draw_files(SOURCE, "json", "m", self.dir, png=True)
            self.assertIsNone(result["png"])
            self.assertIn("did not finish", result["text"])
            self.assertNotIn("not found", out.getvalue() + err.getvalue() + result["text"])


if __name__ == "__main__":
    unittest.main()
