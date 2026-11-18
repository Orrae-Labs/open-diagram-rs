"""Run with: python3 -m unittest discover -s tests"""
import importlib.util
import os
import sys
import tempfile
import unittest

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "src"))

from open_diagram import Diagram, Graph, LayoutError, Panel, Tile, text_width, wrap  # noqa: E402
from open_diagram.fonts import face  # noqa: E402

EXAMPLES = ["quickstart", "systems_map", "layers", "concept_map"]


def load_example(name):
    spec = importlib.util.spec_from_file_location(name, os.path.join(ROOT, "examples", name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class TextTests(unittest.TestCase):
    def test_width_grows_with_text(self):
        self.assertLess(text_width("ab", 12), text_width("abc", 12))

    def test_kerning_is_applied(self):
        f = face("regular")
        self.assertLess(f.kern(*f.names("AV")), 0)
        self.assertLess(text_width("AV", 100), text_width("A", 100) + text_width("V", 100))

    def test_wrap_respects_width(self):
        for line in wrap("one two three four five six seven", 12, 70):
            self.assertTrue(text_width(line, 12) <= 70 or " " not in line, line)

    def test_newline_forces_a_break(self):
        self.assertEqual(wrap("first\nsecond", 12, None), ["first", "second"])


class CheckTests(unittest.TestCase):
    def problems(self, d):
        return d.render()[1].problems

    def test_overlap_is_reported(self):
        d = Diagram("t")
        d.add(Panel("Alpha", ["x"]), 0, 0)
        d.add(Panel("Beta", ["y"]), 20, 20)
        self.assertTrue(any("'Alpha' and 'Beta' overlap" in p for p in self.problems(d)))

    def test_small_gap_is_reported(self):
        d = Diagram("t")
        a = d.add(Panel("Alpha", ["x"]))
        d.right_of(a, Panel("Beta", ["y"]), gap=5)
        self.assertTrue(any("only 5 px apart" in p for p in self.problems(d)))

    def test_clean_layout_has_no_problems(self):
        d = Diagram("t")
        a = d.add(Panel("Alpha", ["x"]))
        b = d.right_of(a, Panel("Beta", ["y"]))
        d.link(a, b)
        self.assertEqual(self.problems(d), [])

    def test_crossing_line_is_reported_unless_allowed(self):
        d = Diagram("t")
        a = d.add(Panel("Alpha", ["x"]))
        b = d.right_of(a, Panel("Beta", ["y"]), align="center")
        c = d.right_of(b, Panel("Gamma", ["z"]), align="center")
        line = d.link(a, c)
        self.assertTrue(any("crosses 'Beta'" in p for p in self.problems(d)))
        line.cross_ok = True
        self.assertEqual(self.problems(d), [])

    def test_lane_route_avoids_the_crossing(self):
        d = Diagram("t")
        a = d.add(Panel("Alpha", ["x"]))
        b = d.right_of(a, Panel("Beta", ["y"]), align="center")
        c = d.right_of(b, Panel("Gamma", ["z"]), align="center")
        d.link(a, c, start="top", end="top", via_y=a.y - 40)
        self.assertEqual(self.problems(d), [])

    def test_graph_label_collision_is_reported(self):
        g = Graph({"a": "A", "b": "B"}, [("a", "b", "a very long label for a short line")],
                  {"a": (0, 0), "b": (0.6, 0)})
        d = Diagram("t")
        d.add(Panel("Map", g))
        self.assertTrue(any("touches the node" in p for p in self.problems(d)))

    def test_unknown_status_names_the_choices(self):
        d = Diagram("t")
        d.add(Panel("Alpha", [Tile("x", status="nope")]))
        with self.assertRaisesRegex(ValueError, "live"):
            d.render()

    def test_save_raises_but_still_writes(self):
        d = Diagram("t")
        d.add(Panel("Alpha", ["x"]), 0, 0)
        d.add(Panel("Beta", ["y"]), 10, 10)
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "out")
            with self.assertRaises(LayoutError) as caught:
                d.save(path, png=False)
            self.assertTrue(os.path.exists(path + ".svg"))
            self.assertIn("overlap", str(caught.exception))


class ExampleTests(unittest.TestCase):
    def test_examples_build_cleanly_and_repeatably(self):
        for name in EXAMPLES:
            with self.subTest(example=name):
                module = load_example(name)
                svg, report = module.build().render()
                self.assertEqual(report.problems, [])
                self.assertGreaterEqual(report.min_text, 10)
                self.assertEqual(svg, module.build().render()[0])


if __name__ == "__main__":
    unittest.main()
