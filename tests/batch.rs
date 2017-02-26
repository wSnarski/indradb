#[macro_use]
extern crate maplit;
#[macro_use]
extern crate braid;
#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate serde_json;
extern crate chrono;
extern crate rand;
extern crate regex;
extern crate hyper;
extern crate uuid;

mod common;

use std::collections::BTreeMap;

use serde::Deserialize;
use hyper::client::Client;
use hyper::status::StatusCode;
use serde_json::value::Value as JsonValue;
use serde_json::Number as JsonNumber;
pub use regex::Regex;
use uuid::Uuid;
pub use braid::*;
pub use common::{HttpDatastore, HttpTransaction, request, response_to_error_message};
use std::io::Read;
use chrono::{DateTime, UTC};

lazy_static! {
	static ref ITEM_ERROR_MESSAGE_PATTERN: Regex = Regex::new(r"Item #0: (.+)").unwrap();
}

fn format_datetime(datetime: Option<DateTime<UTC>>) -> JsonValue {
    match datetime {
        Some(val) => JsonValue::String(val.to_rfc3339()),
        None => JsonValue::Null
    }
}

fn serialize_type(t: Option<Type>) -> JsonValue {
    match t {
        Some(t) => JsonValue::String(t.0),
        None => JsonValue::Null
    }
}

pub struct BatchTransaction {
    port: i32,
    account_id: Uuid,
    secret: String,
}

impl HttpTransaction<BatchTransaction> for BatchTransaction {
    fn new(port: i32, account_id: Uuid, secret: String) -> Self {
        BatchTransaction {
            port: port,
            account_id: account_id,
            secret: secret,
        }
    }
}

impl BatchTransaction {
    fn request<T: Deserialize>(&self, d: BTreeMap<String, JsonValue>) -> Result<T, Error> {
        let body = serde_json::to_string(&vec![d]).unwrap();
        let client = Client::new();
        let req = request(
            &client,
            self.port,
            self.account_id,
            self.secret.clone(),
            "POST",
            "/transaction".to_string(),
            vec![]
        ).body(&body[..]);
        let mut res = req.send().unwrap();

        let mut payload = String::new();
        res.read_to_string(&mut payload).unwrap();

        match res.status {
            StatusCode::Ok => {
                let mut v: Vec<T> = serde_json::from_str(&payload[..]).unwrap();
                let o = v.pop().unwrap();
                Ok(o)
            }
            _ => {
                let o: BTreeMap<String, JsonValue> = serde_json::from_str(&payload[..]).unwrap();

                match o.get("error") {
                    Some(&JsonValue::String(ref error)) => {
                        let cap = ITEM_ERROR_MESSAGE_PATTERN.captures(error).unwrap();
                        let message = cap.get(1).unwrap().as_str();
                        Err(Error::description_to_error(message))
                    }
                    _ => panic!("Could not unpack error message"),
                }
            }
        }
    }
}

impl Transaction<Uuid> for BatchTransaction {
    fn get_vertex_range(&self, start_id: Uuid, limit: u16) -> Result<Vec<Vertex<Uuid>>, Error> {
        self.request(btreemap!{
			"action".to_string() => JsonValue::String("get_vertex_range".to_string()),
			"start_id".to_string() => JsonValue::String(start_id.hyphenated().to_string()),
			"limit".to_string() => JsonValue::Number(JsonNumber::from(limit))
		})
    }

    fn get_vertex(&self, id: Uuid) -> Result<Vertex<Uuid>, Error> {
        self.request(btreemap!{
			"action".to_string() => JsonValue::String("get_vertex".to_string()),
			"id".to_string() => JsonValue::String(id.hyphenated().to_string())
		})
    }

    fn create_vertex(&self, t: Type) -> Result<Uuid, Error> {
        self.request(btreemap!{
			"action".to_string() => JsonValue::String("create_vertex".to_string()),
			"type".to_string() => JsonValue::String(t.0)
		})
    }

