# https://github.com/casey/just

bin := "ftl-hole"

need command intructions:
  #!/usr/bin/env bash
  set -euo pipefail

  if ! which {{command}} > /dev/null; then
    echo you are going to need {{command}} .
    echo you can install it using \"{{intructions}}\"
    exit 1
  fi

need-toolchain:
  #!/usr/bin/env bash
  set -euo pipefail

  if ! rustup target list --installed | grep '^wasm32-unknown-unknown$' > /dev/null; then
    echo You are going to need to install the wasm32-unknown-unknown target.
    echo "this command should do the trick: 'rustup target add wasm32-unknown-unknown'"
    exit 1
  fi

build-web:
  #!/usr/bin/env bash
  set -euo pipefail

  just need-toolchain
  cargo build --target wasm32-unknown-unknown --release --bin {{bin}}

  mkdir -p dist
  cp target/wasm32-unknown-unknown/release/{{bin}}.wasm dist/main.wasm
  cp asset/* dist

serve:
  just need basic-http-server 'cargo install basic-http-server'
  basic-http-server --addr '127.0.0.1:47109' dist

signal-dist-changes:
  just need websocat 'cargo install websocat'
  just need entr 'your system package manager'
  find ./dist/ | entr echo /_ | websocat -s '127.0.0.1:47110'

serves:
  #!/usr/bin/env bash
  set -euo pipefail

  just need rg 'cargo install ripgrep'

  just build-web

  parallel -k --ungroup --halt now,fail=1 --halt now,success=1 ::: \
    "just signal-dist-changes" \
    "just serve"
