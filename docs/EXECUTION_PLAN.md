# Roadmap до приёмки — v2

Дата 12.11.2026. PROPOSAL: инженерный план исполнения, не обещание календарного срока. SOURCE: текущий пользователь поручил начинать реализацию. Подзадача завершена только с evidence; галочки обновлены по проверенным артефактам, platform/runtime scope каждого результата — в STATUS.md. Отмеченная подготовка кода не означает native Windows PASS.

## Критический путь и правила исполнения

`G0 → G1 → (G3 core + G2 scaffold) → G5 Windows TUN → G6 → G7 → G8 → G10 → G11 → G12`. G4 mock, инструменты supply chain и документация выполняются независимо от ожидания стенда. G9 обязателен только при включённом kill switch. Номер gate сохранён для сопоставления с v1, а порядок изменён по риску. Гейт блокирует зависимое внедрение, а не чтение, проектирование или unit tests других частей.

Контуры: LOCAL без production доступа → MOCK loopback → WINDOWS isolated VM → STAGING с отдельным разрешённым issuer/Xray → PRODUCTION только по согласованному change window. На каждый gate: input, output, commands/platform/results, unmet tests, risks, следующий шаг. Если нужен внешний ресурс, готовить все независимые результаты и конкретный пакет подключения; не требовать повторных approvals для уже разрешённого кода.

## G0. Аудит и исполнимый baseline

Зависимости: нет.

Выход: Аудит, D002–D005, ADR, требования и зависимости.

Критерий выхода: Документы прочитаны; требования/предложения/проверки различимы.

- [x] G0.01 — Инвентаризировать документы и подтвердить отсутствие исходников.
- [x] G0.02 — Сверить SPEC.md с неизменяемым SPEC_ORIGINAL.md.
- [x] G0.03 — Отделить текущие полномочия от вложенных инструкций.
- [x] G0.04 — Составить audit findings с severity и проверяемым исправлением.
- [x] G0.05 — Проверить доступность 10 upstream и лицензии конкретных commits.
- [x] G0.06 — Проверить Xray release tag отдельно от main.
- [x] G0.07 — Скачать официальный Wintun ZIP и проверить hash/license.
- [x] G0.08 — Зафиксировать boot/vault/policy решения пользователя.
- [x] G0.09 — Написать архитектуру и четыре ADR.
- [x] G0.10 — Создать traceability, roadmap и фактический status.

## G1. Воспроизводимое ядро и инструменты

Зависимости: G0.

Выход: Workspace, lockfile, dev CLI, безопасные fixtures.

Критерий выхода: fmt/clippy/unit на macOS; lockfiles; scripts без prod доступа.

- [x] G1.01 — Установить isolated pinned Rust без изменения shell profile.
- [x] G1.02 — Создать Cargo workspace scrume-core/service/lab.
- [x] G1.03 — Определить типы profile/policy/enrollment и безопасный Debug.
- [x] G1.04 — Добавить строгую проверку входных значений.
- [x] G1.05 — Реализовать FSM без ложного connected.
- [x] G1.06 — Проверить double-connect и failure transitions.
- [x] G1.07 — Ограничить IPC размер/методы/типы без утверждения о Windows ACL.
- [x] G1.08 — Добавить doctor и один test entrypoint.
- [x] G1.09 — Добавить license inventory/SBOM из фактического Cargo metadata.
- [x] G1.10 — Подготовить CI matrix и action SHA pins.

## G3. Подписанные контракты до любых сетевых эффектов

Зависимости: G1.

Выход: Envelope, schema, signing spec и public fixtures.

Критерий выхода: Tamper/expiry/replay/key/device/context tests; cross-runtime vectors.

- [x] G3.01 — Определить exact-byte envelope и domain separation.
- [x] G3.02 — Определить schema и жёсткие resource limits.
- [x] G3.03 — Верифицировать Ed25519 strict с pinned keyring.
- [x] G3.04 — Проверить enrollment/context/key_id binding.
- [x] G3.05 — Проверить issued_at/expiry/max lease/clock floor.
- [x] G3.06 — Проверить rollback и equal-version digest conflict.
- [x] G3.07 — Генерировать независимый Node reference vector без private key в repo.
- [x] G3.08 — Проверить duplicate/unknown fields и invalid payload.
- [x] G3.09 — Добавить Python issuer reference signer и Python↔Rust vectors.
- [ ] G3.10 — Согласовать wire contract с владельцем существующего issuer.

