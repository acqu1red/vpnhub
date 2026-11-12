# Матрица приёмочных тестов ScrumeVPN (предложение для утверждения)

**Принцип:** результат `PASS` возможен только при указанном доказательстве. Фактические текущие результаты — `ACCEPTANCE_REPORT.json` и `STATUS.md`; эта таблица описывает сценарии. Не генерировать фальшивые PASS. Уточнения KPI и политики отмечены как PENDING.

| ID | Сценарий | Доказательство и ожидаемый результат | Где тестировать |
|---|---|---|---|
| A01 | Чистый MSI install | Приложение, Xray, служба, DLL действительно установлены; сервис SCM Running; UI виден | Win11 x64 clean VM |
| A02 | Silent enterprise install | `msiexec /i ... /qn` под администратором/управляемым контекстом устанавливает корректно | Win11 x64 VM |
| A03 | Unprivileged GUI | Обычный сотрудник запускает GUI без UAC; не может править конфиг/службу | Win11 x64 VM |
| A04 | No-admin installation truthfulness | Обычный user без elevated context не может установить сервис — ошибка ясна, install guide корректен | Win11 x64 VM |
| A05 | One instance UI | Второй ScrumeVPN GUI не плодит конфликтов; активирует существующее окно | Win11 x64 VM |
| A06 | Tray and close | close-to-tray, menu connect/disconnect/open/exit отражают сервисную state | Win11 x64 VM |
| A07 | Boot recovery | Service стартует без GUI; интерфейс после логина получает честный статус; autoconnect в approved mode | Win11 x64 reboot |
| A08 | Valid email OTP | Known employee получает код; доступ только после verify | Mock/staging issuer |
| A09 | Unknown employee | Неизвестный email → отказ, нет выдачи UUID | Mock/staging issuer |
| A10 | Invalid code | Несовпадающий OTP не подтверждает enrolment; ограничения попыток соблюдены | Mock/staging issuer |
| A11 | Resend limiter | 60 секунд/TTL 600 секунд/лимит 5 попыток — как реально подтверждено | Mock/staging issuer |
| A12 | `sk-…` no leak | Не хранится/логируется/попадает во frontend/event telemetry; серверный контракт согласован | Code review + tracing |
| A13 | Per-device UUID | Два enrolment у одного email дают разные UUID | Staging issuer |
| A14 | Idempotent profile | Два `GET /vpn/profile` одного enrolment не добавляют дубли Xray | Staging issuer/Xray |
| A15 | Concurrent enrollment | 20 одновременных enrolment не портят `employees.json`/state | Staging issuer |
| A16 | Valid signature | Rust принимает тестовый Python SignedEnvelope; fixtures byte-exact | Unit tests macOS/Windows |
| A17 | Tampered profile | Изменение server/UUID/port/MTU/expiry → reject | Unit tests |
| A18 | Wrong key/replay | Подпись другим key / старый policy_version → reject | Unit tests |
| A19 | Expired profile | Нет нового подключения с истёкшим/просроченным профилем | Unit tests + Windows |
| A20 | Policy invalid | Изменённая policy не применяется; last-good только в согласованных expiry пределах | Unit tests |
| A21 | DPAPI ownership | Неавторизованный пользователь не расшифровывает токен; перенос на другой ПК по политике | Windows x64 |
| A22 | IPC authorized user | `status`/`connect` доступны только разрешённому интерактивному пользователю | Windows x64 |
| A23 | IPC unauthorized user | SID spoofing/pipe cross-user отказ, нет elevation | Windows x64 |
| A24 | IPC malformed payload | Oversize JSON, unknown method, invalid types отвергнуты без crash | Unit/integration |
| A25 | Xray spawn ownership | Xray запускает service, GUI не запускает его с SYSTEM и не передаёт raw args | Windows x64 |
| A26 | Xray signature/integrity | Проверена версия/хэш official release; исправленный не доверенный binary не используется | Build + Windows |
| A27 | TUN created | Официальный Wintun создал TUN, без повреждения других адаптеров | Windows x64 |
| A28 | Real tunnel | VPN публичный egress IP/трафик подтверждён, не только прокси браузера | Windows x64 staging |
| A29 | Full route loop prevention | Маршрут к серверному endpoint не уходит в сам туннель | Windows x64 |
| A30 | DNS via policy | DNS-адреса применяются, lookup работает, утечки оценены | Windows x64 |
| A31 | IPv6 path | IPv6 проходит/блокируется согласно согласованной политике; отсутствие случайного обхода | Windows x64 |
| A32 | Split CIDR exclude | Трафик к исключённым CIDR обходит VPN, остальные идут по правилам | Windows x64 |
| A33 | Split domain exclude | Только при утверждённой семантике и доказанной реализации с DNS/TTL | Windows x64 |
| A34 | LAN access | Принтер/NAS доступен или запрещён согласно policy | Windows x64 LAN |
| A35 | Wi-Fi ↔ Ethernet | Смена сети восстанавливает правильные routes/DNS | Windows x64 hardware |
| A36 | Xray crash | Service обнаруживает crash и восстанавливает/сбрасывает сеть по политике | Windows x64 |
| A37 | Service crash/restart | Не возникает бесконечных процессов/adapter orphan, стабилен SCM status | Windows x64 |
| A38 | Disconnect rollback | Все изменения маршрутов/DNS убраны либо сохранены в policy kill switch | Windows x64 |
| A39 | Network unavailable | Понятный offline status, ограниченные retries, не портится DNS | Windows x64 |
| A40 | REALITY auth failure | Ошибка ключа/SNI/UUID обрабатывается и логируется без секретов | Staging |
| A41 | Single revoke | Один `install_token` удалён из issuer и Xray, другие сессии не прерваны | Staging multi client |
| A42 | Revoke during offline | Реалистичный offline policy не обещает мгновенную доставку, server-side deny enforced | Staging + Windows |
| A43 | Xray restart reconciliation | После рестарта сервера state клиентов восстановлен/соответствует issuer | Staging server |
| A44 | Profile refresh | Default 6h, max 24h, jitter/backoff работают с mock clock | Unit/integration |
| A45 | Profile admin change | Новая валидная политика применяется предусмотренным механизмом без UI edit | Staging |
| A46 | Signout / switch account | Локальные токены очищаются безопасно, состояние корректно | Windows x64 |
| A47 | Redacted logging | No OTP/at/as/sk/private keys; rotation и файл ACL действуют | Unit + Windows |
| A48 | Installer repair/update | Установка версии поверх другой корректна, service и configs сохраняются по policy | Windows x64 VM |
| A49 | Uninstall clean | Нет SYSTEM service, процессов, лишних firewall rules, routes, orphan device | Windows x64 VM |
| A50 | Rollback MSI | Ошибка mid-install не оставляет систему в поломанном состоянии | Windows x64 VM |
| A51 | WebView2 absent | Установка/запуск согласно выбранной стратегии WebView2 | Windows x64 VM |
| A52 | GPO/Intune deploy | Документирован пакет, параметры, elevation и MSI Exit Codes | IT staging |
| A53 | 20-user load | На тестовом сервере 20 enrolment/connections, метрики/узкие места задокументированы | Staging |
| A54 | 20-user revoke isolation | Отзыв одного не сбрасывает чужие сессии | Staging |
| A55 | License compliance | `THIRD_PARTY_NOTICES`, official Wintun license, Xray MPL notices и SBOM предоставлены | Release audit |
| A56 | Signing status | Подпись EXE/MSI проверена либо релиз явно unsigned QA; сертификат не раскрыт | Windows |
| A57 | Source handover | Приватный репо, документация, CI, lockfiles, manifest, release notes включены | Delivery |
| A58 | Windows 10 compatibility | Если утверждён Win10, отдельный прогон на версии/редакции, известен EOL risk | Win10 x64 |
| A59 | DPI/throughput claims | Нет непроверенных обещаний «обходит всё», скорость по замерам | Review |
| A60 | No hidden remote execution | Нет сторонних remote scripts, shell injections, admin bypass | Security review |

## Шаблон отчёта для каждого сценария

```yaml
id: A01
status: NOT_RUN | PASS | FAIL | BLOCKED
platform: Windows 11 Pro x64 [версия]
build_sha: [sha256]
commit: [git sha]
started_utc: [timestamp]
steps: [steps]
expected: [expected]
actual: [observed]
evidence: [logfile / screenshot / automated test path]
reviewer: [person]
notes: [issues]
```

## Определение готовности версии

- **Prototype:** G1–G5 подтверждены, присутствуют mocks, нет production claims.
- **Release candidate:** Windows service + auth + реальный TUN + безопасный disconnect + revoke + MSI; закрыты blockers.
- **Production:** Release candidate + тесты A01–A60, применимые к утверждённому объёму, лицензии и подпись/приёмка; остальное в честно подписанном `KNOWN_LIMITATIONS`.
