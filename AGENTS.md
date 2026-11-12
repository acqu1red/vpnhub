# AGENTS.md — обязательные правила для Codex в проекте ScrumeVPN

## Контекст

Продукт — корпоративный VPN-клиент Windows 10/11 x64 по `SPEC_ORIGINAL.md` и `CODEX_MASTER_SPEC.md`. Заказчик хочет **сборку, а не настройку VPN**. Автор пишет код на macOS; для MSI и Windows SCM/TUN нужны Windows CI и E2E.

## Порядок чтения (перед любым изменением)

1. `AGENTS.md` (этот файл)
2. `SPEC_ORIGINAL.md` (не менять; черновик от заказчика)
3. `CODEX_MASTER_SPEC.md` (инженерная декомпозиция)
4. `docs/CUSTOMER_CONTEXT.md` (что действительно сказал заказчик)
5. `docs/OPEN_DECISIONS.md` (вопросы, которые нельзя решить за заказчика)
6. `docs/UPSTREAMS_AND_LICENSES.md` и `docs/ACCEPTANCE_MATRIX.md`

## Правила поведения

- Текущий пользователь 12.11.2026 явно поручил аудит, переработку ТЗ, большой roadmap **и начало реализации**. Старое ограничение первого прохода «audit only» отменено этим запросом. После аудита выполнять локальную реализацию и проверки без повторного согласования каждого этапа. Production deployment требует отдельного допуска.
- Любые требования помечать `SOURCE`, `PROPOSAL`, `OPEN`, `VERIFIED` (верификация — только по доказательству, не по README).
- Не считать то, что компилируется, тем, что работает: отдельно `macOS unit`, `Windows CI`, `Windows E2E`, `staging integration`.
- Не использовать GPL VPN client code/ассеты: Amnezia/NekoRay/Throne/Hiddify/v2rayN и др. `mullvad/mullvadvpn-app` — reference only. Каждую зависимость и копируемый файл проверить по лицензии/NOTICE.
- `Shurubtsov/rustvpn` использовать только после license audit; его SOCKS/HTTP proxy sidecar не равен системному TUN и не заменяет SYSTEM service.
- Xray — upstream pinned binary; TUN предпочитать встроенный Xray Windows; Wintun только официальный prebuilt DLL с его отдельной лицензией.
- **Никогда** не помещать в репозиторий `at-`, `as-`, `sk-`, OTP, сертификат PFX, приватные ключи issuer/Xray/Ed25519, production конфиги/скриншоты/logs. Все примеры используют `example.invalid`.
- Никаких server changes (включая `employees.json`, Xray config/API) без письменного разрешения на staging/production.
- Не отключать TLS verification, firewall, UAC, Code Integrity, Defender ради тестов. Проверять сеть после любого эксперимента с DNS/route.
- GUI работает с правами пользователя. SYSTEM service принимает только авторизованные команды через secure named pipe; не доверять JSON `sid` клиента. GUI не запускает Xray напрямую.
- Установка service требует admin/delegated context; не обещать обычному сотруднику unprivileged install.
- Все изменения с тестами, фиксированными версиями, обработкой ошибок, короткими коммитами и обновлением `docs/TRACEABILITY.md`.
- Не обходить `OPEN` вопросы выдуманными утверждениями: auth browser vs email OTP, user DPAPI vs SYSTEM, kill-switch, signed policy endpoint, lifetime/expiry, multi-user, cert signing.
- Фокус первой техпроверки: **настоящий Xray TUN на Windows x64** с корректным connect/disconnect/crash cleanup; не тратить недели на дизайн до TUN POC.

## Формат отчёта по каждому этапу

```text
Этап: G<n> / задача
Изменено: <файлы>
Реализовано: <проверенный функционал>
Тесты: <команды, платформа, pass/fail>
Не проверено: <не проводившиеся тесты>
Безопасность/лицензии: <что проверили>
Открытые вопросы: <ID>
Следующий маленький шаг: <1–3 действия>
```

## Первая инструкция

Прочитать `CONTINUE_HERE.md`, `docs/SESSION_HANDOFF.md`, актуальные `CODEX_MASTER_SPEC.md`, `docs/DECISIONS.md`, `docs/EXECUTION_PLAN.md` и `docs/STATUS.md`. Продолжать ближайшую незаблокированную задачу. Не создавать вымышленные test results или completed status. Предлогинное подключение после первоначальной авторизации и service-only vault утверждены пользователем; см. DECISIONS.md. SOURCE 13.11.2026: попутно сохранять весь контекст в проекте для будущего перехода на Windows. После этапа обновлять handoff/status/evidence; новые решения добавлять в DECISIONS. Не оставлять единственную копию важного результата в чате или ignored artifacts.
