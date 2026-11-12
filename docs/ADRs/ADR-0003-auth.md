# ADR-0003: auth без выдуманного OAuth

Статус: PROPOSAL; production OPEN-01/05.

Email OTP — единственный flow, которому соответствуют исходные endpoints. Использовать OTP в локальном mock. Browser device flow требует реальный authorization server и отдельный контракт device authorization/polling; не имитировать его открытием страницы. Для production предложен versioned `/vpn/v1/auth/request` и `/vpn/v1/auth/verify` adapter к существующему issuer, который возвращает только install_token/enrollment_id. Не вызывать legacy verify, возвращающий сторонний API key, до server contract decision. Старые клиенты не ломать. Client не вызывает localhost revoke; signout локальный, администратор отзывает enrollment на сервере.
