"""Resource diagrams, enclosure styles, and compatibility with existing pictures."""
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from open_diagram import Diagram, Resource, SpecError, Theme, from_spec  # noqa: E402
from open_diagram.cli import load  # noqa: E402
from open_diagram.svg import Canvas  # noqa: E402
from open_diagram.theme import shade  # noqa: E402

NS = {"s": "http://www.w3.org/2000/svg"}


class CompatibilityTests(unittest.TestCase):
    def test_existing_examples_match_committed_svg(self):
        for path in sorted((ROOT / "examples").iterdir()):
            if path.suffix not in (".py", ".json", ".mmd"):
                continue
            with self.subTest(example=path.name):
                svg, report = load(str(path)).render()
                self.assertEqual(report.problems, [])
                self.assertEqual(svg.encode("utf-8"), (ROOT / "docs/images" / (path.stem + ".svg")).read_bytes())


class ResourceTests(unittest.TestCase):
    def test_icon_is_large_embedded_and_connects_at_its_edge(self):
        with tempfile.TemporaryDirectory() as tmp:
            icon = Path(tmp) / "icon.svg"
            icon.write_text('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><circle cx="32" cy="32" r="28"/></svg>')
            d = Diagram("Resources")
            a = d.add(Resource("Long extractor label", icon=str(icon), sub="AWS Lambda"))
            b = d.right_of(a, Resource("Bucket", icon=str(icon)))
            line = d.link(a, b)
            svg, report = d.render()
            self.assertEqual(report.problems, [])
            tree = ET.fromstring(svg)
            images = tree.findall(".//s:image", NS)
            self.assertEqual(len(images), 2)
            self.assertTrue(all(x.get("href").startswith("data:image/svg+xml;base64,") for x in images))
            self.assertEqual(float(images[0].get("width")), 64)
            self.assertEqual(line.start_pt[0], a.x + (a.w + 64) / 2)
            self.assertNotIn("States shown: Live", d.describe())
            self.assertIn("Long extractor label (AWS Lambda)", d.describe())

    def test_missing_icon_and_invalid_dimensions_are_named(self):
        with self.assertRaisesRegex(FileNotFoundError, "Extractor.*icon"):
            Resource("Extractor", icon="missing-cloud-icon.svg")
        for kwargs in ({"icon_size": 0}, {"max_width": -1}, {"icon_size": float("nan")}, {"border": "dahsed"}):
            with self.subTest(kwargs=kwargs), self.assertRaises(ValueError):
                Resource("Extractor", **kwargs)

    def test_wrapped_labels_and_subtitles_fit_the_measured_box(self):
        resource = Resource("A long resource label that wraps", sub="A detailed resource description", max_width=140)
        theme = Theme()
        w, h = resource.measure(theme)
        c = Canvas(w, h)
        problems = []
        resource.draw(c, 0, 0, w - 1, h - 1, theme, problems.append)
        self.assertTrue(any("needs" in p for p in problems))
        self.assertGreater(h, theme.line_h * 2)

    def test_resource_border_styles_do_not_add_deployment_state(self):
        for style in (None, "solid", "dashed", "dotted"):
            with self.subTest(style=style):
                d = Diagram("Framing")
                d.add(Resource("Compute", border=style))
                svg, report = d.render()
                self.assertEqual(report.problems, [])
                self.assertNotIn("States shown:", d.describe())
                self.assertEqual("stroke-dasharray" in svg, style in ("dashed", "dotted"))

    def test_lane_turn_does_not_cross_resource_labels(self):
        d = load(str(ROOT / "templates/aws_etl.json"))
        d.render()
        line = next(line for line in d.links if line.label == "after extraction")
        node = line.a
        # Once below the icon, the outgoing lane must already be outside the whole label box.
        image_bottom = node.y + d.theme.pad + node.block.icon_size
        self.assertFalse(any(node.x < x < node.right and image_bottom < y < node.bottom
                             for x, y in line.samples))


