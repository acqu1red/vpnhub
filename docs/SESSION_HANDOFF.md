# Точка продолжения — 13.11.2026

SOURCE: пользователь просит попутно сохранять весь контекст разработки в проекте,
чтобы в будущем продолжить напрямую на Windows без потери истории и решений.
Начать с CONTINUE_HERE.md. Текущие факты — STATUS.md, а не предположения в старом SPEC.

## Что делаем сейчас

G2: проверяем реальную SYSTEM-службу и IPC до TUN/GUI/MSI. Пользователь выполнил
native adapter tests в Windows11 Pro ARM64 / VMware Fusion: **4 passed,0 failed,
0 ignored,0.15s**. Агент видел итоговую консоль. Evidence:
`docs/evidence/g2-native-arm64-tests.txt`. Это3 DPAPI primitive tests и1 pipe test,
не проверка файлового vault, второго пользователя, boot или Windows x64.

Следующая попытка `service-smoke.ps1` **не запустилась**: Windows сообщает running
scripts is disabled / UnauthorizedAccess. Повтор без `-DisposableLab` тоже был
отклонён. Это ограничение запуска скриптов; служба не была установлена этим тестом.
ExecutionPolicy/UAC/Defender не менялись. Не предлагать Bypass/Unrestricted или
исполнение содержимого запрещённого скрипта через Invoke-Expression.

Собран отдельный native `ScrumeVPNLab.exe --disposable-lab` для того же SCM
smoke. Исходники `apps/lab/src/scm_smoke.rs`, narrow FFI helper
`crates/scrume-windows/src/lab_directory.rs`. Он использует windows-service,
создаёт новую защищённую Program Files папку, проверяет build-pinned SHA256
соседнего ScrumeVPNSvc.exe, устанавливает Manual LocalSystem service, проверяет
UNENROLLED/stop/exit, удаляет service и подтверждает удаление до PASS.
Произвольные пути/команды и автоматическая elevation отсутствуют.
ARM64 EXE успешно слинкован на Mac; runtime нового EXE пока **NOT_RUN**.
Пользователь решил продолжать дальше напрямую на Windows; VMware тесты сейчас
не продолжать без нового указания. Сверять со свежим STATUS после выполнения.

## Следующие действия

1. Собрать native lab через `tools/build-windows.py` на Windows MSVC либо
   `tools/build-windows-lab.py --iso` на текущем Mac ARM64. Builder обязательно
   передаёт SHA256 service в компиляцию runner; обычный cargo build без него
   создаст runner, который откажется выполнять SCM test.
2. В disposable VM с сохранённым snapshot выполнить native SCM smoke из
   elevated terminal: `D:\ScrumeVPNLab.exe --disposable-lab`, уточнив CD letter.
   Получить JSON, hash и cleanup. При ошибке разобраться в ней, не считать PASS.
3. После runtime boundary — vault ACL/atomic state и G5 real Xray TUN на
   разрешённом staging. Не тратить этапы на GUI до сетевого POC.

## Что уже реализовано

- Аудит28 findings, переработанное ТЗ v2,13 gates/140 подзадач,5 ADR.
- Rust core: strict Ed25519 exact-byte envelope, binding/lease/version+digest,
  policy, FSM с cancellation/expiry/revoke, bounded allowlisted IPC.
- Node34 + Python1 public signing vectors, Python↔Rust interoperability.
- SCM scaffold, status-only native pipe/DACL, OS token SID/session extraction,
  pure authorization. Mutations закрыты; enrollment owner пока не установлен.
- Machine DPAPI primitive с zeroizing. **Файлового vault ещё нет.**
- Loopback mock issuer:8 HTTP tests, ephemeral keys, public test fixtures.
- Toolchain/lockfiles, upstream pins/licenses, notices/SBOM, CI config.
- Нет GUI, реального tunnel, HTTP Rust auth adapter, MSI или production release.

Последние полный local проверки и native evidence перечислены в STATUS. Новые
тесты экспорта реально восстанавливают Git history из bundle и проверяют, что
uncommitted работа не теряется молча. MSVC compile с Mac не равен Windows linking.

Последний local suite: **29 Rust +8 Node HTTP +6 Python =43 PASS**; Windows
x64/ARM64 clippy cross-check PASS;70 Cargo dependencies inventory обновлён.
См. docs/evidence/handoff-*.txt. Отдельно старый native ARM64 runtime4/4 PASS.
Новый native SCM runner ARM64 link PASS, runtime NOT_RUN. Перенос реального
проекта проверяется после commit через export/ZIP hashes/Git clone из bundle;
отчёт о проверке будет рядом с архивом, поскольку сам архив создаётся после commit.

## Решения пользователя, которые нельзя потерять

- VPN подключается при загрузке до входа сотрудника **после первой авторизации**.
- До первоначальной авторизации подключения нет.
- Device token доступен только службе: machine DPAPI обязательно с filesystem ACL.
- Signed server policy позволяет отключить auto_connect.
- Продолжать реализацию по этапам; использовать Fusion для доступных Windows tests.
- Сохранять код и контекст для переноса на Windows. Полный список — DECISIONS.md.

## Стенд и внешние зависимости

Текущий Mac Apple Silicon; VM **ScrumeVPN Lab**, Windows11 Pro ARM64,4 CPU,
6144MB RAM,128GB sparse,NAT,Secure Boot/vTPM,VMware Tools установлены.
Снимок **Windows Tools - native adapters PASS - before SCM** сохранён до установки
service. Evidence: fusion-before-scm-snapshot.json. Старый test ISO подключён
как D:, новый SCM ISO имеет отдельное имя `ScrumeVPN-Lab-ARM64-SCM.iso`.

CUA читает окна/управляет host AX, но guest clicks возвращают noWindowsAvailable.
Ранее были transient timeoutReached и ScreenCaptureKit ошибки. Пользователь
запускал Windows tests вручную. На Windows заново определить доступные инструменты;
не считать ограничение старого Mac обязательным ограничением новой среды.

VM encryption password и Windows password задавал пользователь. Агент их не
извлекал, не сохранял и не передавал в argv. VM остаётся вне Git; при переносе VM
нужен отдельный пользовательский backup. DPAPI ciphertext не является переносимым
между компьютерами токеном; на новом устройстве потребуется штатная авторизация.

Существующий issuer source, staging endpoints и server change authorization не
предоставлены. Legacy production данные из SPEC не являются разрешением на тест.
Нет private remote проекта или code-signing сертификата. GitHub upstream открыты
и доступны без авторизации. Публикацию private проекта пользователь не поручал.

## Где искать подробности

SPEC_ORIGINAL.md — исходный документ, не менять. CODEX_MASTER_SPEC.md — v2.
EXECUTION_PLAN — tasks; TRACEABILITY — требование/код/тест; ACCEPTANCE_REPORT —
полные A01–A60 и partial evidence. OPEN_DECISIONS — неутверждённые product choices.
API_CONTRACT/SIGNING_CONTRACT — proposed v1, существующий backend его не подтвердил.
upstreams.lock.json/Cargo.lock/rust-toolchain.toml — воспроизводимые зависимости.
docs/evidence — сохранённые результаты; artifacts — локально создаваемые файлы.
