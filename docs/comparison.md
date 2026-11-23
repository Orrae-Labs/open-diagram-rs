# How open-diagram compares

How open-diagram compares with Mermaid, D2, draw.io, Excalidraw and archify, checked November 23, 2026. Every fact comes from the tool's own docs, repository or pricing page, listed under Sources. These tools change often. If a fact here is out of date, open an issue.

## The short answer

- **Mermaid** fits a diagram that has to show up inside GitHub, GitLab, Notion or Obsidian with nothing to install. It also fits diagram types open-diagram does not draw, such as sequence, Gantt, class and entity-relationship diagrams.
- **draw.io** or **Excalidraw** fits when you want to move shapes by hand, or edit at the same time as other people.
- **D2** fits when you need PDF, PowerPoint or GIF output.
- **archify** fits when you want an interactive web page whose boxes link to source code.
- **open-diagram** fits a dense, static architecture picture (SVG and PNG). An agent or a person writes it in Python, JSON or Mermaid. It checks its own layout on your computer, and a GitHub Action can check it again on every pull request.

## Side by side

MCP is the Model Context Protocol: the way AI apps such as Claude, Cursor and VS Code call an outside tool.

| Tool | How you describe a diagram | Who places the boxes | Reports layout problems | Output | MCP server | License | Price |
| --- | --- | --- | --- | --- | --- | --- | --- |
| open-diagram | Python, JSON, or a Mermaid flowchart | open-diagram, from measured sizes: in columns, in a ring, or at positions you give | Yes. Each problem is a sentence that names the parts, from Python, the command, the MCP server and the GitHub Action | SVG, PNG, and a plain-words description | Yes, runs on your computer | MIT | Free |
| Mermaid | Text. The paid mermaid.ai app adds a visual editor for flowcharts | A layout engine: ELK by default since version 12, or dagre and others | Syntax errors only, with the line and column. Its layout tests are for people who build layout engines | SVG. PNG and PDF through its command-line tool | Official, hosted at mcp.mermaid.ai; needs a free Mermaid account | MIT. The mermaid.ai app is not open source | Free. mermaid.ai: Basic free (3 diagrams), Plus $10 and Premium $20 per user per month, billed yearly |
| D2 | Text | A layout engine: dagre by default, ELK or TALA. TALA lets you lock positions | No. A linter is planned | SVG, PNG, PDF, PowerPoint, GIF, ASCII text | None, by choice: "No MCP, no API" | MPL-2.0, TALA included since September 2026 | Free |
| draw.io | A drag-and-drop editor. Files are XML. Imports Mermaid, PlantUML and CSV | You, with optional automatic layouts | The editor: none found in the docs. Its official MCP viewer lists labels too big for their shape, overlaps and lines through shapes, with a "Fix N issues" button | .drawio, PNG, SVG, PDF, HTML, JPEG, WebP, GIF and more | Official, hosted at mcp.draw.io | Apache-2.0. Desktop app GPL-3.0 | Free. The Confluence and Jira apps are paid |
| Excalidraw | A hand-drawn-style canvas. Files are JSON. Imports Mermaid | You | The editor: none found in the docs. The paid Excalidraw+ MCP server reports text overflowing its box, arrows through shapes and overlaps | PNG, SVG, .excalidraw JSON. PDF and PowerPoint on Excalidraw+ | Official: an open-source one at mcp.excalidraw.com, and one in Excalidraw+ | MIT | Free. Excalidraw+ $6 per user per month paid yearly, $7 paid monthly |
| archify | An agent writes JSON from your words, a plan or a codebase | archify places architecture diagrams; for other types, the agent gives positions | Yes. Checks return rule codes and measurements, and a diagram that fails is not delivered | One interactive HTML file plus its JSON. Image and video exports from the browser | None. It is an agent skill | Apache-2.0 | Free |

## What each one does better than open-diagram

- **Mermaid** shows up inside GitHub, GitLab, Notion, Obsidian and more with nothing to install. Its docs list 32 diagram types.
- **D2** exports PDF, PowerPoint, GIF and ASCII text, and comes with three layout engines.
- **draw.io** is a full drag-and-drop editor with more than 10,000 shapes. Several people can edit at once in Google Drive, OneDrive and Confluence Cloud.
- **Excalidraw** is a hand-drawn-style whiteboard with free real-time collaboration, encrypted end to end.
- **archify** makes an interactive page. You click a box to focus on it, trace what depends on what, and open the source lines behind a box.

