# Hotel Room Service

Experimental use of rust stream as message broker between two multi-instance microservices.

## Code structure

- [Protocol Library (proto)](./proto): Shared cargo package that defines the event and helper library to
  consume and process events.
- [Reception Service (reception)]: Producer that produces event.
- [Staff Service (staff)]: Consumer of events produced by reception Service.
- [docker-compose.yml]: runs an instance of redis-stack on default ports.

## Usage

```bash
# Ensure redis server + run an instance of reception and staff and watch source code
pnpm run dev
# Run an instance of staff with a name (or redis group name) pass nothing to use auto generated id
pnpm run dev:staff name
# Run an instance of reception
pnpm run dev:reception
```

## How it works

- [Reception Service (reception)] auto generated event on interval (5-10 seconds) and pushes to redis stream with id `jobs`.
- [Staff Service (reception)] spawn configurable cocurrent consumers to consume unseen or non acknowledge events.


## TODO:

- [ ] Fix auto-claim handling

[Staff Service (staff)]: ./staff
[Protocol Library (proto)]: ./proto
[Reception Service (reception)]: ./reception