class EnclosureTests(unittest.TestCase):
    def test_panel_container_measures_lid_and_heading_and_embeds_nested_sections(self):
        for lid in (0, 14):
            with self.subTest(lid=lid):
                d = Diagram("Containers", theme=Theme(lid=lid))
                a = d.add(Resource("Bucket"))
                inner = d.enclosure("An intentionally long resource section heading", [a],
                                    style="panel", tone="accent", sub="A measured subtitle", border="solid")
                outer = d.enclosure("Cloud account", [inner], style="panel", border="solid", fill="#FFF9E8")
                svg, report = d.render()
                self.assertEqual(report.problems, [])
                x, y, w, h = d._group_title_box(inner)
                self.assertEqual(h, lid + d.theme.header_h_sub)
                self.assertLessEqual(y + h + inner.pad, a.y)
                self.assertLessEqual(x + w, outer.rect[0] + outer.rect[2])
                self.assertLess(outer.rect[1], inner.rect[1])
                self.assertIn("A measured subtitle", d.describe())
                self.assertEqual(shade(d.theme.tone("accent"), 0.16) in svg, bool(lid))
                self.assertNotIn("States shown:", d.describe())

    def test_panel_container_retains_selectable_border_and_readable_dark_header(self):
        for border, dash in (("solid", None), ("dashed", "5 3.5"), ("dotted", "0.1 3.4")):
            with self.subTest(border=border):
                d = Diagram("Frame")
                a = d.add(Resource("Compute"))
                d.enclosure("Dark section", [a], style="panel", tone="dark", border=border)
                svg, report = d.render()
                self.assertEqual(report.problems, [])
                tree = ET.fromstring(svg)
                heading = next(p for p in tree.findall(".//s:path", NS)
                               if p.find("s:title", NS) is not None and p.find("s:title", NS).text == "Dark section")
                self.assertEqual(heading.get("fill"), d.theme.header_text_on_dark)
                styles = {r.get("stroke-dasharray") for r in tree.findall(".//s:rect", NS)}
                self.assertIn(dash, styles)

    def test_nested_enclosures_render_outer_first_and_contain_titles(self):
        d = Diagram("Nested")
        a = d.add(Resource("Extractor"))
        b = d.right_of(a, Resource("Database"), gap=160)
        inner = d.enclosure("Private subnet", [a], border="dotted", fill="#F1F2F3")
        outer = d.enclosure("AWS account", [inner, b], border="dashed", fill="#FAFBFC")
        svg, report = d.render()
        self.assertEqual(report.problems, [])
        self.assertLess(svg.index('fill="#FAFBFC"'), svg.index('fill="#F1F2F3"'))
        self.assertLess(outer.rect[0], inner.rect[0])
        self.assertLess(outer.rect[1], inner.rect[1])
        self.assertIn("stroke-dasharray=\"0.1 3.4\"", svg)
        self.assertIn("stroke-dasharray=\"5 3.5\"", svg)
        self.assertIn("Private subnet", d.describe())

    def test_long_enclosure_title_fits_and_solid_has_no_dash(self):
        d = Diagram("Enclosure")
        node = d.add(Resource("S3"))
        group = d.enclosure("A deliberately long descriptive cloud account boundary", [node], border="solid")
        svg, report = d.render()
        self.assertEqual(report.problems, [])
        bx, _, bw, _ = d._group_title_box(group)
        self.assertLessEqual(bx + bw, group.rect[0] + group.rect[2])
        self.assertNotIn("stroke-dasharray", svg)

    def test_empty_foreign_and_invalid_members_are_rejected(self):
        d = Diagram("Validation")
        foreign = Diagram("Other").add(Resource("Foreign"))
        for members, kwargs in (([], {}), ([foreign], {}), (["not placed"], {})):
            with self.subTest(members=members), self.assertRaises(ValueError):
                d.enclosure("Account", members, **kwargs)
        node = d.add(Resource("Node"))
        for kwargs in ({"border": "dahsed"}, {"pad": -1}, {"pad": float("inf")},
                       {"style": "pannel"}, {"tone": "missing-tone"}):
            with self.subTest(kwargs=kwargs), self.assertRaises(ValueError):
                d.enclosure("Account", [node], **kwargs)

    def test_enclosure_does_not_hide_unrelated_overlap(self):
        d = Diagram("Overlap")
        a = d.add(Resource("A"))
        d.add(Resource("Unrelated"), x=a.right + 18)
        d.enclosure("Boundary", [a], pad=50)
        self.assertTrue(any("overlaps 'Unrelated'" in p for p in d.render()[1].problems))

    def test_dashed_control_link_is_independent_of_planned_state(self):
        d = Diagram("Control", theme=Theme(link_labels={"control": "Orchestration"}))
        a = d.add(Resource("MWAA"))
        b = d.right_of(a, Resource("Lambda"))
        d.link(a, b, kind="control", dash="dashed")
        svg, report = d.render()
        self.assertEqual(report.problems, [])
        self.assertIn('stroke-dasharray="5 3.5"', svg)
        self.assertIn("Orchestration".lower(), d.describe())
        self.assertNotIn("planned", d.describe().lower())
        self.assertIn("Dashed lines: orchestration", d.describe())
        with self.assertRaisesRegex(ValueError, "dash"):
            d.link(a, b, kind="control", dash="dahsed")

    def test_link_dash_is_supported_in_json_with_named_errors(self):
        spec = {"title": "Dash", "columns": [[{"id": "a", "title": "A"}], [{"id": "b", "title": "B"}]],
                "links": [{"from": "a", "to": "b", "dash": "dashed"}]}
        svg, report = from_spec(spec).render()
        self.assertEqual(report.problems, [])
        self.assertIn('stroke-dasharray="5 3.5"', svg)
        spec["links"][0]["dash"] = "typo"
        with self.assertRaisesRegex(SpecError, r"links\[0\].*dash"):
            from_spec(spec)

    def test_mixed_link_styles_have_distinct_legend_entries(self):
        d = Diagram("Mixed", theme=Theme(link_labels={"flow": "Data"}))
        a = d.add(Resource("A"))
        b = d.right_of(a, Resource("B"))
        c = d.right_of(b, Resource("C"))
        d.link(a, b)
        d.link(b, c, dash="dashed")
        svg, report = d.render()
        self.assertEqual(report.problems, [])
        self.assertIn("<title>Data (solid)</title>", svg)
        self.assertIn("<title>Data (dashed)</title>", svg)

    def test_transparent_nested_enclosure_retains_child_content(self):
        d = Diagram("Transparent")
        a = d.add(Resource("Bucket"))
        inner = d.enclosure("Storage", [a], fill="#EFEFEF")
        d.enclosure("Account", [inner], fill="none")
        svg, report = d.render()
        self.assertEqual(report.problems, [])
        self.assertIn('fill="none" stroke=', svg)
        self.assertIn('fill="#EFEFEF"', svg)


