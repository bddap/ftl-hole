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
  #!/usr/bin/env bash
  set -euo pipefail

  just need rg 'cargo install ripgrep'
  just need static-reload 'cargo install --git https://github.com/bddap/static-reload.git'
  just need entr 'http://eradman.com/entrproject/ or apt install entr or brew install entr'
  just build-web
  
  find ./dist/ | entr echo /_ | static-reload dist '127.0.0.1:47109'
