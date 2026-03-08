# Bun Built-in APIs Cheatsheet

Prefer Bun-native APIs for performance in Bun-first runtimes. If deploying to Vercel Functions, remember `Bun.serve()` is not supported there.

## Fast Filesystem I/O

```ts
const file = Bun.file("./data.json");
const data = await file.json();

await Bun.write("./out.txt", "hello");
```

## HTTP Server (Bun Runtime Only)

```ts
const server = Bun.serve({
  port: 3000,
  fetch() {
    return new Response("ok");
  },
});

console.log(server.port);
```

## WebSocket Server (Bun Runtime Only)

```ts
Bun.serve({
  port: 3000,
  fetch(req, server) {
    if (server.upgrade(req)) return;
    return new Response("upgrade failed", { status: 500 });
  },
  websocket: {
    open(ws) {
      ws.send("welcome");
    },
    message(ws, message) {
      ws.send(String(message));
    },
  },
});
```

## SQLite (bun:sqlite)

```ts
import { Database } from "bun:sqlite";

const db = new Database("my.db");
db.run("create table if not exists t (id integer primary key, name text)");
db.prepare("insert into t (name) values (?)").run("Alice");
const row = db.query("select * from t where name = ?").get("Alice");
```

## Password Hashing

```ts
const hash = await Bun.password.hash("pw");
const ok = await Bun.password.verify("pw", hash);
```

