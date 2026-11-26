---
name: cloud-icons
description: Select consistent AWS, Azure, and Google Cloud service icons from pinned SVG links and embed them in open-diagram-rs architecture, deployment, network, and data-flow diagrams.
license: MIT
---

# Cloud icons

The mappings live separately from this skill in the repository's top-level `cloud-icons/` directory:

| Provider | Mapping file |
| --- | --- |
| AWS | [cloud-icons/aws.json](../../cloud-icons/aws.json) |
| Azure | [cloud-icons/azure.json](../../cloud-icons/azure.json) |
| GCP | [cloud-icons/gcp.json](../../cloud-icons/gcp.json) |

Each file maps service keys directly to SVG URLs, without a provider wrapper. For example, the `s3` entry in
`aws.json` is the Amazon S3 icon. Read the selected provider's keys before choosing services; do not construct URLs
from names.

## Choose icons that explain the diagram

| Diagram | Useful choices |
| --- | --- |
| Application or deployment | Compute, containers, registry, database, storage |
| Network or security | VPC / virtual network, load balancer, gateway, DNS, identity, key management |
| Events or data pipeline | Queue, topic, event bus, data processing, database, analytics |
| AI application | Bedrock, Azure OpenAI, or Vertex AI, plus the services actually in the request |
| Multi-cloud | Select each service from its own provider group; group panels by cloud and keep labels explicit |

Use a service icon when the request names that service. For abstract concepts such as "backend", unspecified
clouds, or services absent from the mapping, use labeled resources or tiles without icons. Do not infer a provider or replace a
missing service with a different product's logo. The Azure `block-blob` key is specifically a **Block Blob** resource
in Blob Storage; `event-grid` uses the Event Grid Topics symbol. Label those resources accordingly.

Keep one icon family per provider. Preserve colors, orientation and proportions, with the product name beside or below
each icon. For cloud topology or an ETL DAG, prefer a resource whose JSON sets `"icon": "aws/s3"`. That draws a 64 px standalone icon with labels
underneath, and every resource can connect independently. Start with the repository's
[cloud templates](../../crates/open-diagram/templates/README.md). Enclosures nest accounts, networks and logical groups.
Set the enclosure `border` to `solid`, `dashed`, or `dotted`. For dense panel summaries, a tile keeps its 16 px icon box.
Use spacing and labels to unify a multi-cloud picture.

## Name a built-in icon, or add a file

JSON names an icon as `"icon": "aws/s3"`. The command looks in the JSON file's own folder, then each `--icons` folder, then the built-in folder `crates/open-diagram/templates/assets/`. Absolute paths and `..` are refused. Mermaid does not take icons.

The built-in set is the icons the templates use. For a service that is only in `cloud-icons/<provider>.json`:

1. Read that key's URL. Do not invent a URL from the service name.
2. Download the file and confirm the root element is SVG.
3. Save it as `<provider>/<service>.svg` inside a folder you pass with `--icons`.
4. Use `"icon": "<provider>/<service>"` in the JSON.

The rendered SVG embeds the icon bytes, so the PNG and SVG can be shared without the icon host. If a download fails, tell the user which service is missing and keep the text label. A pinned URL fixes the selected file, not the host's availability.

Enclosure borders and `"dash": "dashed"` are visual styles. Do not set `"status": "planned"` or `"kind": "planned"` just to obtain dashes. Give orchestration its own theme line kind, such as `control`, so the key explains the connection.

After the icons are named, run `open-diagram-rs check` on the JSON file, fix every named layout problem, and inspect the PNG. Compare `open-diagram-rs describe` with the requested services and connections. Use the [open-diagram-rs skill](../open-diagram-rs/SKILL.md) for layout and review.

## Sources, family versions and attribution

The links use theSVG as a third-party host for provider architecture icons, pinned to
[commit 551229c188b4ab6e2d8b5763b9633ec91206d04c](https://github.com/glincker/thesvg/tree/551229c188b4ab6e2d8b5763b9633ec91206d04c).
Every selected file belongs to that provider's collection, uses its `default` variant, and has the catalog's
`2026-Q1` collection version. These labels describe the catalog snapshot, not the providers' latest releases.

| Provider | Selected family | Official source and guidance |
| --- | --- | --- |
| AWS | Colored service architecture icons, square backgrounds | [AWS architecture icons](https://aws.amazon.com/architecture/icons/) |
| Azure | Colored Azure architecture/portal service and resource icons | [Azure architecture icons and terms](https://learn.microsoft.com/en-us/azure/architecture/icons/) |
| GCP | Blue console/product icon family | [Google Cloud icon library](https://cloud.google.com/icons) |

Google also offers a refreshed 2025 core/category family. This mapping consistently uses the blue family available
in the pinned catalog; do not mix the two families within a GCP diagram. Some keys retain names from that snapshot,
such as `cloud-functions` and `cache-for-redis`; do not imply that the artwork represents a newer product edition.

Provider artwork and trademarks retain their own terms; the repository's MIT license does not relicense them.
The catalog identifies AWS assets as [CC BY-ND 2.0](https://creativecommons.org/licenses/by-nd/2.0/); credit Amazon Web
Services and link to the AWS source and license when distributing diagrams using them. Follow Microsoft's icon
terms for Azure and Google's applicable guidance for GCP. Preserve required notices and credit the icon owner and
source alongside shared SVG/PNG files. See also the catalog's
[legal notice](https://github.com/glincker/thesvg/blob/551229c188b4ab6e2d8b5763b9633ec91206d04c/LEGAL.md).

To refresh the mapping, use Exa to locate the official catalogs and Firecrawl to extract their download links and
guidance. Enumerate the candidate host's actual files and metadata, select one family per provider, and pin a full
commit SHA. Fetch every changed URL, confirm that it returns an SVG, and render samples before replacing links.
Update the family/version notes together with the mapping. No Firecrawl or Exa credentials are needed to use the
existing links or render downloaded assets.
