# verbena

- - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

$ cd /home/aleksey/Projects/verbena/frontend

Собрать и стратовать приложение frontend
```bash
$ npx ng serve -o --port 4250
```
http://127.0.0.1:4250/
- - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

Собрать проект для релиза
# удалить все файлы в каталоге ../backend/static/
```bash
$ npx ng build --configuration=production --base-href / --output-path ../backend/static/
```
- - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

$ cd /home/aleksey/Projects/verbena/backend

$ cargo build
$ cargo run

http://127.0.0.1:8080/api/healthchecker
http://127.0.0.1:8080/404

- - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

