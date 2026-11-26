# Releasing

The three crates are published to crates.io with [trusted publishing](https://crates.io/docs/trusted-publishing). The publish workflow trades a GitHub OIDC token for a crates.io token that expires after about 30 minutes, so the repository holds no crates.io API token.

## Pick the version

The version numbers follow open-diagram, the Python project this one ports. Our `X.Y.Z` has the features and fixes of open-diagram `X.Y.Z`, so a release waits for open-diagram to tag `vX.Y.Z`. Before releasing:

1. List the upstream commits since the last release: `git log --oneline vA.B.C..vX.Y.Z` in an open-diagram checkout, where `vA.B.C` is the previous upstream tag.
2. Check that each one is ported, or named as not ported in a pull request.

Changes made only in this repository wait in `Unreleased` and ship with the next matching version.

## Cut a release

1. Set the same `version` in the three package manifests (`crates/open-diagram`, `crates/open-diagram-cli`, `crates/open-diagram-mcp`) and in `server.json`.
2. Push a tag `vX.Y.Z` with that version.

The tag starts two workflows.

[`crate-provenance.yml`](../.github/workflows/crate-provenance.yml) packages each crate with `cargo package --locked`. That job's token cannot request OIDC. A second job attests the resulting `.crate` files with `actions/attest` and does not run Cargo.

[`publish.yml`](../.github/workflows/publish.yml) then:

1. Verifies the attestation for the tag.
2. Repacks each crate with `cargo package --locked --no-verify` and stops if the digest differs from the attested one. Cargo cannot upload an existing `.crate`, so the repack is how the workflow proves it is uploading the attested bytes. Neither step compiles anything.
3. Uploads with `cargo publish --locked --no-verify`. The crates.io token exists only during this step.
4. Checks that the checksum in the crates.io index equals the attested digest.

The library goes first. The command and the MCP server follow once the library's version is in the index. After `open-diagram-rs-mcp` is on crates.io, the same workflow logs in to the MCP Registry with GitHub OIDC and publishes `server.json` under `io.github.Orrae-Labs/open-diagram-rs`.

The `release` environment accepts only `v*` tags and needs its reviewer's approval. That approval is what stops an edited workflow from publishing.

## Set up a new crate name

crates.io only allows trusted publishing for a name that already exists. For each new name:

1. A crates.io owner publishes it once from a trusted machine, then revokes the API token they used.
2. Add a trusted publisher with owner `Orrae-Labs`, repository `open-diagram-rs`, workflow `publish.yml` and environment `release`.
3. Turn on "Require trusted publishing".

Every later version goes through the workflow.

## Verify a downloaded crate

```bash
gh attestation verify open-diagram-rs-0.6.2.crate --repo Orrae-Labs/open-diagram-rs --signer-workflow Orrae-Labs/open-diagram-rs/.github/workflows/crate-provenance.yml --source-ref refs/tags/v0.6.2
```
