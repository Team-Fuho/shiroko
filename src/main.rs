#[macro_use]
extern crate rocket;

use redis::{Connection, FromRedisValue, RedisError};
use rocket::{
    serde::{json::Json, Deserialize, Serialize},
    State,
};

struct RDConn {
    conn: Connection,
    set: String,
}

#[derive(Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
struct ICnf {
    // https://docs.rs/redis/latest/redis/#connection-parameters
    rdscs: String,
    rdsn: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct IDonator {
    who: String,
    did: i64,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct IErr {
    msg: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
enum IResult<T> {
    Ok(T),
    Err(String),
}
// is this cringe?
fn use_redis(cfg: &ICnf) -> RDConn {
    RDConn {
        conn: redis::Client::open(cfg.rdscs.clone())
            .unwrap()
            .get_connection()
            .unwrap(),
        set: cfg
            .rdsn
            .to_owned()
            .unwrap_or("redkiy".to_string())
            .to_owned(),
    }
}

fn handle_rerr<AnyResponse>(r: RedisError) -> Json<IResult<AnyResponse>> {
    print!("DETAIL: {}\n", r);
    return Json(IResult::Err(format!("panic! {}", r)));
}

macro_rules! assr {
    ( $k: expr ) => {{
        if $k.is_err() {
            return handle_rerr($k.unwrap_err());
        }
    }};
}

#[get("/")]
fn index(cfg: &State<ICnf>) -> Json<IResult<Vec<IDonator>>> {
    let mut conn = use_redis(&cfg);
    let rawr: Result<Vec<String>, redis::RedisError> = redis::cmd("ZRANGE")
        .arg(conn.set)
        .arg(0)
        .arg(9)
        .arg("REV")
        .arg("WITHSCORES")
        .query(&mut conn.conn);
    assr!(rawr);

    let raw = rawr.unwrap();
    let mut ret: Vec<IDonator> = vec![];
    for b in raw.chunks(2) {
        let did = b[1].parse().unwrap();
        if did > 0 {
            ret.push(IDonator {
                who: b[0].to_owned(),
                did: did,
            });
        }
    }
    Json(IResult::Ok(ret))
}

fn influence<T: FromRedisValue>(
    opt: &str,
    conn: &mut RDConn,
    who: String,
    did: i64,
) -> Result<T, RedisError> {
    redis::cmd(opt)
        .arg(conn.set.clone())
        .arg(did)
        .arg(who)
        .query(&mut conn.conn)
}

#[put("/", data = "<dnt>")]
fn put_index(cfg: &State<ICnf>, dnt: Json<IDonator>) -> Json<IResult<i64>> {
    let mut conn = use_redis(&cfg);

    let rawr = influence::<String>("ZINCRBY", &mut conn, dnt.who.to_owned(), dnt.did);
    assr!(rawr);

    Json(IResult::Ok(rawr.unwrap().parse().unwrap()))
}

#[post("/", data = "<dnt>")]
fn post_index(cfg: &State<ICnf>, dnt: Json<IDonator>) -> Json<IResult<u16>> {
    let mut conn = use_redis(&cfg);

    let rawr = influence::<u16>("ZADD", &mut conn, dnt.who.to_owned(), dnt.did);
    assr!(rawr);

    Json(IResult::Ok(rawr.unwrap()))
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Merge {
    from: String,
    to: String,
}

#[patch("/", data = "<dnt>")]
fn patch_index(cfg: &State<ICnf>, dnt: Json<Merge>) -> Json<IResult<u16>> {
    let mut conn = use_redis(&cfg);

    let fetch: Result<String, RedisError> = redis::cmd("ZSCORE")
        .arg(conn.set.clone())
        .arg(dnt.from.to_owned())
        .query(&mut conn.conn);

    assr!(fetch);
    let score_left = fetch.unwrap().parse::<i64>().unwrap();

    let rawr = influence::<u16>("ZADD", &mut conn, dnt.to.to_owned(), score_left);
    assr!(rawr);

    let prune = influence::<u16>("ZADD", &mut conn, dnt.from.to_owned(), 0);
    assr!(prune);

    Json(IResult::Ok(rawr.unwrap()))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, put_index, post_index, patch_index])
        .attach(rocket::fairing::AdHoc::config::<ICnf>())
}
