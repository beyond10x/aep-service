//! The workspace selects one released AEP generation and one Entity Runtime generation.
//!
//! `AGENTS.md` § *Cross-repository changes* requires workspace dependencies to select one exact
//! AEP tag and one exact Entity Runtime tag, with no crate-local pin. A re-pin that moves some of
//! those declarations and leaves the rest, or that moves `Cargo.toml` and leaves `Cargo.lock` on
//! the previous generation, composes a boundary no release was verified as a whole. The gate reads
//! the declarations and the resolved lockfile here so that mixture fails a test instead of being
//! read off a diff.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The released AEP tag this service consumes. A re-pin moves this constant and `Cargo.toml`
/// together; the tests below refuse the state where only one of them moved.
const AEP_TAG: &str = "0.53.0";

/// The Entity Runtime tag AEP `AEP_TAG` selects. The service and the AEP backends exchange
/// `entity-core` types, so a second Entity Runtime generation in the graph is not a duplicate
/// build, it is two incompatible sets of the same types.
const ENTITY_RUNTIME_TAG: &str = "0.17.6";

/// The AEP repository, spelled as the manifest and the lockfile spell it.
const AEP_GIT: &str = "https://github.com/beyond10x/aep";

/// The Entity Runtime repository, spelled as the manifest and the lockfile spell it.
const ENTITY_RUNTIME_GIT: &str = "https://github.com/beyond10x/entity-runtime";

/// The AEP crates this workspace depends on directly.
const AEP_DEPENDENCIES: [&str; 6] = [
    "aep-backend-entity",
    "aep-backend-postgres",
    "aep-client",
    "aep-contract",
    "aep-domain",
    "aep-project",
];

/// The Entity Runtime crates this workspace depends on directly.
const ENTITY_RUNTIME_DEPENDENCIES: [&str; 4] = [
    "entity-core",
    "entity-postgres",
    "entity-query",
    "entity-store",
];

#[test]
fn workspace_declares_the_released_aep_tag_for_every_aep_dependency() {
    let declared = declared_pins(&read(&workspace_root().join("Cargo.toml")), AEP_GIT);

    assert_eq!(
        declared.keys().cloned().collect::<BTreeSet<_>>(),
        AEP_DEPENDENCIES
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<BTreeSet<_>>(),
        "the workspace declares exactly the AEP crates this test knows the generation of"
    );
    for (name, tag) in &declared {
        assert_eq!(
            tag, AEP_TAG,
            "workspace dependency {name} names AEP tag {tag}, not the selected {AEP_TAG}"
        );
    }
}

#[test]
fn workspace_declares_the_entity_runtime_generation_aep_selects() {
    let declared = declared_pins(
        &read(&workspace_root().join("Cargo.toml")),
        ENTITY_RUNTIME_GIT,
    );

    assert_eq!(
        declared.keys().cloned().collect::<BTreeSet<_>>(),
        ENTITY_RUNTIME_DEPENDENCIES
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<BTreeSet<_>>(),
        "the workspace declares exactly the Entity Runtime crates this test knows the generation of"
    );
    for (name, tag) in &declared {
        assert_eq!(
            tag, ENTITY_RUNTIME_TAG,
            "workspace dependency {name} names Entity Runtime tag {tag}, not the selected \
             {ENTITY_RUNTIME_TAG}"
        );
    }
}

