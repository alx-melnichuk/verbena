## Verbena

### Introduction

This project is a demonstration project.
When developing the backend, I used rust 1.91.
When developing the frontend, I used angular 20.3.17.


### Frontend build and run

```bash
$ cd ~/Projects/verbena/frontend/
$ npx ng serve -o --port 4250 --proxy-config proxy.conf.json
# build release
$ npx ng build --configuration production --base-href /
```
URL to Browser
```bash
http://localhost:4250/login
```


### Backend build

```bash
$ cd ~/Projects/verbena/backend/
$ cargo build
# build release
$ cargo build --release
```


### Backend run

```bash
$ cd ~/Projects/verbena/backend/
$ cargo run -p vrb_app
```
URL to Browser
```bash
http://127.0.0.1:8080/ind/login
```


###Parameters for "cargo"
Собрать проект                              $ cargo build      // cargo b
Анализ проекта на ошибки без сборки         $ cargo check      // cargo c
Стартовать проект                           $ cargo run
Создать новый проект                        $ cargo new
Создать новый проект в текущем каталоге     $ cargo init
Собрать проект как релиз (без отладки)      $ cargo build --release
Стартовать все тесты проекта                $ cargo test
Стартовать тесты с "mockdata" и выводом     $ cargo test --features mockdata  -- --show-output
Стартовать тест для "hash_tools.rs"         $ cargo test hash_tools


###Tests for BackEnd

    Tests of the "vrb_authent"        cd ~/Projects/verbena/backend/vrb_authent
  /
$ cargo test -p vrb_authent -F mockdata authentication_test  -- --show-output //26.08.20 (10)
$ cargo test -p vrb_authent -F mockdata user_authent_test    -- --show-output //26.08.20 (42)
$ cargo test -p vrb_authent -F mockdata user_registr_test    -- --show-output //26.08.20 (25)
$ cargo test -p vrb_authent -F mockdata user_recovery_test   -- --show-output //26.08.20 (20)

$ cargo test -p vrb_authent -F mockdata                      -- --show-output //26.08.20 (97)

    Tests of the "vrb_common"         cd ~/Projects/verbena/backend/vrb_common
  /
$ cargo test -p vrb_common    alias_path          -- --show-output            //26.08.20 (7)
$ cargo test -p vrb_common    api_error           -- --show-output            //26.08.20 (12)
$ cargo test -p vrb_common    crypto              -- --show-output            //26.08.20 (16)
$ cargo test -p vrb_common    file_path           -- --show-output            //26.08.20 (2)
$ cargo test -p vrb_common    validators          -- --show-output            //26.08.20 (24)

$ cargo test -p vrb_common                        -- --show-output            //26.08.20 (61)

    Tests of the "vrb_chats"          cd ~/Projects/verbena/backend/vrb_chats
  /
$ cargo test -p vrb_chats -F mockdata chat_event_ws          -- --show-output //26.08.20 (2)
$ cargo test -p vrb_chats -F mockdata chat_msg_blocked_test  -- --show-output //26.08.20 (31)
$ cargo test -p vrb_chats -F mockdata chat_msg_test_get      -- --show-output //26.08.20 (6)
$ cargo test -p vrb_chats -F mockdata chat_msg_test_post_put -- --show-output //26.08.20 (16)
$ cargo test -p vrb_chats -F mockdata chat_msg_test_delete   -- --show-output //26.08.20 (7)
$ cargo test -p vrb_chats -F mockdata chat_ws_test_base      -- --show-output //26.08.20 (4)
$ cargo test -p vrb_chats -F mockdata chat_ws_test_blck      -- --show-output //26.08.20 (3)
$ cargo test -p vrb_chats -F mockdata chat_ws_test_msg       -- --show-output //26.08.20 (2)
$ cargo test -p vrb_chats -F mockdata chat_ws_test_prm       -- --show-output //26.08.20 (2)

$ cargo test -p vrb_chats -F mockdata                        -- --show-output //26.08.20 (73)

    Tests of the "vrb_profiles"       cd ~/Projects/verbena/backend/vrb_profiles/
  /
$ cargo test -p vrb_profiles -F mockdata profile_test_get    -- --show-output //26.08.20 (8)
$ cargo test -p vrb_profiles -F mockdata profile_test_put    -- --show-output //26.08.20 (43)
$ cargo test -p vrb_profiles -F mockdata profile_test_delete -- --show-output //26.08.20 (10)
?? test_put_profile_c_with_old1_new1 ... FAILED
$ cargo test -p vrb_profiles -F mockdata                     -- --show-output //26.08.20 (61)

    Tests of the "vrb_streams"        cd ~/Projects/verbena/backend/vrb_streams
  /
$ cargo test -p vrb_streams -F mockdata stream_test_get1     -- --show-output //26.08.20 (26)
$ cargo test -p vrb_streams -F mockdata stream_test_get2     -- --show-output //26.08.20 (14)
$ cargo test -p vrb_streams -F mockdata stream_test_post_delete -- --show-output 26.08.20 (26)
$ cargo test -p vrb_streams -F mockdata stream_test_put      -- --show-output //26.08.20 (37)

$ cargo test -p vrb_streams -F mockdata                      -- --show-output //26.08.20 (103)

    Tests of the "vrb_tools"          cd ~/Projects/verbena/backend/vrb_tools
  /
$ cargo test -p vrb_tools hash_tools              -- --show-output            //26.08.20 (7)
$ cargo test -p vrb_tools token_coding            -- --show-output            //26.08.20 (7)
Просроченный (expired) token только если более 60 сек.
  /cdis
$ cargo test -p vrb_tools coding                  -- --show-output            //26.08.20 (42)
  /loading
$ cargo test -p vrb_tools dynamic_image           -- --show-output            //26.08.20 (8)

$ cargo test -p vrb_tools                         -- --show-output            //26.08.20 (57+6)



### Backend build release

```bash
$ cd ~/Projects/verbena/backend/
$ cargo build
$ cargo run -p vrb_app
```