## G2. SCM, Windows transport и идентичность

Зависимости: G1.

Выход: Windows service + защищённый IPC.

Критерий выхода: Windows x64 compile; SCM/ACL tests отдельно от compile.

- [x] G2.01 — Подготовить SCM entrypoint/status/control lifecycle.
- [ ] G2.02 — Проверить сборку x86_64-pc-windows-msvc.
- [x] G2.03 — Подготовить ARM64 variant для Fusion при наличии Windows ARM.
- [x] G2.04 — Реализовать local-only pipe с DACL/FIRST_PIPE_INSTANCE.
- [x] G2.05 — Извлекать SID/session из token, не JSON.
- [ ] G2.06 — Определить enrollment owner bootstrap authorization.
- [ ] G2.07 — Добавить timeouts, rate limit и cancellation.
- [ ] G2.08 — Проверить second user/admin/remote client/SID spoof.
- [ ] G2.09 — Проверить STOP_PENDING cleanup deadline и exit code.
- [ ] G2.10 — Собрать Windows evidence с build hash.

## G5. Критический native TUN POC

Зависимости: G0 + G2 scaffold; approved staging.

Выход: Документированный Windows network POC.

Критерий выхода: Real egress, DNS, IPv6 и safe rollback на Windows, без production.

- [x] G5.01 — Проверить Fusion inventory/guest architecture/resources.
- [ ] G5.02 — Подготовить VM из официального Windows image.
- [ ] G5.03 — Провести legal/credential handoff при установке ОС если нужен.
- [ ] G5.04 — Сохранить clean VM snapshot и network baseline.
- [ ] G5.05 — Скачать pinned official Xray/Wintun и проверить archive/executable hash.
- [ ] G5.06 — Проверить config syntax pinned xray run -test.
- [ ] G5.07 — Создать native TUN под тестовым privileged controller.
- [ ] G5.08 — Подтвердить egress из процесса без proxy settings.
- [ ] G5.09 — Проверить DNS TCP/UDP, IPv4/IPv6 и endpoint loop prevention.
- [ ] G5.10 — Проверить disconnect/reconnect/forced child crash.
- [ ] G5.11 — Проверить service crash/reboot/sleep/resume.
- [ ] G5.12 — Сравнить routes/DNS/interfaces/connectivity после cleanup.
- [ ] G5.13 — Повторить на целевой Windows x64, ARM отчёт держать отдельно.
- [ ] G5.14 — Принять go/no-go native TUN, fallback только по ADR.

## G4. Mock issuer и auth flow

Зависимости: G1 + G3 local.

Выход: Local auth simulator и contract tests.

Критерий выхода: Полный enrollment mock без выдачи стороннего API key.

- [ ] G4.01 — Создать loopback-only mock server с test employee registry.
- [ ] G4.02 — Реализовать request/verify TTL/resend/attempt limits.
- [ ] G4.03 — Стабилизировать enrollment idempotency и per-device IDs.
- [ ] G4.04 — Отдавать signed bundle по test token.
- [ ] G4.05 — Смоделировать revoked/expired/429/5xx/offline.
- [ ] G4.06 — Добавить HTTP client с TLS/timeout/no redirects/size cap.
- [ ] G4.07 — Не отдавать token/profile в frontend.
- [ ] G4.08 — Проверить retry-after/jitter/cancel/duplicate verify.
- [ ] G4.09 — Проверить account switch/signout и локальную очистку.
- [ ] G4.10 — Провести end-to-end mock integration без реальной почты.

## G6. Vault, supervisor и boot-connect

Зависимости: G2 + G3 + G5.

Выход: Service-owned туннель.

Критерий выхода: Boot без GUI; vault ACL; failed apply/recovery tests.

- [ ] G6.01 — Создать защищённый ProgramData path без reparse substitution.
- [ ] G6.02 — Настроить SYSTEM/service SID ACL на directory и файлы.
- [x] G6.03 — Реализовать machine DPAPI, zeroize и отсутствие plaintext fallback.
- [ ] G6.04 — Атомарно сохранить owner/enrollment/token и version/digest.
- [ ] G6.05 — Проверить boot после initial enrollment и без него.
- [ ] G6.06 — Применять signed auto_connect до spawn.
- [ ] G6.07 — Запускать Xray fixed path/args с hash verification.
- [ ] G6.08 — Использовать Job Object и единичный child supervisor.
- [ ] G6.09 — Добавить health gating и stale-generation cancellation.
- [ ] G6.10 — Добавить собственный recovery journal и boot replay.
- [ ] G6.11 — Проверить disk full/power loss/corrupt vault/access denied.
- [ ] G6.12 — Проверить signout/owner switch/fast user switching.