#[test]
fn the_lockfile_resolves_every_pinned_crate_at_the_declared_generation() {
    let lock = read(&workspace_root().join("Cargo.lock"));

    let aep = locked_pins(&lock, AEP_GIT);
    for name in AEP_DEPENDENCIES {
        assert!(
            aep.contains_key(name),
            "the lockfile resolves {name} from {AEP_GIT}"
        );
    }
    for (name, resolved) in &aep {
        assert_eq!(
            resolved.tag, AEP_TAG,
            "the lockfile resolves {name} at AEP tag {}, not the selected {AEP_TAG}",
            resolved.tag
        );
        assert_eq!(
            resolved.version, AEP_TAG,
            "the lockfile resolves {name} at version {}, which is not the {AEP_TAG} release",
            resolved.version
        );
    }

    let entity_runtime = locked_pins(&lock, ENTITY_RUNTIME_GIT);
    for name in ENTITY_RUNTIME_DEPENDENCIES {
        assert!(
            entity_runtime.contains_key(name),
            "the lockfile resolves {name} from {ENTITY_RUNTIME_GIT}"
        );
    }
    for (name, resolved) in &entity_runtime {
        assert_eq!(
            resolved.tag, ENTITY_RUNTIME_TAG,
            "the lockfile resolves {name} at Entity Runtime tag {}, not the selected \
             {ENTITY_RUNTIME_TAG}",
            resolved.tag
        );
        assert_eq!(
            resolved.version, ENTITY_RUNTIME_TAG,
            "the lockfile resolves {name} at version {}, which is not the {ENTITY_RUNTIME_TAG} \
             release",
            resolved.version
        );
    }
}

#[test]
fn no_member_crate_declares_its_own_git_dependency() {
    let crates = workspace_root().join("crates");
    let mut manifests: Vec<PathBuf> = std::fs::read_dir(&crates)
        .expect("the workspace has a crates directory")
        .map(|entry| {
            entry
                .expect("a readable crates entry")
                .path()
                .join("Cargo.toml")
        })
        .filter(|manifest| manifest.is_file())
        .collect();
    manifests.sort();
    assert!(
        !manifests.is_empty(),
        "the workspace has member manifests to check"
    );

    for manifest in manifests {
        let offending: Vec<String> = read(&manifest)
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with('#') && line.contains("git = \""))
            .map(str::to_owned)
            .collect();
        assert!(
            offending.is_empty(),
            "{} declares a crate-local git pin: {offending:?}; the tag belongs to \
             [workspace.dependencies] alone",
            manifest.display()
        );
    }
}

/// A package as `Cargo.lock` resolved it: the release version and the git tag it came from.
struct Resolved {
    version: String,
    tag: String,
}

/// The workspace root, two directories above this crate's manifest.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root exists above this crate")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// Every `name = { git = "<repository>", tag = "<tag>" }` declaration in one manifest.
fn declared_pins(manifest: &str, repository: &str) -> BTreeMap<String, String> {
    let marker = format!("git = \"{repository}\"");
    manifest
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#') && line.contains(&marker))
        .map(|line| {
            let (name, rest) = line
                .split_once('=')
                .expect("a dependency declaration names its crate");
            let tag = quoted(rest, "tag")
                .unwrap_or_else(|| panic!("{} names a git dependency without a tag", name.trim()));
            (name.trim().to_owned(), tag)
        })
        .collect()
}

/// Every locked package whose source is a tagged checkout of `repository`.
fn locked_pins(lock: &str, repository: &str) -> BTreeMap<String, Resolved> {
    let prefix = format!("git+{repository}?tag=");
    let mut resolved = BTreeMap::new();
    let mut name = String::new();
    let mut version = String::new();
    for line in lock.lines().map(str::trim) {
        if let Some(value) = quoted(line, "name") {
            name = value;
        } else if let Some(value) = quoted(line, "version") {
            version = value;
        } else if let Some(source) = quoted(line, "source") {
            let Some(rest) = source.strip_prefix(&prefix) else {
                continue;
            };
            let tag = rest.split('#').next().unwrap_or(rest).to_owned();
            resolved.insert(
                name.clone(),
                Resolved {
                    version: version.clone(),
                    tag,
                },
            );
        }
    }
    resolved
}

/// The string value of `key = "…"` within one line, if the line carries that key.
fn quoted(line: &str, key: &str) -> Option<String> {
    let needle = format!("{key} = \"");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}
