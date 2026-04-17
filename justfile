name := 'caffeine-applet'
appid := 'com.github.codevardhan.caffeine-applet'

rootdir := ''
prefix := '/usr'

base-dir := absolute_path(clean(rootdir / prefix))

bin-src := 'target' / 'release' / name
bin-dst := base-dir / 'bin' / name

desktop := appid + '.desktop'
desktop-src := 'resources' / desktop
desktop-dst := clean(rootdir / prefix) / 'share' / 'applications' / desktop

appdata := appid + '.metainfo.xml'
appdata-src := 'resources' / appdata
# Changed: share/appdata is deprecated; FreeDesktop spec uses share/metainfo
appdata-dst := clean(rootdir / prefix) / 'share' / 'metainfo' / appdata

icons-src := 'resources' / 'icons' / 'hicolor'
icons-dst := clean(rootdir / prefix) / 'share' / 'icons' / 'hicolor'

app-icons-src-dir := icons-src / 'scalable' / 'apps'
app-icons-dst-dir := icons-dst / 'scalable' / 'apps'

# Default recipe which runs `just build-release`
default: build-release

# Runs `cargo clean`
clean:
    cargo clean

# Removes vendored dependencies
clean-vendor:
    rm -rf .cargo vendor vendor.tar

# `cargo clean` and removes vendored dependencies
clean-dist: clean clean-vendor

# Compiles with debug profile
build-debug *args:
    cargo build {{args}}

# Compiles with release profile
build-release *args: (build-debug '--release' args)

# Compiles release profile with vendored dependencies
build-vendored *args: vendor-extract (build-release '--frozen --offline' args)

# Runs a clippy check
check *args:
    cargo clippy --all-features {{args}} -- -W clippy::pedantic

# Runs a clippy check with JSON message format
check-json: (check '--message-format=json')

# Run the application for testing purposes
run *args:
    env RUST_BACKTRACE=full cargo run --release {{args}}

# Checks that required runtime services are present on the host
check-env:
    #!/usr/bin/env bash
    set -euo pipefail
    ok=true
    if ! busctl status org.freedesktop.login1 &>/dev/null 2>&1; then
        echo "WARNING: org.freedesktop.login1 (logind/elogind) not reachable on the system bus." >&2
        echo "         The applet will build fine but inhibit locks won't work at runtime." >&2
        ok=false
    fi
    $ok && echo "Environment OK."

# Installs files
install:
    #!/usr/bin/env bash
    set -euo pipefail

    # Binary
    install -Dm0755 {{bin-src}} {{bin-dst}}

    # Desktop & AppStream
    install -Dm0644 resources/app.desktop {{desktop-dst}}
    install -Dm0644 resources/app.metainfo.xml {{appdata-dst}}

    # Icons: copy and rename
    mkdir -p {{app-icons-dst-dir}}
    shopt -s nullglob
    for src in {{app-icons-src-dir}}/*.svg; do
      base="$(basename "$src")"
      case "$base" in
        icon.svg)
          install -Dm0644 "$src" "{{app-icons-dst-dir}}/{{appid}}.svg"
          ;;
        icon-*.svg)
          suffix="${base#icon}"
          install -Dm0644 "$src" "{{app-icons-dst-dir}}/{{appid}}${suffix}"
          ;;
        coffee-full.svg|active.svg)
          install -Dm0644 "$src" "{{app-icons-dst-dir}}/{{appid}}.svg"
          install -Dm0644 "$src" "{{app-icons-dst-dir}}/{{appid}}.On.svg"
          ;;
        coffee-empty.svg|inactive.svg)
          install -Dm0644 "$src" "{{app-icons-dst-dir}}/{{appid}}.Off.svg"
          ;;
        {{appid}}*.svg)
          install -Dm0644 "$src" "{{app-icons-dst-dir}}/$base"
          ;;
        *)
          echo "Skipping unknown icon: $base" >&2
          continue
          ;;
      esac
    done

    # Refresh caches (best-effort; not all distros ship these tools)
    if command -v update-desktop-database &>/dev/null; then
        update-desktop-database -q "{{clean(rootdir / prefix) / 'share' / 'applications'}}" || true
    fi
    if command -v gtk-update-icon-cache &>/dev/null; then
        gtk-update-icon-cache -qtf "{{icons-dst}}" || true
    fi

# Uninstalls installed files
uninstall:
    #!/usr/bin/env bash
    set -euo pipefail
    rm -f {{bin-dst}} {{desktop-dst}} {{appdata-dst}}
    rm -f \
      {{app-icons-dst-dir}}/{{appid}}.svg \
      {{app-icons-dst-dir}}/{{appid}}.On.svg \
      {{app-icons-dst-dir}}/{{appid}}.Off.svg \
      {{app-icons-dst-dir}}/{{appid}}-*.svg
    if command -v update-desktop-database &>/dev/null; then
        update-desktop-database -q "{{clean(rootdir / prefix) / 'share' / 'applications'}}" || true
    fi
    if command -v gtk-update-icon-cache &>/dev/null; then
        gtk-update-icon-cache -qtf "{{icons-dst}}" || true
    fi

# Vendor dependencies locally
vendor:
    #!/usr/bin/env bash
    mkdir -p .cargo
    cargo vendor --sync Cargo.toml | head -n -1 > .cargo/config.toml
    echo 'directory = "vendor"' >> .cargo/config.toml
    echo >> .cargo/config.toml
    echo '[env]' >> .cargo/config.toml
    if [ -n "${SOURCE_DATE_EPOCH}" ]
    then
        source_date="$(date -d "@${SOURCE_DATE_EPOCH}" "+%Y-%m-%d")"
        echo "VERGEN_GIT_COMMIT_DATE = \"${source_date}\"" >> .cargo/config.toml
    fi
    if [ -n "${SOURCE_GIT_HASH}" ]
    then
        echo "VERGEN_GIT_SHA = \"${SOURCE_GIT_HASH}\"" >> .cargo/config.toml
    fi
    tar pcf vendor.tar .cargo vendor
    rm -rf .cargo vendor

# Extracts vendored dependencies
vendor-extract:
    rm -rf vendor
    tar pxf vendor.tar
