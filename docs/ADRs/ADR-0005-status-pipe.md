# ADR-0005: первый Windows IPC — status-only

Статус: PROPOSAL реализации G2; cross-check выполнен, native Windows security tests ожидаются.

Первый транспорт предоставляет только `status.get`, без секретов и без network effects. `vpn.connect`/`disconnect` проходят строгий parser, но при отсутствующем owner отвергаются; даже при owner handler возвращает NOT_IMPLEMENTED до появления controller. Работающий SCM не означает подключённый VPN.

## Контроль доступа

- Имя `\\.\pipe\ScrumeVPN-v1`, единственный instance с FIRST_PIPE_INSTANCE. Один handle сохраняется между connections, чтобы не отдавать namespace между запросами.
- DACL: SYSTEM full access, Interactive Users только individual data/attribute rights. Не давать IU FILE_GENERIC_WRITE: он включает FILE_CREATE_PIPE_INSTANCE. Поэтому собственный клиент использует FILE_READ_DATA/WRITE_DATA/WRITE_ATTRIBUTES.
- PIPE_REJECT_REMOTE_CLIENTS; message frames ≤8192 bytes; oversize, trailing data и unknown fields отвергаются. В одной connection один запрос. Deadline1s и nonblocking poll позволяют остановить service даже с зависшим клиентом; FlushFileBuffers не используется.
- После чтения request Windows API ImpersonateNamedPipeClient → OpenThreadToken(query only) → TokenUser/TokenSessionId. Транспортная session сверяется с token session и активной console session. Anonymous/SYSTEM/service accounts и Session0 отвергаются. RevertToSelf выполняется на той же нити до handler/IO; неуспешный revert завершает процесс.
- Первая версия поддерживает только активную физическую console session; RDP/fast switching и multi-user приёмка ещё OPEN. Это ограничение development adapter, не согласованное исключение из конечного ТЗ.
- Owner SID передаётся в core только из защищённого service state; JSON-поля sid запрещены. Пока durable owner store отсутствует, owner=None и все mutation commands закрыты.

## Границы текущей проверки

Core authorization/response tests выполняются на macOS. Win32 FFI cross-check проверяет типы, но не DACL, impersonation semantics, timing и actual peer identity на Windows. `tools/windows/service-smoke.ps1` подготовлен для disposable VM и пока не считается PASS. Требуются отдельные second-user/anonymous/remote/oversize/slow-client/SCM stop tests.

Development client `--status` использует SECURITY_IDENTIFICATION, не передаёт credentials и не выполняет privileged operations. **До добавления auth/secret IPC обязательна проверка подлинности сервера** через SCM PID/защищённый executable identity; доверять одному pipe name недостаточно. Этот клиент нельзя просто расширить передачей токена без закрытия данного пункта.

Unsafe FFI изолирован в scrume-windows; каждый блок содержит обоснование времени жизни/ownership/размера. В core и service unsafe остаётся запрещён. Это не отменяет code review и native regression.

Источник: [Microsoft named pipe access rights](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights), [ImpersonateNamedPipeClient](https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-impersonatenamedpipeclient).