class TemplateTests(unittest.TestCase):
    def test_etl_sections_have_depth_and_links_avoid_their_headers(self):
        d = load(str(ROOT / "templates/aws_etl.json"))
        d.render()
        self.assertEqual(len(d.groups), 7)
        self.assertTrue(all(g.style == "panel" for g in d.groups))
        self.assertNotEqual(d.theme.paper, "#FFFFFF")
        for line in d.links:
            for g in d.groups:
                x, y, w, h = d._group_title_box(g)
                self.assertFalse(any(x < sx < x + w and y < sy < y + h for sx, sy in line.samples),
                                 f"{line.a.name} -> {line.b.name} crosses {g.label}'s header")

    def test_templates_render_offline_deterministically_and_match_previews(self):
        for path in sorted((ROOT / "templates").glob("*.json")):
            with self.subTest(template=path.name), patch("socket.socket", side_effect=AssertionError("network used")):
                d = load(str(path))
                svg, report = d.render()
                self.assertEqual(report.problems, [])
                self.assertGreaterEqual(report.min_text, 10)
                self.assertEqual(svg, load(str(path)).render()[0])
                self.assertEqual(svg.encode("utf-8"), (ROOT / "docs/images" / (path.stem + ".svg")).read_bytes())
                tree = ET.fromstring(svg)
                images = tree.findall(".//s:image", NS)
                self.assertTrue(images)
                self.assertTrue(all(x.get("href").startswith("data:") for x in images))

    def test_all_etl_border_options_and_required_resource_connectivity(self):
        d = load(str(ROOT / "templates/aws_etl.json"))
        self.assertEqual({p.name for p in d.placed},
                         {"mwaa", "api", "saas", "database", "raw", "glue", "curated", "catalog", "athena", "dashboards"})
        edges = {(line.a.name, line.b.name) for line in d.links}
        for key in ("api", "saas", "database"):
            self.assertIn(("mwaa", key), edges)
            self.assertIn((key, "raw"), edges)
        for border in ("solid", "dashed", "dotted"):
            with self.subTest(border=border):
                for group in d.groups:
                    group.border = border
                self.assertEqual(d.render()[1].problems, [])