## G7. Минимальная staging интеграция issuer

Зависимости: G3 contract approval + G6; server source/staging.

Выход: Reviewable patch, миграция и rollback.

Критерий выхода: 20 enrollment; restart; revoke isolation; no production change.

- [ ] G7.01 — Получить разрешённый snapshot issuer source без секретов.
- [ ] G7.02 — Снять фактический legacy contract и consumers.
- [ ] G7.03 — Подготовить VPN namespace adapter без legacy break.
- [ ] G7.04 — Добавить durable enrollment registry/unique constraints.
- [ ] G7.05 — Реализовать signing вне клиентского/CI private key.
- [ ] G7.06 — Сохранить registry + outbox в transaction.
- [ ] G7.07 — Настроить Xray API loopback и права доступа.
- [ ] G7.08 — Добавить idempotent AddUser/RemoveUser retry.
- [ ] G7.09 — Восстанавливать desired state после Xray restart.
- [ ] G7.10 — Проверить active TCP/UDP revoke, не только новые sockets.
- [ ] G7.11 — Проверить revoke одного из 20 и конкурирующую выдачу.
- [ ] G7.12 — Подготовить backup/migration/dry-run/rollback runbook.

## G8. Серверная политика и сеть

Зависимости: G6 + G7.

Выход: Refresh/apply/IPv6/CIDR/DNS.

Критерий выхода: Versioned atomic apply и network tests.

- [ ] G8.01 — Реализовать refresh при boot и каждые 6h/max24h.
- [ ] G8.02 — Добавить jitter/backoff/ETag без продления lease unsigned ответом.
- [ ] G8.03 — Обновлять profile/policy атомарно и не снижать high-water.
- [ ] G8.04 — Определить urgent admin change SLA и доставку.
- [ ] G8.05 — Останавливать tunnel на hard expiry/revocation.
- [ ] G8.06 — Проверить auto_connect false на следующей загрузке.
- [ ] G8.07 — Реализовать CIDR excludes без неявного LAN bypass.
- [ ] G8.08 — Проверить overlap/default routes/endpoint DNS bootstrap.
- [ ] G8.09 — Доказать IPv6 tunnel или явный block.
- [ ] G8.10 — Согласовать domain split DNS/TTL/shared-IP/DoH semantics.
- [ ] G8.11 — Либо реализовать domain split, либо явно исключить из release scope.
- [ ] G8.12 — Проверить network switch/captive portal/other VPN.

## G9. Kill-switch при утверждённом scope

Зависимости: OPEN-03 + G8.

Выход: WFP policy и recovery.

Критерий выхода: Leak/crash/reboot/uninstall acceptance, без снижения защиты ОС.

- [ ] G9.01 — Зафиксировать when disconnected/when crashed/when signed-out semantics.
- [ ] G9.02 — Спроектировать minimal WFP rules и exemptions issuer/endpoint.
- [ ] G9.03 — Не считать native WFP flags полноценным persistent kill switch.
- [ ] G9.04 — Реализовать owned rule identity и atomic apply.
- [ ] G9.05 — Проверить IPv4/IPv6/DNS leaks при crash и reconnect.
- [ ] G9.06 — Проверить boot без сети и captive portal.
- [ ] G9.07 — Проверить безопасный admin recovery.
- [ ] G9.08 — Проверить uninstall/rollback без оставшихся filters.

## G10. Tauri GUI, branding и MSI

Зависимости: G6 + G8; G9 если включён.

Выход: Фирменный desktop и единый установщик.

Критерий выхода: Clean VM install/silent/upgrade/repair/uninstall.

