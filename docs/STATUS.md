# Состояние проекта — 13.11.2026

SOURCE: аудит, переработка ТЗ и реализация по этапам разрешены пользователем. Предлогинное подключение после initial enrollment, service-only token vault и отключение autoconnect подписанной серверной policy утверждены. Это начальная реализация, **не готовый VPN/MSI**.

| Gate | Проверенный результат | Осталось |
|---|---|---|
| G0 | Аудит, ТЗ v2, 5 ADR, 13 gates / 140 подзадач, upstream/license evidence | Последующие product OPEN решения |
| G1 | Rust workspace/toolchain/lock, local CLI, CI config, license inventory | GitHub workflow ещё не запускался, remote отсутствует |
| G3 | Ed25519 exact-byte verification, binding/lease/version/digest; Node34 + Python1 public vectors; Python↔Rust interop | Согласование с владельцем issuer; durable rollback floor |
| G2 | SCM lifecycle, local pipe/DACL, настоящий token SID/session adapter, pure authorization, status-only protocol; ARM64 console transport test PASS | SCM tests, adversarial identities; server authenticity до auth IPC |
| G4 | Loopback mock OTP/enrollment/bundle/policy/revoke; 8 HTTP tests | Production HTTP adapter, durable server behaviour |
| G5 | Official Xray/Wintun скачаны и hash-verified; Windows11 Pro ARM установлена в Fusion | Native TUN на разрешённом staging; отдельная x64 приёмка |
| G6 | Machine DPAPI primitive:3 native ARM64 tests PASS | Service-only файловый vault/ACL, persistence, supervisor, boot |
| G7–G12 | Подробно спланированы | Интеграция сервера, сеть, GUI/MSI, release/pilot |

## VERIFIED local

macOS arm64, Rust1.99.1 изолированно в .tools; Node24.11.0, Python3.9.6. Shell profile не менялся. Локальные commits, без публикации.

- `python3 tools/dev.py test`: PASS — fmt/clippy, **29 Rust tests +8 Node HTTP +4 Python tools =41**. Node34 public vectors проверяются отдельно; Rust дополнительно принимает Python fixture, отвергает его повторную сериализацию без новой подписи.
- Optional PyCA50.0.2 reference проверил **35 public vectors**. Private test keys существуют только в памяти генераторов, в fixtures только public material. Четыре Mac wheels сверены с official PyPI hashes; это local dev tool, не production issuer.
- Windows x64/ARM64 MSVC cargo check/clippy PASS. Это cross-check, не MSVC EXE linking и не Windows CI.
- ARM64 native EXE и adapter-test EXE успешно собраны на Mac через Rust gnullvm + pinned/hash-verified Zig0.15.2. Payload `artifacts/windows-lab`, read-only ISO `artifacts/ScrumeVPN-Lab-ARM64.iso`. Signing: UNSIGNED_LAB_ONLY. Build manifests сохраняют исходный BUILT_NOT_EXECUTED; отдельный runtime evidence теперь подтверждает4 native adapter tests PASS, 0.15s.
- Cargo inventory:70 dependencies, actual license files; development SBOM/NOTICE. Не заменяет final release audit.

Evidence: docs/evidence/g2-*.txt, g3-python-*.txt, python-signing-deps.json, dependencies.json, sbom.cdx.json и public fixtures. A16/A17/A18 PASS только в указанном unit scope; остальные57 acceptance scenarios NOT_RUN. Дополнительно **4 native Windows ARM64 tests PASS**: docs/evidence/g2-native-arm64-tests.txt. Полный Windows x64 E2E ещё не проведён; A21–23 не закрыты этим smoke-test.

## VMware Fusion

Создана **ScrumeVPN Lab**: Windows11 Pro ARM64, UEFI Secure Boot/vTPM,4 CPU,6144MB RAM,128GB sparse disk,NAT. Новые passwords вводил пользователь; агент их не сохранял/не извлекал. Windows install/OOBE завершены, рабочий стол наблюдался. Пользователь установил Tools и перезагрузил Windows; Fusion теперь показывает **Reinstall VMware Tools**, что подтверждает установленный Tools. Windows Update завершил перезагрузку; рабочий стол и консоль с результатом native tests наблюдались.

Read-only test ISO подключён через Fusion CD/DVD settings; пользователь запустил `run-native-tests.cmd`. Агент увидел консоль **4 passed / 0 failed / 0 ignored / 0.15s**, пользователь подтвердил полный текст. Это3 DPAPI tests и1 console IPC test. Console test пропускается в обычном CI, требует active console и отсутствия работающей product service. Guest log сохраняется в TEMP; username редактирован в репозиторном evidence. Build hashes связаны с mounted payload manifest; отдельный hash внутри guest ещё не собран.

Синтетический клик внутри guest по-прежнему возвращает noWindowsAvailable. Native Fusion AX menu и чтение окна снова работают; прежний timeoutReached больше не является текущим blocker. Создаётся snapshot перед SCM smoke. Следующий runtime шаг — temporary SYSTEM service install/status/stop/delete; затем файловый vault/ACL и сетевой POC.

Encrypted vmrun guest operations требуют VM password; агент не читает Keychain и не передаёт credentials через чат. Нет host home sharing или нового remote access. ARM64 guest не закрывает Windows x64 driver/platform acceptance.

## Следующий этап

1. Завершить clean snapshot, выполнить SCM smoke и собрать guest build hashes. Исправить ошибки по результатам; собрать build hashes/evidence.
2. Реализовать protected filesystem vault и durable rollback state после проверки native security boundary; проверить ACL/second user/reboot.
3. G5 настоящий Xray TUN с baseline/recovery на approved staging, до GUI/MSI. Staging и существующий issuer source пока не предоставлены. Не использовать production сведения из исходного SPEC как разрешение на подключение.

Production issuer/Xray/employees и сеть хостового Mac не изменялись. Главный остаточный риск — native Windows network lifecycle и server revoke semantics.
