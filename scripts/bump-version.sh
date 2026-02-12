#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

CARGO_TOML="$ROOT_DIR/Cargo.toml"

# Extract current version from Cargo.toml [package] section
current_version=$(sed -n '/^\[package\]/,/^\[/{s/^version = "\([^"]*\)"/\1/p;}' "$CARGO_TOML")

usage() {
  cat <<EOF
Usage: $(basename "$0") <major|minor|patch|VERSION> [OPTIONS]

Bump the version across all project config files.

  major        Bump major version (e.g. 0.1.1 -> 1.0.0)
  minor        Bump minor version (e.g. 0.1.1 -> 0.2.0)
  patch        Bump patch version (e.g. 0.1.1 -> 0.1.2)
  VERSION      Set an explicit version (e.g. 2.0.0)

Current version: $current_version

Options:
  -h, --help   Show this help message
  --tag        Create a git commit and tag after bumping
  --push       Push commit and tags to remote (implies --tag)
EOF
  exit 0
}

# Parse flags
CREATE_TAG=false
PUSH=false
POSITIONAL=()
for arg in "$@"; do
  case "$arg" in
  -h | --help) usage ;;
  --tag) CREATE_TAG=true ;;
  --push) PUSH=true; CREATE_TAG=true ;;
  *) POSITIONAL+=("$arg") ;;
  esac
done

if [ ${#POSITIONAL[@]} -ne 1 ]; then
  echo "Error: exactly one version argument required."
  echo ""
  usage
fi

BUMP="${POSITIONAL[0]}"

IFS='.' read -r major minor patch <<<"$current_version"

case "$BUMP" in
major) new_version="$((major + 1)).0.0" ;;
minor) new_version="$major.$((minor + 1)).0" ;;
patch) new_version="$major.$minor.$((patch + 1))" ;;
*)
  if [[ ! "$BUMP" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: invalid version '$BUMP'. Must be 'major', 'minor', 'patch', or a semver like X.Y.Z."
    exit 1
  fi
  new_version="$BUMP"
  ;;
esac

echo "Bumping version: $current_version -> $new_version"
echo ""

# Update Cargo.toml (only the [package] version, not dependency versions)
sed -i '' "s/^version = \"$current_version\"/version = \"$new_version\"/" "$CARGO_TOML"
echo "  Updated Cargo.toml"

# Update Cargo.lock by running cargo check
cd "$ROOT_DIR"
cargo update -p markdown-viewer --precise "$new_version" 2>/dev/null \
  || cargo check --quiet 2>/dev/null \
  || true
echo "  Updated Cargo.lock"

echo ""
echo "Version bumped to $new_version"

if [ "$CREATE_TAG" = true ]; then
  echo ""
  git add "$CARGO_TOML" "$ROOT_DIR/Cargo.lock"
  git commit -m "release: v$new_version"
  git tag "v$new_version"
  echo "Created commit and tag v$new_version"

  if [ "$PUSH" = true ]; then
    git push && git push --tags
    echo "Pushed to remote"
  fi
fi
