# Состояние проекта — 13.11.2026

SOURCE: аудит, переработка ТЗ и реализация по этапам разрешены пользователем. Предлогинное подключение после initial enrollment, service-only token vault и отключение autoconnect подписанной серверной policy утверждены. Это начальная реализация, **не готовый VPN/MSI**.

| Gate | Проверенный результат | Осталось |
|---|---|---|
| G0 | Аудит, ТЗ v2, 5 ADR, 13 gates / 140 подзадач, upstream/license evidence | Последующие product OPEN решения |
| G1 | Rust workspace/toolchain/lock, local CLI, CI config, license inventory | GitHub workflow ещё не запускался, remote отсутствует |
| G3 | Ed25519 exact-byte verification, binding/lease/version/digest; Node34 + Python1 public vectors; Python↔Rust interop | Согласование с владельцем issuer; durable rollback floor |
| G2 | SCM lifecycle, local pipe/DACL, настоящий token SID/session adapter, pure authorization, status-only protocol; ARM64 EXE собран | Native transport/SCM tests, adversarial identities; server authenticity до auth IPC |
| G4 | Loopback mock OTP/enrollment/bundle/policy/revoke; 8 HTTP tests | Production HTTP adapter, durable server behaviour |
| G5 | Official Xray/Wintun скачаны и hash-verified; Windows11 Pro ARM установлена в Fusion | Native TUN на разрешённом staging; отдельная x64 приёмка |
| G6 | Machine DPAPI primitive с bounds/zeroizing и 3 native tests; только cross-compile/build | Service-only файловый vault/ACL, persistence, supervisor, runtime DPAPI/boot |
| G7–G12 | Подробно спланированы | Интеграция сервера, сеть, GUI/MSI, release/pilot |

## VERIFIED local

macOS arm64, Rust1.99.1 изолированно в .tools; Node24.11.0, Python3.9.6. Shell profile не менялся. Локальные commits, без публикации.

- `python3 tools/dev.py test`: PASS — fmt/clippy, **29 Rust tests +8 Node HTTP +4 Python tools =41**. Node34 public vectors проверяются отдельно; Rust дополнительно принимает Python fixture, отвергает его повторную сериализацию без новой подписи.
- Optional PyCA50.0.2 reference проверил **35 public vectors**. Private test keys существуют только в памяти генераторов, в fixtures только public material. Четыре Mac wheels сверены с official PyPI hashes; это local dev tool, не production issuer.
- Windows x64/ARM64 MSVC cargo check/clippy PASS. Это cross-check, не MSVC EXE linking и не Windows CI.
- ARM64 native EXE и adapter-test EXE успешно собраны на Mac через Rust gnullvm + pinned/hash-verified Zig0.15.2. Payload `artifacts/windows-lab`, read-only ISO `artifacts/ScrumeVPN-Lab-ARM64.iso`. Signing: UNSIGNED_LAB_ONLY; runtime: BUILT_NOT_EXECUTED.
- Cargo inventory:70 dependencies, actual license files; development SBOM/NOTICE. Не заменяет final release audit.

Evidence: docs/evidence/g2-*.txt, g3-python-*.txt, python-signing-deps.json, dependencies.json, sbom.cdx.json и public fixtures. A16/A17/A18 PASS только в указанном unit scope; остальные57 acceptance scenarios NOT_RUN. Windows E2E PASS пока нет.

## VMware Fusion

Создана **ScrumeVPN Lab**: Windows11 Pro ARM64, UEFI Secure Boot/vTPM,4 CPU,6144MB RAM,128GB sparse disk,NAT. Новые passwords вводил пользователь; агент их не сохранял/не извлекал. Windows install/OOBE завершены, рабочий стол наблюдался. Пользователь установил Tools и перезагрузил Windows; Fusion теперь показывает **Reinstall VMware Tools**, что подтверждает установленный Tools. После перезагрузки Windows применяет обновления (наблюдалось30%, затем Restarting); не прерывать питание.

Синтетический ввод в гостевую систему ещё не подтверждён. Прежние coordinate calls возвращали noWindowsAvailable; native Fusion AX menu работает. Read-only test ISO подключён через Fusion CD/DVD settings (название ISO и Connect=1 подтверждены); `run-native-tests.cmd` запускает3 DPAPI tests и1 console IPC test и сохраняет guest TEMP log. Console IPC test пропускается в обычном CI: требует active console и отсутствия работающей product service. После подключения ISO CUA стал возвращать timeoutReached даже при чтении окна; reset сеанса не помог. Пользователю передан запуск launcher после Windows Update; результат ожидается. Native тесты ещё **не исполнены**. Следом — protected-directory temporary SYSTEM service install/status/stop smoke, затем clean snapshot/evidence и сетевой POC.

Encrypted vmrun guest operations требуют VM password; агент не читает Keychain и не передаёт credentials через чат. Нет host home sharing или нового remote access. ARM64 guest не закрывает Windows x64 driver/platform acceptance.

## Следующий этап

1. Дождаться Windows Update, проверить guest input, подключить test ISO, выполнить native adapter tests и SCM smoke. Исправить ошибки по результатам; собрать build hashes/evidence.
2. Реализовать protected filesystem vault и durable rollback state после проверки native security boundary; проверить ACL/second user/reboot.
3. G5 настоящий Xray TUN с baseline/recovery на approved staging, до GUI/MSI. Staging и существующий issuer source пока не предоставлены. Не использовать production сведения из исходного SPEC как разрешение на подключение.

Production issuer/Xray/employees и сеть хостового Mac не изменялись. Главный остаточный риск — native Windows network lifecycle и server revoke semantics.
