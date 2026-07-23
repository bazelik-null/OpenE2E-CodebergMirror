# Welcome and General
welcome-header = Интерфейс OpenE2E CLI
help-tip = Введите 'help' для доступных команд
copyright-notice =
    | Copyright (C) 2026 bazelik-dev
    |
    | Эта программа является свободным программным обеспечением: вы можете распространять её и/или изменять
    | в соответствии с условиями Универсальной общественной лицензии GNU, опубликованной
    | Фондом свободного программного обеспечения, либо версии 3 лицензии,
    | либо (по вашему выбору) любой более поздней версии.
invalid-command = Неверная команда/аргумент. Введите 'help' для доступных команд.
exiting = Завершение приложения...
shutting-down = Отключение приложения...
language-changed = Язык изменен на { $language }

# User Management
creating-user = Создание пользователя...
user-created = Пользователь '{ $username }' успешно создан
user-deleted = Пользователь '{ $username }' удален
user-selected = Пользователь '{ $username }' выбран
no-user-selected = Пользователь не выбран
no-users-found = Пользователи не найдены
logged-out = Выход выполнен

# Session Management
session-type-title = Тип сеанса:
inbound-session = входящий
outbound-session = исходящий
creating-session = Создание сеанса...
session-created = Сеанс '{ $session_name }' успешно создан
session-deleted = Сеанс '{ $session_name }' удален
session-selected = Сеанс '{ $session_name }' выбран
session-closed = Сеанс закрыт
no-sessions-found = Сеансы не найдены
unknown-session-type = Неизвестный тип сеанса

# Session Creation
generating-keys = Генерируем ваши ключи...
share-keys = Поделитесь этим с другой стороной:
paste-other-keys = Вставьте ключи другой стороны:
paste-init-message = Вставьте их инициализирующее сообщение:
session-established = Сеанс установлен! Отправьте инициализирующее сообщение для завершения:

# Help Messages
section-user-management = Управление пользователями
section-session-management = Управление сеансами
section-chatting = Чат
help-exit = Выход из приложения
help-help = Показать эту справку
help-user-new = Создать нового пользователя
help-user-delete = Удалить пользователя
help-user-list = Список всех пользователей
help-user-login = Вход
help-user-logout = Выход
help-session-new = Создать новый сеанс
help-session-delete = Удалить сеанс
help-session-list = Список всех сеансов
help-session-open = Открыть сеанс
help-session-close = Закрыть сеанс
help-encrypt = Зашифровать текст
help-decrypt = Расшифровать текст
help-history = Показать историю
help-lang = Изменить язык интерфейса (en/ru)
help-tip-quotes = Совет: Вы можете использовать "кавычки" для написания имен с пробелами. Не требуется для шифрования и расшифровки, так как все аргументы объединяются в один.
help-available-commands = Доступные команды

# Encryption/Decryption
encrypt-label = Зашифровать текст
decrypt-label = Расшифровать текст
history-label = История сообщений
