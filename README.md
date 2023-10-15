# `shiroko`
`shiroko` is a HTTP microservice for record a leaderboard (in this case, used for a donation board... and wisely hardcoded for it) with a conventional HTTP interface
## Dependency
> - `redis`
## Service
### Bootup
1. Configuration
> *reference: https://rocket.rs/v0.5-rc/guide/configuration/#configuration*

2. Configurable keys
> - `rdscs`/`ROCKET_RDSCS`**\***: Redis connection string *(not include database name? Not confirmed on ACL-partritioned Redis)*. *reference: https://docs.rs/redis/latest/redis/#connection-parameters*
> - `rdsn`/`ROCKET_RDSN`*default=`redkiy`*: Sorted KV set name
> ***\*** field is required* 
## HTTP Specs
## 1. Get a list
- request
```http
GET / HTTP/1.1
```
- response schema
```ts
interface Schema {
  Err?: string;
  Ok: {
    who: string;
    did: number;
  }[];
}
```
## 2. Add amount to entry
- request
```http
PUT / HTTP/1.1
Content-Type: application/json

{
  "who": "string",
  "did": 1000
}
```
- response schema
```ts
interface Schema {
  Err?: string;
  Ok: number;
}
```
## 3. Force set entry
- request
```http
POST / HTTP/1.1
Content-Type: application/json

{
  "who": "string",
  "did": 1000
}
```
- response schema
```ts
interface Schema {
  Err?: string;
  Ok: number;
}
```
## 4. Move amount, then set `0` to user in `from`
```http
PATCH / HTTP/1.1
Content-Type: application/json

{
  "from": "jane",
  "to": "john"
}
```
- response schema
```ts
interface Schema {
  Err?: string;
  Ok: number;
}
```