## What open-diagram does not do

- It has no visual editor and no real-time collaboration. You change the words and draw again.
- GitHub does not draw it inside Markdown. You commit the SVG or PNG.
- It draws architecture and systems maps, pipelines, layers, rings, concept maps, business processes, cloud resources and Mermaid flowcharts. It does not draw sequence, Gantt, class or entity-relationship diagrams.
- Icons work from Python only. JSON, Mermaid and the MCP server do not take them.
- PNG output needs Chrome, Chromium or Edge on the computer. SVG output needs nothing extra.

## What open-diagram does

- It measures every label from the font before it places anything, so each block's size is known, never guessed.
- It reports each layout problem as a sentence that names the parts, for example `'Storage' and 'Recommendations' are only 6 px apart (minimum 16)`. An agent can fix the problem without looking at the picture.
- The same script always draws the same SVG, byte for byte. The GitHub Action checks every diagram in a pull request.
- Everything runs on your computer. You need no account, and the MCP server writes files only inside one output folder.
- It is free under the MIT license.

## Sources

Read on November 23, 2026.

- open-diagram: [README](https://github.com/lukendeasy/open-diagram#readme) and [PyPI](https://pypi.org/project/open-diagram/)
- Mermaid: [introduction](https://mermaid.js.org/intro/), [layouts](https://mermaid.js.org/config/layouts.html), [layout makers' guide](https://mermaid.js.org/community/layout-makers-guide.html), [API skill guide](https://mermaid.ai/docs/guides/mermaid-api-skill), [command-line tool](https://github.com/mermaid-js/mermaid-cli), [visual editor](https://mermaid.ai/docs/build-and-edit/use-the-visual-editor), [MCP server](https://mermaid.ai/docs/ai/mcp-server), [integrations](https://mermaid.js.org/ecosystem/integrations-community.html), [license](https://github.com/mermaid-js/mermaid/blob/develop/LICENSE), [pricing](https://mermaid.ai/pricing)
- D2: [introduction](https://d2lang.com/tour/intro/), [layouts](https://d2lang.com/tour/layouts/), [exports](https://d2lang.com/tour/exports/), [future plans](https://d2lang.com/tour/future/), [D2 as a non-profit](https://d2lang.com/blog/d2-non-profit/), [TALA is open source](https://d2lang.com/blog/tala-is-open-source/), [license](https://github.com/d2lang/d2/blob/master/LICENSE.txt), [sponsor page](https://d2lang.com/sponsor/)
- draw.io: [repository](https://github.com/jgraph/drawio), [file formats](https://www.drawio.com/docs/manual/editor/save-file-formats/), [Mermaid import](https://www.drawio.com/docs/manual/insert/insert-mermaid/), [CSV import](https://www.drawio.com/docs/manual/insert/insert-from-csv/), [layouts](https://www.drawio.com/docs/manual/layouts/), [export](https://www.drawio.com/docs/manual/export/export-diagram/), [collaboration](https://www.drawio.com/docs/manual/collaboration/concurrent-editing/), [MCP server](https://github.com/jgraph/drawio-mcp) and [its viewer notes](https://github.com/jgraph/drawio-mcp/blob/main/mcp-app-server/AGENTS.md), [Atlassian licensing](https://www.drawio.com/docs/integrations/atlassian/licensing/)
- Excalidraw: [repository](https://github.com/excalidraw/excalidraw), [license](https://github.com/excalidraw/excalidraw/blob/master/LICENSE), [MCP server](https://github.com/excalidraw/excalidraw-mcp), [Excalidraw+ MCP](https://plus.excalidraw.com/docs/mcp), [AI features](https://plus.excalidraw.com/ai), [pricing](https://plus.excalidraw.com/pricing)
- archify: [repository](https://github.com/tt-a1i/archify), [authoring contract](https://github.com/tt-a1i/archify/blob/main/archify/references/authoring-contract.md), [site](https://archify.si), [full description](https://archify.si/llms-full.txt), [license](https://github.com/tt-a1i/archify/blob/main/LICENSE)