    fn set_vertex(&self, v: Vertex<Uuid>) -> Result<(), Error> {
        self.request(btreemap!{
			"action".to_string() => JsonValue::String("set_vertex".to_string()),
			"id".to_string() => JsonValue::String(v.id.hyphenated().to_string()),
			"type".to_string() => JsonValue::String(v.t.0)
		})
    }

    fn delete_vertex(&self, id: Uuid) -> Result<(), Error> {
        self.request(btreemap!{
			"action".to_string() => JsonValue::String("delete_vertex".to_string()),
			"id".to_string() => JsonValue::String(id.hyphenated().to_string())
		})
    }
    
    fn get_edge(&self, outbound_id: Uuid, t: Type, inbound_id: Uuid) -> Result<Edge, Error> {
        self.request(btreemap!{
			"action".to_string() => JsonValue::String("get_edge".to_string()),
			"outbound_id".to_string() => JsonValue::String(outbound_id.hyphenated().to_string()),
			"type".to_string() => JsonValue::String(t.0),
			"inbound_id".to_string() => JsonValue::String(inbound_id.hyphenated().to_string())
		})
    }

    fn set_edge(&self, e: Edge) -> Result<(), Error> {
        self.request(btreemap!{
			"action".to_string() => JsonValue::String("set_edge".to_string()),
			"outbound_id".to_string() => JsonValue::String(e.outbound_id.hyphenated().to_string()),
			"type".to_string() => JsonValue::String(e.t.0),
			"inbound_id".to_string() => JsonValue::String(e.inbound_id.hyphenated().to_string()),
			"weight".to_string() => JsonValue::Number(JsonNumber::from_f64(e.weight.0 as f64).unwrap())
		})
    }

    fn delete_edge(&self, outbound_id: Uuid, t: Type, inbound_id: Uuid) -> Result<(), Error> {
        self.request(btreemap!{
			"action".to_string() => JsonValue::String("delete_edge".to_string()),
			"outbound_id".to_string() => JsonValue::String(outbound_id.hyphenated().to_string()),
			"type".to_string() => JsonValue::String(t.0),
			"inbound_id".to_string() => JsonValue::String(inbound_id.hyphenated().to_string())
		})
    }

    fn get_edge_count(&self, outbound_id: Uuid, t: Option<Type>) -> Result<u64, Error> {
        self.request(btreemap!{
			"action".to_string() => JsonValue::String("get_edge_count".to_string()),
			"outbound_id".to_string() => JsonValue::String(outbound_id.hyphenated().to_string()),
			"type".to_string() => serialize_type(t)
		})
    }

    fn get_edge_range(&self, outbound_id: Uuid, t: Option<Type>, offset: u64, limit: u16) -> Result<Vec<Edge>, Error> {
        self.request(btreemap!{
			"action".to_string() => JsonValue::String("get_edge_range".to_string()),
			"outbound_id".to_string() => JsonValue::String(outbound_id.hyphenated().to_string()),
			"type".to_string() => serialize_type(t),
			"offset".to_string() => JsonValue::Number(JsonNumber::from(offset)),
			"limit".to_string() => JsonValue::Number(JsonNumber::from(limit))
		})
    }

    fn get_edge_time_range(&self, outbound_id: Uuid, t: Option<Type>, high: Option<DateTime<UTC>>, low: Option<DateTime<UTC>>, limit: u16) -> Result<Vec<Edge>, Error> {
        self.request(btreemap!{
			"action".to_string() => JsonValue::String("get_edge_time_range".to_string()),
			"outbound_id".to_string() => JsonValue::String(outbound_id.hyphenated().to_string()),
			"type".to_string() => serialize_type(t),
            "high".to_string() => format_datetime(high),
            "low".to_string() => format_datetime(low),
			"limit".to_string() => JsonValue::Number(JsonNumber::from(limit))
		})
    }

