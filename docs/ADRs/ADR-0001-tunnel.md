# ADR-0001: native Xray TUN, чистая оболочка

Статус: PROPOSAL, принят как направление локальной разработки; Windows feasibility OPEN.

Использовать официальный закреплённый Xray native TUN с официальным Wintun. Служба не реализует IP stack и не копирует VPN-client code. Tauri создаётся отдельно после сетевого POC. RustVPN не использовать: proxy-sidecar не решает SYSTEM/TUN, лицензия в дереве отсутствует. Собственный Wintun FFI — fallback только после документированного провала native TUN, отдельный ADR и лицензионная проверка. Xray readme/Go config проверены на release pin; опции main могут отличаться. Включение функции в config ещё не доказывает Windows поведение.
