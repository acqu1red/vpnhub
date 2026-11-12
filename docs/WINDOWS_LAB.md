# Windows lab и VMware Fusion

SOURCE: пользователь поручил использовать VMware Fusion. VERIFIED: host macOS arm64; Создана ScrumeVPN Lab: Windows11 Pro ARM с Microsoft, Secure Boot/vTPM,4CPU,6GiB RAM,128GiB sparse,NAT. Windows/OOBE и VMware Tools установлены; после перезагрузки применяются обновления. Текущее состояние уточнять в STATUS.md.

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

## Готовый ARM64 payload с текущего Mac

`python3 tools/build-windows-lab.py --iso` создаёт `artifacts/windows-lab` и `artifacts/ScrumeVPN-Lab-ARM64.iso`. Rust1.99.1 target aarch64-pc-windows-gnullvm + Zig0.15.2 (официальный архив, pinned SHA256). Native PE ARM64 сборка успешна, Windows runtime result отдельно. Это дополнительный lab build; целевой MSVC x64 release не заменяет.

1. После обновлений Windows сохранить clean snapshot. В Fusion Virtual Machine → CD/DVD → Choose Disc Image выбрать lab ISO. Host folders не расшаривать.
2. В Windows active console user открыть `run-native-tests.cmd` с CD, без elevation. Все4 tests должны завершиться:3 DPAPI primitive и1 native IPC test. Launcher сохраняет log в guest TEMP и оставляет результат на экране. Наличие работающей ScrumeVPNSvc блокирует pipe test; выполнять до SCM smoke.
3. Pipe test проверяет real token SID/session, отказ второго listener, reconnect, oversize и idle stop. Он `ignored` в обычном CI, поскольку hosted/service session не является active console. Сам test executable запускать с `--include-ignored --test-threads=1`.
4. DPAPI roundtrip/corruption/bounds **не** доказывают service-only ownership: A21 требует будущего filesystem vault + ACL + second user/cross-machine.
5. В elevated guest PowerShell: `powershell -NoProfile -File D:\service-smoke.ps1 -Binary D:\ScrumeVPNSvc.exe -DisposableLab` (уточнить CD letter). Только новый disposable lab: скрипт откажет при существующих service/directory, установит temporary Manual LocalSystem status service под защищённым Program Files, проверит status/stop и удаление SCM. PASS выводится только после cleanup. Бинарник остаётся для inspection; для повторного чистого прогона использовать snapshot.
6. Если ExecutionPolicy запрещает локальный ps1 — остановиться и согласовать штатное signing/review с администратором стенда. Не применять Bypass/Unrestricted, не отключать Defender/UAC. Скрипт ещё не был исполнен; сборка EXE его не проверяет.
7. Сохранить native logs, binary SHA256 из build-manifest и итог SCM JSON. До просмотра actual results статусы NOT_RUN. Только затем расширять тесты до anonymous/remote/second user и сетевых экспериментов.

Encrypted vmrun требует пароль VM. Не извлекать его из Keychain и не передавать в чат/журналы/CLI argv. Guest secrets отсутствуют в payload. На текущем этапе не настраивать дополнительный remote login или host home sharing ради удобства автоматизации.
