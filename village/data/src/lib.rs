//! The `supa` resource. The world publishes one numbered request at a time in
//! `hud.ask`; Contract calls `supa(hud.ask)`; this turns it into a Supabase
//! call and turns the reply into one tab-separated line the world reads from
//! its live `net` argument. The world never sees HTTP or JSON.
//!
//! Ask:   `id \t kind \t token \t args…`   (kind: join, signin, state, save)
//! Reply: `id \t ok \t fields…` | `id \t conflict \t version \t text` |
//!        `id \t auth` (sign in again) | `id \t offline` | `id \t error \t why`
//!
//! The publishable key is public by design (Supabase row level security guards
//! the data); the service key never leaves the server.
use exact_runner::{Answer, DataError, DataSource, Outcome, Request, Store, Value};
use serde_json::{json, Value as Json};

pub const URL: &str = "https://lqcecndhhmbifrxbexne.supabase.co";
pub const KEY: &str = "sb_publishable_INxedNXuIKyn3VJZxYpSgg_enBjwiaY";
const GRANTS: &str = "net.fetch https://lqcecndhhmbifrxbexne.supabase.co";

#[derive(Default)]
pub struct Supa;

/// The request an ask makes: (id, path, bearer token, JSON body).
pub fn request(ask: &str) -> Option<(u64, String, String, Json)> {
    let mut f = ask.split('\t');
    let id: u64 = f.next()?.parse().ok()?;
    let kind = f.next()?;
    let token = f.next().unwrap_or("").to_string();
    let args: Vec<&str> = f.collect();
    let (path, body) = match kind {
        "join" => ("/functions/v1/join".to_string(), json!({})),
        "signin" => (
            "/auth/v1/token?grant_type=password".to_string(),
            json!({ "email": args.first()?, "password": args.get(1)? }),
        ),
        "state" => ("/rest/v1/rpc/get_state".to_string(), json!({})),
        "bug" => (
            "/rest/v1/bug_reports".to_string(),
            json!({ "text": args.first()?, "save": args.get(1).copied().unwrap_or("") }),
        ),
        "mkcode" => ("/functions/v1/restore".to_string(), json!({ "action": "make" })),
        "claim" => ("/functions/v1/restore".to_string(), json!({ "action": "claim", "code": args.first()? })),
        "save" => {
            let base: i64 = args.first()?.parse().ok()?;
            let contract: i64 = args.get(1)?.parse().ok()?;
            let events: Vec<i64> = match args.get(2) {
                Some(s) if !s.is_empty() => s.split(',').map(|n| n.parse().ok()).collect::<Option<_>>()?,
                _ => Vec::new(),
            };
            let text = args.get(3)?;
            (
                "/rest/v1/rpc/put_save".to_string(),
                json!({ "p_text": text, "p_contract": contract, "p_base": base, "p_event_ids": events }),
            )
        }
        _ => return None,
    };
    Some((id, path, token, body))
}

fn clean(s: &str) -> String {
    s.replace(['\t', '\n', '\r'], " ")
}

/// Turns a reply into the world's line.
pub fn normalize(id: u64, kind: &str, status: u16, body: &[u8]) -> String {
    if status == 401 || status == 403 {
        return format!("{id}\tauth");
    }
    // A table insert answers 201 with no body.
    if kind == "bug" && (200..300).contains(&status) {
        return format!("{id}\tok");
    }
    let Ok(j) = serde_json::from_slice::<Json>(body) else {
        return format!("{id}\terror\tHTTP {status}");
    };
    if !(200..300).contains(&status) {
        // A bad password or unknown account at sign-in is "auth" too.
        if kind == "signin" && status == 400 {
            return format!("{id}\tauth");
        }
        let why = j["error"].as_str().or(j["msg"].as_str()).or(j["message"].as_str()).unwrap_or("error");
        if kind == "claim" || kind == "mkcode" || kind == "bug" {
            // Shown to the player as is ("no account has that code").
            return format!("{id}\terror\t{}", clean(why));
        }
        return format!("{id}\terror\tHTTP {status}: {}", clean(why));
    }
    match kind {
        "mkcode" => match j["code"].as_str() {
            Some(c) => format!("{id}\tok\t{}", clean(c)),
            None => format!("{id}\terror\tno code"),
        },
        "join" | "claim" => match (j["email"].as_str(), j["password"].as_str()) {
            (Some(e), Some(p)) => format!("{id}\tok\t{}\t{}", clean(e), clean(p)),
            _ => format!("{id}\terror\tmalformed join"),
        },
        "signin" => match j["access_token"].as_str() {
            Some(t) => format!("{id}\tok\t{}", clean(t)),
            None => format!("{id}\terror\tno token"),
        },
        "state" => {
            let events: Vec<String> = j["events"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter(|e| e["kind"] == "review")
                        .map(|e| format!("{}:{}:{}", e["id"].as_i64().unwrap_or(0), e["n"].as_i64().unwrap_or(0), e["day"].as_i64().unwrap_or(0)))
                        .collect()
                })
                .unwrap_or_default();
            let save = &j["save"];
            match (save["version"].as_i64(), save["text"].as_str()) {
                (Some(v), Some(t)) => format!("{id}\tok\t{v}\t{}\t{}", events.join(","), clean(t)),
                _ => format!("{id}\tok\t0\t{}\t", events.join(",")),
            }
        }
        "save" => {
            if j["ok"] == true {
                format!("{id}\tok\t{}", j["version"].as_i64().unwrap_or(0))
            } else {
                format!(
                    "{id}\tconflict\t{}\t{}",
                    j["version"].as_i64().unwrap_or(0),
                    clean(j["text"].as_str().unwrap_or(""))
                )
            }
        }
        _ => format!("{id}\terror\tunknown"),
    }
}