class TemplateStandardTests(unittest.TestCase):
    """The rules in templates/README.md, "The standard"."""

    def test_every_template_file_is_in_the_table_and_every_row_has_a_file(self):
        from open_diagram import catalog
        files = sorted(p.name for p in (ROOT / "templates").iterdir() if p.suffix in (".json", ".mmd"))
        rows = [name for name, _ in catalog.table(str(ROOT / "templates/README.md"))]
        self.assertEqual(sorted(rows), files)
        self.assertEqual([p.name for p in (ROOT / "templates").iterdir() if p.suffix == ".py"], [])
        for name, purpose in catalog.table(str(ROOT / "templates/README.md")):
            with self.subTest(template=name):
                self.assertRegex(name, r"^[a-z0-9]+(_[a-z0-9]+)*\.(json|mmd)$")
                self.assertTrue(purpose)

    def test_each_template_fits_one_screen_names_its_services_and_has_a_png(self):
        for path in sorted((ROOT / "templates").glob("*.json")):
            with self.subTest(template=path.name):
                d = load(str(path))
                _, report = d.render()
                self.assertLessEqual(report.width, 2000)
                self.assertLessEqual(report.height, 1400)
                for placed in d.placed:
                    if isinstance(placed.block, Resource) and placed.block.icon:
                        self.assertTrue(placed.block.sub, f"{placed.block.label} needs its service name in sub")
                self.assertTrue((ROOT / "docs/images" / (path.stem + ".png")).exists())

    def test_every_icon_folder_is_credited(self):
        credits = (ROOT / "templates/assets/README.md").read_text()
        for folder in sorted(p for p in (ROOT / "templates/assets").iterdir() if p.is_dir()):
            with self.subTest(folder=folder.name):
                self.assertIn(f"## {folder.name}\n", credits)

    def test_templates_ship_in_the_source_checkout_and_package_layout(self):
        from open_diagram import catalog
        self.assertEqual(Path(catalog.template_dir()).resolve(), (ROOT / "templates").resolve())
        self.assertIn("azure_web_app", catalog.names())
        self.assertTrue(catalog.read("aws_etl").lstrip().startswith("{"))
        with self.assertRaisesRegex(ValueError, "no template named 'nope'; templates: aws_etl"):
            catalog.path("nope")


