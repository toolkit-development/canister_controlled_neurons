# canister_controlled_neuron

## Release Process

Releases should be done when merging a branch into master. Most of the release process is automatic. However there are a couple of thing that need to be done before the release. These are as follows:

1. Update the version number in the Cargo.toml file (Make sure to follow semantic versioning (MAJOR.MINOR.PATCH) when creating new versions)
2. Update the `CHANGELOG.md` by adding a new verison and details of all the changes you have made.
3. Create and push a new git tag:

   ```bash
   git tag -a v<version_number> -m "Release v<version_number>"
   git push origin v<version_number>
   ```

4. The release pipeline will automatically trigger when the tag is pushed

The release pipeline will:

- Extract the version number from Cargo.toml
- Verify that the version exists in CHANGELOG.md
- Create a GitHub release with the WASM file
- Use the CHANGELOG.md entry as the release notes