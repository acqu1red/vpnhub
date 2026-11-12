# Windows lab и VMware Fusion

SOURCE: пользователь поручил использовать VMware Fusion. VERIFIED: host macOS arm64; Fusion запущен; изначально VM library пуста. Через официальный Fusion workflow начата загрузка Windows11 Professional ARM en-us с Microsoft. Текущее состояние уточнять в STATUS.md.

## Границы стенда

Fusion на Apple Silicon использует ARM64 guest. Для нативного SCM/vault/IPC smoke можно собирать aarch64-pc-windows-msvc. Целевой production x64 build требует x86_64-pc-windows-msvc; x64 user-mode emulation на Windows ARM не заменяет проверку x64 Wintun/драйвера. Поэтому сохраняются отдельные отчёты ARM64 guest и Windows11 x64.

Windows EULA требует принятия пользователем при появлении; создание пользовательского пароля тоже выполняет пользователь. Не обходить OOBE, TPM, Secure Boot, UAC, Defender или требования лицензии ради автоматизации.

## Подготовка

1. Fusion Get Windows from Microsoft, ARM64 Pro image; создать VM с достаточной RAM/disk и NAT (без bridge к корпоративной сети). Не менять network configuration хостового Mac.
2. Завершить официальный install/OOBE, user account и VMware Tools. Если нужен ввод пароля — handoff пользователю, не передавать пароль в чат.
3. Обновить guest, подтвердить `RuntimeInformation.OSArchitecture`, сохранить clean snapshot. Tests с admin context выполняются только внутри disposable VM.
4. Передавать в guest source/build без SPEC_ORIGINAL, secrets и production material. Shared folder ограничивать каталогом тестовых artifacts, не всем home. Настройка host sharing требует отдельной оценки доступа.
5. На guest с Rust/MSVC/Node/Python: `python tools/dev.py test`. Это core/compile test, не VPN.
6. Service scaffold можно установить в VM через SCM только после сборки. Он сообщает RUNNING без VPN; capabilities явно NOT_IMPLEMENTED. До secure pipe/vault он не должен обрабатывать реальные auth/tunnel команды.

## Сетевые испытания G5

Требуются approved staging Xray и test enrollment через secure channel. Не использовать production IP/UUID из старого SPEC. Снять Get-NetRoute/Get-DnsClientServerAddress/Get-NetAdapter и connectivity baseline. Затем start → real egress → DNS/IPv6 → disconnect → forced child crash → forced service crash → reboot → baseline comparison. Сохранять только redacted evidence; routes/DNS могут раскрывать сеть и требуют review перед export.

Recovery управляет только собственными changes. Никогда не импортировать весь старый network snapshot поверх текущей системы и не удалять чужие adapters. Если проверка не проведена, соответствующий A-test NOT_RUN/BLOCKED, а не PASS.
