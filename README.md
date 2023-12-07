## Verbena

### Introduction

This project is a demonstration project.
When developing the backend, I used rust 1.73.0.
When developing the frontend, I used angular 16.1.7.


### Frontend build

```bash
$ cd ~/Projects/verbena/frontend/
$ npx ng serve -o --port 4250 --proxy-config proxy.conf.json
```

```bash
http://localhost:4250/login
```

### Backend build

```bash
$ cd ~/Projects/verbena/backend/
$ cargo build
$ cargo run
```

```bash
http://127.0.0.1:8080/ind/login
```
