# Templates

Start from a finished cloud diagram, keep its shape, and replace its words.

Every template here is a JSON file, so the `open-diagram` command and any program that can write JSON can start from it.

| Template | Purpose | Preview |
| --- | --- | --- |
| [aws_etl.json](aws_etl.json) | Data ingestion on AWS in six numbered sections, from orchestration to dashboards | [PNG](../../../docs/images/aws_etl.png) |
| [aws_architecture.json](aws_architecture.json) | An AWS account, network and subnets around an API, a function, a database and storage | [PNG](../../../docs/images/aws_architecture.png) |
| [azure_web_app.json](azure_web_app.json) | A web app on Azure behind Front Door, with its database and cache inside a virtual network | [PNG](../../../docs/images/azure_web_app.png) |
| [gcp_event_pipeline.json](gcp_event_pipeline.json) | An event pipeline on Google Cloud from Pub/Sub to a warehouse and a forecast model | [PNG](../../../docs/images/gcp_event_pipeline.png) |

## Start from a template

```sh
open-diagram new                           # list the templates
open-diagram new azure_web_app             # write azure_web_app.json here
open-diagram draw azure_web_app.json       # draw it; the icons come with the package
```

`open-diagram new NAME` writes that JSON into the current directory. Edit it, then run `check` and `draw`.

Change the `label` of each resource to your own words and keep its `sub`, which names the cloud service under its
icon. Move a resource by changing its `x` and `y`. Run `open-diagram check` after each change: it names every
overlap, crossing line and cramped label.

## The standard

Every template meets these rules. `cargo test` checks each rule marked "tested", so a pull request that breaks one fails.

1. **One JSON file per template** in this folder, named in lowercase words joined by underscores. (tested)
2. **Listed in the table above**, with its purpose in plain words and a link to its preview. The website's Templates
   page reads this table. (tested)
3. **No layout problems**, and no text smaller than 10 px. (tested)
4. **Fits on one screen or slide**: no wider than 2,000 px and no taller than 1,400 px. A picture larger than that has
   to be shrunk to be seen whole, and its text stops being readable. (tested)
5. **The same picture every time**, matching the committed preview in `docs/images/<name>.svg` and `.png`. (tested)
6. **Works offline.** Every icon is a file in `assets/`; drawing a template never uses the network. (tested)
7. **The service name sits under each icon**, in the resource's `sub`. Azure's icon terms ask for the product name
   next to the icon, and it tells a reader what each picture is. (tested)
8. **Icons are the providers' own files, unchanged**, each folder with its source, version and license in
   [assets/README.md](assets/README.md). (tested: every icon folder has a section there)
9. **A made-up workload.** Real cloud service names are fine; real company names, people and data are not. Say so in
   a note or a subtitle.

Colors, borders and the style of each box are up to the template.

## What a template is made of

A template uses the `resources` layout: each resource is placed where you say, with `x` and `y` in pixels.

```json
{
  "title": "Web app on Azure",
  "resources": [
    {"id": "web", "label": "Web app", "sub": "Azure App Service", "icon": "azure/app-service", "x": 0, "y": 0},
    {"id": "db", "label": "Orders database", "sub": "Azure SQL Database", "icon": "azure/sql-database", "x": 320, "y": 0}
  ],
  "enclosures": [
    {"id": "data", "label": "Data subnet", "members": ["db"], "border": "dotted"},
    {"id": "vnet", "label": "Virtual network", "members": ["web", "data"], "border": "dashed"}
  ],
  "links": [{"from": "web", "to": "db", "label": "SQL"}]
}
```

- **Resource:** `label` (required), `x` and `y` (required), `id`, `sub`, `icon`, `icon_size` (default 64),
  `max_width` (default 180), `border` (`solid`, `dashed` or `dotted`).
- **Enclosure:** a box around resources or other enclosures. `label` and `members` (required), `id`, `sub`, `pad`
  (default 24), `border` (default `dashed`), `fill` (`none` for see-through), `stroke`, `style` (`outline`, or
  `panel` for the sloped lid and filled header), `tone` (a header color for `panel`). List enclosures from the inside
  out: an enclosure must come before any enclosure that holds it.
- **Icon:** a name inside an icon folder, such as `aws/s3`. The built-in folder is `assets/`. A JSON file can also
  use icons in its own folder, and `--icons FOLDER` adds another folder. Full paths and `..` are refused, so a
  diagram description can never read other files on the computer.
- **Links** between resources work as they do between panels. Use `kind` with `dash` for a second kind of line, for
  example `{"kind": "control", "dash": "dashed"}`, and name it in `theme.link_labels`.

Resources can share a picture with panels: give both `x` and `y` and use the `panels` layout alongside `resources`.

## From Rust

`Resource` is a large icon with its label and service name under it. `Diagram::enclosure` draws a box around resources or other enclosures. Load a template with `open_diagram::load_spec`. The lid, header, subtitle and nested enclosures all count toward the measured size, so the layout checks see the whole box. Border style says nothing about whether a resource is planned or live. `Diagram::group` keeps its shaded look.

## Add a template

1. Write `<name>.json` in this folder with a made-up workload. Put any new icons in `assets/<provider>/`, unchanged, and add their source and license to `assets/README.md`.
2. Run `open-diagram check crates/open-diagram/templates/<name>.json` until it reports 0 problems.
3. Draw the preview: `open-diagram draw crates/open-diagram/templates/<name>.json -o docs/images/<name>`. Look at the PNG.
4. Add a row to the table above, then run `cargo test --workspace`.

More provider icons, with links to each file, are listed in [cloud-icons/](../../../cloud-icons/); the
[cloud-icons skill](../../../skills/cloud-icons/SKILL.md) downloads them.