impl DataSource for Supa {
    fn grants(&self) -> &str {
        GRANTS
    }

    fn query(&mut self, source: &str, _: &[Value]) -> Result<Value, DataError> {
        match source {
            "supa" => Ok(Value::str("")),
            other => Err(DataError::UnknownSource(other.into())),
        }
    }

    fn answer(&mut self, _: &mut Store, source: &str, args: &[Value]) -> Result<Answer, DataError> {
        if source != "supa" {
            return Err(DataError::UnknownSource(source.into()));
        }
        let ask = args.first().and_then(Value::as_str).unwrap_or("");
        if ask.is_empty() {
            return Ok(Answer::Now(Value::str("")));
        }
        let Some((id, path, token, body)) = request(ask) else {
            let id = ask.split('\t').next().unwrap_or("0");
            return Ok(Answer::Now(Value::str(&format!("{id}\terror\tbad request"))));
        };
        let bearer = if token.is_empty() { KEY.to_string() } else { token };
        let _ = id;
        let request = Request::post_json(&format!("{URL}{path}"), &body.to_string())
            .header("apikey", KEY)
            .header("authorization", &format!("Bearer {bearer}"))
            .timeout(15_000);
        Ok(Answer::Later(request))
    }

    fn parse(&mut self, _: &mut Store, source: &str, args: &[Value], outcome: Outcome) -> Result<Answer, DataError> {
        if source != "supa" {
            return Err(DataError::UnknownSource(source.into()));
        }
        let ask = args.first().and_then(Value::as_str).unwrap_or("");
        let mut f = ask.split('\t');
        let id: u64 = f.next().and_then(|n| n.parse().ok()).unwrap_or(0);
        let kind = f.next().unwrap_or("");
        let line = match outcome {
            Outcome::Response(r) => normalize(id, kind, r.status, &r.body),
            Outcome::Failed { .. } => format!("{id}\toffline"),
            _ => format!("{id}\toffline"),
        };
        Ok(Answer::Now(Value::str(&line)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asks_become_supabase_calls() {
        let (id, path, token, body) = request("7\tsave\ttok\t3\t4\t11,12\tv2|1|500").unwrap();
        assert_eq!((id, path.as_str(), token.as_str()), (7, "/rest/v1/rpc/put_save", "tok"));
        assert_eq!(body["p_base"], 3);
        assert_eq!(body["p_event_ids"], json!([11, 12]));
        assert_eq!(body["p_text"], "v2|1|500");
        assert!(request("x\tsave").is_none());
        assert!(request("1\tdrop\t\t").is_none());
    }

    #[test]
    fn replies_become_one_line() {
        assert_eq!(normalize(2, "save", 200, br#"{"ok":true,"version":5}"#), "2\tok\t5");
        assert_eq!(normalize(2, "save", 200, br#"{"ok":false,"conflict":true,"version":6,"text":"v2|9"}"#), "2\tconflict\t6\tv2|9");
        assert_eq!(normalize(3, "state", 200, br#"{"save":null,"events":[]}"#), "3\tok\t0\t\t");
        assert_eq!(
            normalize(3, "state", 200, br#"{"save":{"version":2,"text":"v2|1|x","contract":4},"events":[{"id":9,"kind":"review","n":40,"day":20000}]}"#),
            "3\tok\t2\t9:40:20000\tv2|1|x"
        );
        assert_eq!(normalize(4, "signin", 400, br#"{"error":"invalid_grant"}"#), "4\tauth");
        assert_eq!(normalize(4, "state", 401, b""), "4\tauth");
        assert_eq!(normalize(5, "join", 200, br#"{"email":"a@b","password":"p"}"#), "5\tok\ta@b\tp");
        assert_eq!(normalize(9, "bug", 201, b""), "9\tok");
        let (_, path, _, body) = request("9\tbug\ttok\tit froze\tv2|1|x").unwrap();
        assert_eq!((path.as_str(), body["text"].as_str(), body["save"].as_str()), ("/rest/v1/bug_reports", Some("it froze"), Some("v2|1|x")));
        assert_eq!(normalize(6, "mkcode", 200, br#"{"code":"ABCD2345"}"#), "6\tok\tABCD2345");
        assert_eq!(normalize(7, "claim", 404, br#"{"error":"no account has that code"}"#), "7\terror\tno account has that code");
        assert_eq!(normalize(7, "claim", 200, br#"{"email":"a@b","password":"q/+="}"#), "7\tok\ta@b\tq/+=");
        let (_, path, _, body) = request("8\tclaim\t\tABCD2345").unwrap();
        assert_eq!((path.as_str(), body["code"].as_str()), ("/functions/v1/restore", Some("ABCD2345")));
    }
}