class JsonResourceTests(unittest.TestCase):
    SPEC = {
        "title": "Resources from JSON",
        "resources": [
            {"id": "fn", "label": "Worker", "sub": "AWS Lambda", "icon": "aws/lambda", "x": 0, "y": 0},
            {"id": "db", "label": "Records", "sub": "Amazon RDS", "icon": "aws/rds", "x": 300, "y": 0}],
        "enclosures": [
            {"id": "data", "label": "Data subnet", "members": ["db"], "border": "dotted"},
            {"id": "net", "label": "Network", "members": ["fn", "data"], "border": "solid", "pad": 32}],
        "links": [{"from": "fn", "to": "db", "label": "SQL", "dash": "dashed"}]}

    def spec(self, **changes):
        import copy
        s = copy.deepcopy(self.SPEC)
        s.update(changes)
        return s

    def test_resources_and_nested_enclosures_draw_cleanly(self):
        d = from_spec(self.spec())
        svg, report = d.render()
        self.assertEqual(report.problems, [])
        self.assertEqual([g.label for g in d.groups], ["Data subnet", "Network"])
        self.assertEqual(len(ET.fromstring(svg).findall(".//s:image", NS)), 2)
        self.assertIn("Worker (AWS Lambda)", d.describe())

    def test_mistakes_name_the_place(self):
        cases = [
            (self.spec(enclosures=[{"label": "Net", "members": ["fn", "data"]}, {"id": "data", "label": "D", "members": ["db"]}]),
             r"enclosures\[0\]\.members: enclosure 'data' is listed after this one"),
            (self.spec(enclosures=[{"label": "Net", "members": ["ghost"]}]), r"enclosures\[0\]\.members: no resource.*'ghost'"),
            (self.spec(resources=[{"id": "a", "label": "A", "x": 0, "y": 0}, {"id": "a", "label": "B", "x": 200, "y": 0}],
                       enclosures=[], links=[]), r"resources\[1\]\.id: 'a' is already used"),
            (self.spec(resources=[{"label": "A", "x": 0}], enclosures=[], links=[]), r"resources\[0\]: missing required key 'y'"),
            (self.spec(columns=[[{"title": "P"}]]), r"resources: .*not with 'columns'"),
            (self.spec(enclosures=[{"label": "Net", "members": ["fn"], "border": "wavy"}]), r"enclosures\[0\]: .*border"),
            (self.spec(resources=[{"label": "A", "icon": "aws/nope", "x": 0, "y": 0}], enclosures=[], links=[]),
             r"resources\[0\]\.icon: no icon named 'aws/nope'; icons available: .*aws/lambda"),
        ]
        for spec, pattern in cases:
            with self.subTest(pattern=pattern), self.assertRaisesRegex(SpecError, pattern):
                from_spec(spec)

    def test_icons_never_come_from_outside_the_icon_folders(self):
        with tempfile.TemporaryDirectory() as tmp:
            secret = Path(tmp) / "secret.svg"
            secret.write_text('<svg xmlns="http://www.w3.org/2000/svg"/>')
            folder = Path(tmp) / "icons"
            (folder / "own").mkdir(parents=True)
            (folder / "own" / "box.svg").write_text('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 8 8"/>')
            for name in (str(secret), "../secret", "own/../../secret", "C:/secret", "own//box"):
                spec = self.spec(resources=[{"label": "A", "icon": name, "x": 0, "y": 0}], enclosures=[], links=[])
                with self.subTest(name=name), self.assertRaises(SpecError):
                    from_spec(spec, icons=[str(folder)])
            spec = self.spec(resources=[{"label": "A", "sub": "Mine", "icon": "own/box", "x": 0, "y": 0}],
                             enclosures=[], links=[])
            self.assertEqual(from_spec(spec, icons=[str(folder)]).render()[1].problems, [])

    def test_icons_next_to_a_json_file_and_on_tiles(self):
        import json
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / "mine.svg").write_text('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 8 8"/>')
            path = Path(tmp) / "d.json"
            path.write_text(json.dumps({"title": "Tiles", "columns": [[{"title": "Store",
                                        "tiles": [{"label": "Bucket", "icon": "aws/s3"}, {"label": "Mine", "icon": "mine"}]}]]}))
            svg, report = load(str(path)).render()
            self.assertEqual(report.problems, [])
            self.assertEqual(len(ET.fromstring(svg).findall(".//s:image", NS)), 2)


class NewCommandTests(unittest.TestCase):
    def test_new_lists_writes_and_refuses_to_overwrite(self):
        import contextlib
        import io
        import os
        from open_diagram.cli import main
        with tempfile.TemporaryDirectory() as tmp:
            here = os.getcwd()
            os.chdir(tmp)
            try:
                out = io.StringIO()
                with contextlib.redirect_stdout(out):
                    self.assertEqual(main(["new"]), 0)
                self.assertIn("azure_web_app", out.getvalue())
                with contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(main(["new", "azure_web_app"]), 0)
                    self.assertEqual(main(["check", "azure_web_app.json"]), 0)
                with contextlib.redirect_stderr(io.StringIO()) as err:
                    self.assertEqual(main(["new", "azure_web_app"]), 2)
                self.assertIn("already exists", err.getvalue())
            finally:
                os.chdir(here)


if __name__ == "__main__":
    unittest.main()
