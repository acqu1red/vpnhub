# Реестр фактического reuse

Собственный код написан в проекте; чужие VPN source/UI/assets не копировались. Cargo dependencies: exact versions/checksums в Cargo.lock, actual licenses в inventory/notices. Audit reference metadata не являются reuse исходников.

| Source | Pin | Файлы | License | Изменения / причина |
|---|---|---|---|---|
| Cargo registry | Cargo.lock | Crates as dependencies | Actual package LICENSE/NOTICE | Без изменений; serialization/crypto/SCM |
| Official Xray release | upstreams.lock.json | Downloaded xray.exe/LICENSE вне git | MPL-2.0; binary transitive audit pending | Без изменений; будущий engine |
| Official Wintun ZIP | 0.14.1 / SHA в lock | DLL/license в .cache при fetch | Prebuilt Binaries License | Без изменений; будущий TUN |

Download не означает bundling/установку/Windows execution. RustVPN не использован: LICENSE отсутствует. Финальный список included components формируется из actual MSI, а не из списка исследованных проектов.
