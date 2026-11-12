# Проверенные upstream и лицензии

Дата: 12.11.2026. VERIFIED — metadata/LICENSE/tree, не работоспособность продукта. Evidence: docs/evidence/upstreams.json, runtime-pins.json, reference-hashes.json и license texts. 10 репозиториев публично доступны, archived=false; pushed_at не доказывает качество сопровождения. Полные source readme/Go files для исследования только в .cache/upstream-reference, не входят в продукт.

| Upstream | Проверка | Решение |
|---|---|---|
| [XTLS/Xray-core](https://github.com/XTLS/Xray-core) | MPL-2.0; main c4561418; release v26.11.9 commit676eaf66 отдельно | Official pinned binary; native TUN требует Windows POC |
| [XTLS/Xray-docs-next](https://github.com/XTLS/Xray-docs-next) | CC-BY-SA-4.0; 44ed2e81 | Reference only |
| [XTLS/Xray-examples](https://github.com/XTLS/Xray-examples) | CC-BY-SA-4.0; 167cded1 | Reference only |
| [tauri-apps/tauri](https://github.com/tauri-apps/tauri) | MIT/Apache license texts; 5edd31c8 | Будущий GUI, пока не включён |
| [tauri-apps/create-tauri-app](https://github.com/tauri-apps/create-tauri-app) | MIT/Apache texts; c6f5cf68 | Optional generator, reuse пока нет |
| [mullvad/windows-service-rs](https://github.com/mullvad/windows-service-rs) | MIT OR Apache-2.0; 82fe85ad | Crate windows-service=0.8.0; отдельная permissive library |
| [microsoft/windows-rs](https://github.com/microsoft/windows-rs) | MIT/Apache texts; 8ca69fda | Actual transitive windows-sys=0.59.0 |
| [dalek-cryptography/curve25519-dalek](https://github.com/dalek-cryptography/curve25519-dalek) | Root NOASSERTION, ed25519-dalek crate BSD-3-Clause | Actual package license из Cargo inventory |
| [WireGuard/wintun](https://github.com/WireGuard/wintun) | Source GPL-2.0; prebuilt DLL separate PBL | Только официальный ZIP0.14.1, actual hash/license verified |
| [Shurubtsov/rustvpn](https://github.com/Shurubtsov/rustvpn/tree/6a942e40624ea4bd2667f994a70aff62a3f2c152) | MIT заявлена README/Cargo, LICENSE/COPYING/NOTICE нет | No reuse; proxy sidecar не заменяет SYSTEM/TUN |

Полные immutable commits и URLs в evidence JSON. Root license metadata не заменяет проверку nested/transitive components. Перед выпуском нужен audit actual binaries и библиотек.

## Runtime

upstreams.lock.json фиксирует official HTTPS URLs, version, source commit, SHA256, allowlisted archive paths. tools/dev.py fetch-runtime проверяет archive hash до извлечения, не использует extractall и не запускает Windows binary на Mac. Actual download status в STATUS.md, URL не означает успешную загрузку.

Wintun0.14.1 ZIP SHA256 сверён с wintun.net, лицензия извлечена из actual ZIP. Распространение DLL только по PBL и через Permitted API, без модификации и endorsement брендом WireGuard. Xray MPL notices и transitive binary obligations проверяются для финальной поставки отдельно.

## Rust

Exact direct versions в Cargo.toml, dependency graph/checksums в Cargo.lock. tools/dev.py licenses читает actual downloaded crate LICENSE/NOTICE. Development inventory/SBOM не равны финальному MSI SBOM. Vulnerability audit и release signing — G11.

Не использовать GPL VPN client code/assets Amnezia/NekoRay/Throne/Hiddify/v2rayN/Mullvad VPN app. Для copied file нужен source URL/commit/file/license/modifications/reason в THIRD_PARTY_REUSE.md. Сейчас сторонний source в реализацию не копировался. Не использовать mirror binaries или latest на рабочих ПК.
