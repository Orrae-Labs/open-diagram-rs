# Template artwork

These files are unmodified cloud provider architecture icons, cached so the templates and their
tests work offline. They are embedded into the output SVG/PNG. The project MIT license does not
relicense any provider's artwork or trademarks. An icon's name in a template is its path here
without the extension: `aws/s3` is `aws/s3.svg`.

## aws

Amazon Web Services owns the artwork. Source and guidance:
[AWS architecture icons](https://aws.amazon.com/architecture/icons/).

The SVG filenames match service keys in [`cloud-icons/aws.json`](../../../../cloud-icons/aws.json).
Those links pin theSVG catalog's AWS colored service family at
[`551229c188b4ab6e2d8b5763b9633ec91206d04c`](https://github.com/glincker/thesvg/tree/551229c188b4ab6e2d8b5763b9633ec91206d04c),
catalogued as [CC BY-ND 2.0](https://creativecommons.org/licenses/by-nd/2.0/).
The catalog identifies the family as `2026-Q1`. `glue.svg` represents the AWS Glue service;
the Data Catalog node explicitly names its Glue subsystem.

QuickSight is absent from that snapshot. Its unmodified PNG comes from AWS Labs'
[2025 icon release](https://github.com/awslabs/aws-icons-for-plantuml/blob/11cbe08742e1d548f83612beba1d45de4358ac3f/dist/Analytics/QuickSight.png),
which uses the same colored AWS architecture style. This is the QuickSight BI artwork,
not a substituted Quick Suite logo. The source repository's license is included in
[`aws/LICENSE-QuickSight.txt`](aws/LICENSE-QuickSight.txt).

## azure

Microsoft owns the artwork. Source and terms:
[Azure architecture icons](https://learn.microsoft.com/en-us/azure/architecture/icons/), which permit
the icons in architecture diagrams and ask that they are not cropped, flipped, rotated or reshaped,
and that the product name appears next to the icon (each template's `sub` line).

Files: `front-door`, `app-service`, `sql-database`, `cache-for-redis`, `key-vault`, `block-blob`.
Each matches a key in [`cloud-icons/azure.json`](../../../../cloud-icons/azure.json) and was downloaded,
unchanged, from the link there: theSVG catalog at
[`551229c188b4ab6e2d8b5763b9633ec91206d04c`](https://github.com/glincker/thesvg/tree/551229c188b4ab6e2d8b5763b9633ec91206d04c),
collection `azure`, version `2026-Q1`, which lists their license as MIT.

## gcp

Google owns the artwork. Source: [Google Cloud icons](https://cloud.google.com/icons).

Files: `pub-sub`, `cloud-run`, `cloud-functions`, `cloud-storage`, `bigquery`, `vertex-ai`.
Each matches a key in [`cloud-icons/gcp.json`](../../../../cloud-icons/gcp.json) and was downloaded,
unchanged, from the link there: theSVG catalog at
[`551229c188b4ab6e2d8b5763b9633ec91206d04c`](https://github.com/glincker/thesvg/tree/551229c188b4ab6e2d8b5763b9633ec91206d04c),
collection `gcp`, version `2026-Q1`, which lists their license as Apache-2.0.

Preserve these credits, source links and applicable license notices when distributing
the example images or cached assets.
