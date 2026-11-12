# Инженерный аудит — 12.11.2026

## Что реально существует

VERIFIED: прочитаны SPEC.md, все 9 Markdown файлов исходного пакета (1291 строка). SPEC.md и SPEC_ORIGINAL.md побайтно совпадают. В исходном каталоге не было Cargo workspace, исходников приложения, installer, CI или git repository. Сервер, issuer и admin UI описаны документами, доступом не проверены. CUSTOMER_CONTEXT.md содержит переданный контекст прежнего обсуждения, а не подтверждённые текущим пользователем новые полномочия.

VERIFIED: хост macOS arm64, Node 24.11.0, Python 3.9.6, Xcode CLI tools, Git и авторизованный GitHub CLI. Rust отсутствовал в PATH. Fusion открыт через UI, библиотека виртуальных машин пустая. Факт установленного Fusion не означает наличие Windows.

## Находки и исправления

| ID | Важность | Слабое место | Исправление / проверка |
|---|---|---|---|
| F01 | Critical | OTP endpoints названы SSO/device flow | ADR-0003; mock OTP; настоящий browser OAuth требует provider/endpoints, OPEN-01 |
| F02 | Critical | User DPAPI несовместим с boot SYSTEM | D002–D004; vault службы + ACL; A07/A21 |
| F03 | Critical | Машинный DPAPI ошибочно можно принять за service-only | Ограничить файл и каталог SYSTEM/service SID; обычные users не читают ciphertext; локальный admin вне гарантии |
| F04 | Critical | `/verify` отдаёт посторонний API secret | Отдельный VPN namespace/adapter; не менять старый контракт; legacy endpoint не вызывать клиентом до согласования |
| F05 | Critical | Подпись не покрывает expiry | Подписывать все метаданные + payload; raw signed bytes contract; A16–A20 |
| F06 | High | Подписанный JSON не определяет каноникализацию | ADR-0004: base64url exact bytes, strict parse, no duplicates; Python/Rust vector |
| F07 | High | Нет device binding / anti rollback | enrollment_id, context, version, key_id, bounded lease; persistent digest high-water mark |
| F08 | High | Read-only ProgramData открывает UUID другим пользователям | Service-only profile/vault/log ACL; GUI получает только безопасный status DTO |
| F09 | Critical | Named pipe JSON SID может подделываться | Windows transport token, owner SID, session checks, REJECT_REMOTE_CLIENTS, FIRST_PIPE_INSTANCE; A22–A24 |
| F10 | Critical | Встроенный TUN принимается за готовый VPN | Закреплён v26.11.9; отдельный POC реальных egress/DNS/IPv6/routes/crash |
| F11 | High | GUI-first roadmap откладывает главный риск | G5 выполняется после минимального ядра/контрактов, до разработки полного UI |
| F12 | Critical | Crash cleanup не имеет владельца | Xray owns TUN changes; service журналирует только собственные дополнительные изменения; watchdog/reboot recovery; не восстанавливать весь stale snapshot |
| F13 | High | Full tunnel не определяет IPv6 | Dual-stack или явный блок IPv6; запрещён silent IPv6 direct; A31 |
| F14 | High | Domain exclude обещан как список routes | До DNS/TTL алгоритма nonempty domains отклонять; не маскировать неподдерживаемую функцию |
| F15 | High | LAN bypass может открыть слишком широкую сеть | Только подписанные CIDR; никакого автоматического RFC1918 bypass |
| F16 | High | Kill switch имеет неоднозначную семантику | OPEN-03; fail-closed crypto не равен fail-closed network; без реализации kill_switch=true отклонять |
| F17 | Critical | Dynamic RemoveUser приравнен к прекращению активных потоков | Отдельный тест active TCP/UDP + new connections; SLA отзыва не подтверждён; при необходимости серверная session termination |
| F18 | High | Dynamic AddUser теряется при restart | Durable enrollment registry + outbox/reconciler; порядок revoke-first; A43 |
| F19 | High | GET создаёт server state без контракта идемпотентности | Регистрация до публикации профиля; unique enrollment key; transaction + retry; без дублирования UUID |
| F20 | High | Offline cache может работать бесконечно | Подписанный hard expiry; на expiry остановить tunnel; last-good только внутри lease |
| F21 | High | Админский push обещан без канала | Poll bundle с version/ETag; urgent deadline отдельно; push не обещать |
| F22 | High | «Без admin install», SmartScreen и размер заявлены неточно | MSI под IT/admin; signer не гарантирует reputation; размер измерять |
| F23 | High | macOS или Windows ARM могут ошибочно закрыть x64 E2E | Отдельные targets/evidence; Fusion ARM — дополнительный стенд, x64 runner + x64 E2E обязательны |
| F24 | High | RustVPN заявлен MIT, но нет LICENSE | Проверен commit 6a942e4…; tree не содержит LICENSE/COPYING/NOTICE. Reuse запрещён до разрешения правообладателя |
| F25 | Medium | Docs/examples Xray названы просто примерами | Фактически CC-BY-SA-4.0; читать как reference, не копировать текст в proprietary code |
| F26 | High | «Всё последовательно, каждый этап с разрешения» блокирует автономность | DAG независимых работ; повторные approvals для локального кода не нужны по D001 |
| F27 | High | 4–5 недель без стенда/сертификата выдаются за срок | План по выходным критериям и внешним зависимостям; оценка не является обещанием |
| F28 | High | Отсутствуют recovery при clock rollback, disk full, power loss | Persist version+digest+last trusted time атомарно; bounded skew; fault tests |

## Репозитории и воспроизводимость

Проверены 10 upstream через GitHub API; сохранены commit, archived, pushed_at, license paths и release metadata в `docs/evidence/upstreams.json`. Все проверенные репозитории не archived; дата push не является доказательством качества сопровождения. Свежий `main` не используется как release pin. Xray release candidate: v26.11.9, commit в runtime-pins.json, Windows ZIP SHA-256 из официального release API. Официальный Wintun 0.14.1 ZIP скачан, hash сверен с wintun.net, лицензия извлечена из фактического ZIP. DLL ещё не включена в продукт. Для Rust dependencies проверяется фактическое разрешённое Cargo дерево, отдельно от лицензий repository root.

## Вывод

Документ v1 хорошо перечислял риски, но оставлял слишком много неисполняемых требований и остановок. Новая спецификация отделяет требование от предлагаемого контракта и от доказанного результата, сводит remote config к одному атомарному bundle, переносит TUN POC на критический путь и даёт одну команду локальной проверки. «Идеальное» ТЗ не означает отсутствие внешних зависимостей: Windows, staging, server code и signing всё ещё необходимы для конечной приёмки.

Источники: [Xray TUN](https://github.com/XTLS/Xray-core/tree/v26.11.9/proxy/tun), [Tauri MSI](https://v2.tauri.app/distribute/windows-installer/), [DPAPI](https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata), [Xray API](https://xtls.github.io/en/config/api.html), [Wintun](https://www.wintun.net/), [RustVPN tree](https://github.com/Shurubtsov/rustvpn/tree/6a942e40624ea4bd2667f994a70aff62a3f2c152). Live infrastructure не проверялась.