    fn get_reversed_edge_count(&self, inbound_id: Uuid, t: Option<Type>) -> Result<u64, Error> {
        self.request(btreemap!{
			"action".to_string() => JsonValue::String("get_reversed_edge_count".to_string()),
			"inbound_id".to_string() => JsonValue::String(inbound_id.hyphenated().to_string()),
			"type".to_string() => serialize_type(t)
		})
    }

    fn get_reversed_edge_range(&self, inbound_id: Uuid, t: Option<Type>, offset: u64, limit: u16) -> Result<Vec<Edge>, Error> {
        self.request(btreemap!{
			"action".to_string() => JsonValue::String("get_reversed_edge_range".to_string()),
			"inbound_id".to_string() => JsonValue::String(inbound_id.hyphenated().to_string()),
			"type".to_string() => serialize_type(t),
			"offset".to_string() => JsonValue::Number(JsonNumber::from(offset)),
			"limit".to_string() => JsonValue::Number(JsonNumber::from(limit))
		})
    }

    fn get_reversed_edge_time_range(&self, inbound_id: Uuid, t: Option<Type>, high: Option<DateTime<UTC>>, low: Option<DateTime<UTC>>, limit: u16) -> Result<Vec<Edge>, Error> {
        self.request(btreemap!{
			"action".to_string() => JsonValue::String("get_reversed_edge_time_range".to_string()),
			"inbound_id".to_string() => JsonValue::String(inbound_id.hyphenated().to_string()),
			"type".to_string() => serialize_type(t),
            "high".to_string() => format_datetime(high),
            "low".to_string() => format_datetime(low),
			"limit".to_string() => JsonValue::Number(JsonNumber::from(limit))
		})
    }

    fn get_global_metadata(&self, _: String) -> Result<JsonValue, Error> {
        panic!("Unimplemented")
    }

    fn set_global_metadata(&self, _: String, _: JsonValue) -> Result<(), Error> {
        panic!("Unimplemented")
    }

    fn delete_global_metadata(&self, _: String) -> Result<(), Error> {
        panic!("Unimplemented")
    }

    fn get_account_metadata(&self, _: Uuid, _: String) -> Result<JsonValue, Error> {
        panic!("Unimplemented")
    }

    fn set_account_metadata(&self, _: Uuid, _: String, _: JsonValue) -> Result<(), Error> {
        panic!("Unimplemented")
    }

    fn delete_account_metadata(&self, _: Uuid, _: String) -> Result<(), Error> {
        panic!("Unimplemented")
    }

    fn get_vertex_metadata(&self, _: Uuid, _: String) -> Result<JsonValue, Error> {
        panic!("Unimplemented")
    }

    fn set_vertex_metadata(&self, _: Uuid, _: String, _: JsonValue) -> Result<(), Error> {
        panic!("Unimplemented")
    }

    fn delete_vertex_metadata(&self, _: Uuid, _: String) -> Result<(), Error> {
        panic!("Unimplemented")
    }

    fn get_edge_metadata(&self, _: Uuid, _: Type, _: Uuid, _: String) -> Result<JsonValue, Error> {
        panic!("Unimplemented")
    }

    fn set_edge_metadata(&self, _: Uuid, _: Type, _: Uuid, _: String, _: JsonValue) -> Result<(), Error> {
        panic!("Unimplemented")
    }

    fn delete_edge_metadata(&self, _: Uuid, _: Type, _: Uuid, _: String) -> Result<(), Error> {
        panic!("Unimplemented")
    }

    fn commit(self) -> Result<(), Error> {
        Ok(())
    }

    fn rollback(self) -> Result<(), Error> {
        Err(Error::Unexpected("Cannot rollback an HTTP-based transaction".to_string()))
    }
}

test_transaction_impl! {
	test_batch_transaction {
	    HttpDatastore::<BatchTransaction, BatchTransaction>::new(8000)
	}
}
