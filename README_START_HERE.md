# ScrumeVPN — начать здесь

Перенос на Windows / новая рабочая сессия: **[CONTINUE_HERE.md](CONTINUE_HERE.md)**,
[точка продолжения](docs/SESSION_HANDOFF.md), [Windows setup](docs/WINDOWS_DEVELOPMENT.md).

Дата: 12.11.2026. Теперь это документация **и начальная реализация**, не готовый VPN/MSI. Текущее проверенное состояние — docs/STATUS.md. Исходный SPEC_ORIGINAL.md сохранён без изменений, v1 инженерного ТЗ — docs/archive/MASTER_SPEC_v1.md.

1. CODEX_MASTER_SPEC.md — переработанное исполнимое ТЗ v2.
2. docs/AUDIT.md — слабые места и исправления.
3. docs/DECISIONS.md — прямые решения пользователя, отдельно от предложений.
4. docs/EXECUTION_PLAN.md — 13 gates, 140 подзадач до приёмки.
5. docs/TRACEABILITY.md — требование → код → тест → состояние.
6. docs/ARCHITECTURE.md и docs/ADRs — архитектура и альтернативы.
7. docs/API_CONTRACT.md, docs/SIGNING_CONTRACT.md, contracts — proposed wire contracts.
8. docs/UPSTREAMS_AND_LICENSES.md — проверенные публичные repositories/licenses/pins.
9. docs/WINDOWS_LAB.md — VMware Fusion ARM и отдельная выпускная Windows x64 проверка.

## Запуск

На текущем Mac Rust установлен изолированно в .tools. Shell profile не менялся.

```sh
cd /Users/headliner/Documents/vpn/ScrumeVPN
python3 tools/dev.py doctor
python3 tools/dev.py test
python3 tools/dev.py licenses
python3 tools/dev.py check-windows
python3 tools/dev.py fetch-runtime
```

На другом компьютере сначала pinned Rust из rust-toolchain.toml, Node.js и Python3; tools/bootstrap.py готовит изолированный Rust на поддерживаемых хостах. Для MSVC EXE linking нужны Windows SDK/Build Tools, cargo check на Mac их не заменяет.

Тесты не требуют VPN server/OTP/production secrets. Runtime download проверяет SHA256, не запускает binaries и не меняет сеть Mac. Windows SCM/TUN/DPAPI/MSI E2E отдельно; workflow подготовлен, но без private remote не выполнялся.

Boot-connect после initial enrollment утверждён пользователем. Machine DPAPI + ACL service-only — обязательны. Signed policy отключает autoconnect. Это требование и core logic, а не уже выполненный boot test.

Продолжать ближайшую незаблокированную задачу roadmap без повторного разрешения каждого локального этапа. Production changes и публикация инфраструктурных документов не входят в автоматическое продолжение. Исходный SPEC содержит инфраструктурные сведения: remote должен быть private; секретов в git/CI/чат не добавлять.

## Лабораторная Windows ARM64 сборка

На текущем Mac: `python3 tools/build-windows-lab.py --iso`. Pinned Zig проверяется по SHA256; создаются native EXE и read-only ISO без изменения VM. В guest открыть `run-native-tests.cmd` с ISO. Эти проверки охватывают DPAPI primitive и console IPC; TUN/vault ACL/boot/MSI отдельно. Дальнейшие шаги и границы — docs/WINDOWS_LAB.md.

Для переноса: `python3 tools/dev.py export-handoff` (clean committed Git). Исходники+history в artifacts/handoff. На Windows сборка: `py -3 tools/build-windows.py --target x86_64-pc-windows-msvc`; подробности в WINDOWS_DEVELOPMENT. Новый SCM ISO: `artifacts/ScrumeVPN-Lab-ARM64-SCM.iso`.
