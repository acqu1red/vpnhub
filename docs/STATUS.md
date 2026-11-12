# Состояние проекта — 12.11.2026

## Результат текущего прохода

Аудит исходного пакета завершён; ТЗ v2, архитектура, 4 ADR, traceability и roadmap из 13 gates / 140 подзадач записаны. Исходные SPEC не изменены. Начата локальная реализация. Это **не готовый VPN и не установщик**.

SOURCE: boot-connect после initial enrollment; service-only token vault; machine DPAPI+ACL; signed policy может отключить auto_connect. Данные решения отражены в ТЗ и core tests.

| Gate | Факт | Осталось |
|---|---|---|
| G0 | Audit/ADRs/requirements/upstream evidence выполнены | Product OPEN решения для последующих gates |
| G1 | Rust workspace, pinned toolchain/lock, dev CLI, tests, CI config | Расширять вместе с adapters; GitHub workflow ещё не запускался |
| G3 | Strict Ed25519 envelope, binding/lease/version/digest, 34 public vectors | Python reference, backend approval, durable rollback state |
| G2 | SCM lifecycle scaffold; cargo check x64 и ARM64 | Secure pipe/SID/ACL, actual Windows runtime tests |
| G4 | Loopback mock OTP/enrollment/bundle/policy/revoke; 8 HTTP tests | Rust HTTP adapter, GUI integration, durable server behaviour |
| G5 | Official runtime downloaded+hash-verified; Fusion launched | Windows guest install + approved staging + настоящий TUN POC |
| G6–G12 | Подробно спланированы | Реализация и реальные acceptance tests |

## VERIFIED local checks

Платформа: macOS arm64. Rust1.99.1 установлен изолированно в `.tools`, shell profile не изменён. Node24.11.0, Python3.9.6. Git initialized locally; результаты сохраняются локальными commits, remote не создан и публикации нет.

- `python3 tools/dev.py test` — PASS: rustfmt, clippy -D warnings, 20 Rust tests, 34 signed vectors, 8 mock HTTP tests, 4 Python supply-chain tests, offline lab verification, explicit service capabilities.
- `python3 tools/dev.py check-windows` — PASS **cargo check** для x86_64-pc-windows-msvc с macOS; это не линковка EXE, не Windows CI и не E2E.
- `cargo check --workspace --locked --target aarch64-pc-windows-msvc` — PASS cross-check для дополнительного Fusion target. Дополнительно `cargo clippy --all-targets --target <x64/ARM64> -- -D warnings` прошёл для обеих Windows архитектур; evidence windows-checks.txt.
- `python3 tools/dev.py licenses` — PASS inventory 70 packages; actual license files найдены, unknown license metadata нет. THIRD_PARTY_NOTICES.txt и development CycloneDX SBOM созданы. Полный legal/vulnerability/release audit ещё не завершён.
- `python3 tools/dev.py fetch-runtime` — PASS после retry обрыва сети: official Xray v26.11.9 и Wintun0.14.1 archive SHA256 совпали; EXE/DLL извлечены в `.cache/runtime/windows-x64`, не исполнялись. Download manifest содержит hashes actual files.

Evidence: docs/evidence/local-checks.txt, dependencies.json, sbom.cdx.json, upstreams.json, runtime-download.json; source tests и fixtures. В docs/ACCEPTANCE_REPORT.json только A17/A18 PASS в unit scope; 58 полных сценариев пока NOT_RUN. Ни одного Windows E2E PASS не заявлено.

## VMware Fusion

Fusion запущен; изначально VM library пуста. Через Get Windows from Microsoft выбрана Windows11 Pro ARM en-us. Последний наблюдаемый UI: загрузка99.8%, Continue disabled; в журнале download timeouts. Штатная vmw_esd2iso использована для получения official Microsoft image URL/hash. Её HTTP URL не был использован для ручной дозагрузки: попытки HTTPS на Microsoft delivery hosts остановлены из-за hostname mismatch сертификата. TLS verification не отключалась. Windows guest не создана и тесты в ней не выполнялись. Fusion оставлен на своём экране загрузки; partial images в его temp directory и локальной .cache не считаются валидными установочными образами. Альтернатива — получить ISO с [официальной страницы Microsoft](https://www.microsoft.com/en-us/software-download/windows11arm64) через исправное HTTPS-соединение и сверить published SHA256, затем создать VM. Не считать Windows установленной по progress bar.

Mac Apple Silicon/Fusion ARM guest не закрывает x64 driver/platform acceptance. Windows x64 E2E остаётся отдельной внешней зависимостью.

## Ближайшие работы

1. Завершить официальный Windows image/guest setup; EULA и новые credentials — пользовательский handoff при необходимости.
2. Реализовать secure named pipe + реальную Windows identity, service vault/ACL/DPAPI и SCM runtime tests.
3. Выполнить G5 native Xray TUN на разрешённом staging до production UI/MSI. Нужны сам staging и исходник issuer для дальнейшего adapter; секреты в чат/репозиторий не передавать.

Production issuer/Xray/employees не изменялись; сеть host Mac не перенастраивалась. Основной остаточный риск — реальные Windows network lifecycle и server revoke semantics, которые unit tests не доказывают.