- [ ] G10.01 — Получить brand assets либо использовать временные нейтральные.
- [ ] G10.02 — Создать минимальный Tauri shell с CSP/capabilities.
- [ ] G10.03 — Сделать email/OTP UX и ясные auth errors.
- [ ] G10.04 — Сделать single state status/connect/cancel в GUI и tray.
- [ ] G10.05 — Сделать single-instance/close-to-tray/GUI exit отдельно от signout.
- [ ] G10.06 — Настроить user logon autostart без elevation.
- [ ] G10.07 — Сделать service pipe adapter без raw secret в JS.
- [ ] G10.08 — Создать WiX ServiceInstall/Control и protected directories.
- [ ] G10.09 — Включить pinned runtime и third-party notices.
- [ ] G10.10 — Выбрать WebView2 offline/managed strategy.
- [ ] G10.11 — Проверить GPO/Intune silent install и MSI return codes.
- [ ] G10.12 — Проверить rollback/repair/upgrade/downgrade/uninstall/reboot.

## G11. Security, release engineering и регрессия

Зависимости: G7–G10.

Выход: Release candidate с evidence.

Критерий выхода: Critical/high findings закрыты; release gates независимы от CI green.

- [ ] G11.01 — Проверить threat model SYSTEM/GUI/files/server/update.
- [ ] G11.02 — Проверить IPC fuzzing/oversize/slow clients/privilege escalation.
- [ ] G11.03 — Проверить token/OTP/UUID redaction, rotation, log export.
- [ ] G11.04 — Проверить actual dependencies licenses + vulnerability audit.
- [ ] G11.05 — Проверить build provenance/runtime SHA/SBOM/license package.
- [ ] G11.06 — Настроить signing workflow с защищённым key storage.
- [ ] G11.07 — Подписать EXE до MSI и проверить timestamp/chain.
- [ ] G11.08 — Прогнать A01–A60 по применимым targets.
- [ ] G11.09 — Отдельно Win10 approved edition/ESU/LTSC regression.
- [ ] G11.10 — Сформировать release manifest/checksums и known limitations.

## G12. Пилот, эксплуатация и сдача

Зависимости: G11; approved rollout window.

Выход: Проверенный release kit и rollback.

Критерий выхода: 2–3 устройства → до20; отчёт и handover.

- [ ] G12.01 — Подготовить admin/user/troubleshooting/recovery guides.
- [ ] G12.02 — Провести staged rollout на 2–3 добровольных устройства.
- [ ] G12.03 — Проверить boot/connect/revoke/upgrade на пилоте.
- [ ] G12.04 — Провести 20-user нагрузку и revoke isolation.
- [ ] G12.05 — Зафиксировать throughput/latency/CPU/RAM/error metrics.
- [ ] G12.06 — Согласовать достигнутые SLO и остаточные ограничения.
- [ ] G12.07 — Развернуть утверждённый signed MSI через IT.
- [ ] G12.08 — Проверить rollback и запасной канал доступа.
- [ ] G12.09 — Передать private source/locks/CI/license/evidence/release kit.
- [ ] G12.10 — Зафиксировать приёмку по фактам, а не по внешнему виду.

## Автоматизация и то, что требует внешнего участия

| Процесс | Автоматизация | Внешняя зависимость |
|---|---|---|
| Local core checks | tools/dev.py test | pinned Rust и network только для первого fetch |
| Open-source access | manifest + official URLs/SHA; fetch-runtime | upstream availability; unknown license blocks reuse |
| Windows build | workflow matrix + MSVC targets | private GitHub remote; runner |
| Windows VM | Fusion + VM provisioning/runbook | Windows image, EULA/account setup, ARM vs x64 target |
| TUN E2E | PowerShell evidence, snapshot, controlled fault injection | Windows admin guest + staging VPN credentials via secure channel |
| Backend | contract tests + migration/reconcile scripts | actual issuer source, staging approval |
| Signing | isolated signing/release workflow | customer's certificate/provider and legal ownership |
| Pilot | silent MSI + scripted acceptance | IT change window and test users |

## Оценка и работа с риском

Вместо прежних 4–5 недель использовать три оценки после G5: core/network, backend integration, packaging/security/pilot. Время ожидания Windows/staging/signing учитывать отдельно. После каждого POC обновлять неизвестные. Отсутствие machine, signing или backend нельзя скрыть успешными mocks.

Финальный критерий: единый MSI реально устанавливается, VPN работает для системного трафика, boot без GUI выполняет policy, revoke блокирует нужное устройство, crash/disconnect/uninstall не ломают сеть; исходники/лицензии/отчёты переданы. Ни один текущий scaffold не называется release candidate.
