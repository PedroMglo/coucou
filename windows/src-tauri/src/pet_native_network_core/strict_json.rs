//! Bounded by the caller's existing byte/frame and serde recursion limits.
//! Duplicate object keys cannot supply an ambiguous protocol/capability proof.
use serde::{Deserialize,Deserializer,de::{Visitor,MapAccess,SeqAccess,Error}};
use serde_json::{Value,Map,Number};
use std::fmt;
struct Strict(Value);
impl<'de> Deserialize<'de> for Strict {
    fn deserialize<D:Deserializer<'de>>(d:D)->Result<Self,D::Error>{
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value=Strict;
            fn expecting(&self,f:&mut fmt::Formatter)->fmt::Result{f.write_str("unambiguous JSON")}
            fn visit_unit<E:Error>(self)->Result<Strict,E>{Ok(Strict(Value::Null))}
            fn visit_none<E:Error>(self)->Result<Strict,E>{self.visit_unit()}
            fn visit_bool<E:Error>(self,v:bool)->Result<Strict,E>{Ok(Strict(Value::Bool(v)))}
            fn visit_i64<E:Error>(self,v:i64)->Result<Strict,E>{Ok(Strict(Value::Number(v.into())))}
            fn visit_u64<E:Error>(self,v:u64)->Result<Strict,E>{Ok(Strict(Value::Number(v.into())))}
            fn visit_f64<E:Error>(self,v:f64)->Result<Strict,E>{Number::from_f64(v).map(|n|Strict(Value::Number(n))).ok_or_else(||E::custom("invalid number"))}
            fn visit_str<E:Error>(self,v:&str)->Result<Strict,E>{Ok(Strict(Value::String(v.into())))}
            fn visit_string<E:Error>(self,v:String)->Result<Strict,E>{Ok(Strict(Value::String(v)))}
            fn visit_seq<A:SeqAccess<'de>>(self,mut a:A)->Result<Strict,A::Error>{
                let mut rows=Vec::new();while let Some(Strict(v))=a.next_element()?{rows.push(v);}Ok(Strict(Value::Array(rows)))
            }
            fn visit_map<A:MapAccess<'de>>(self,mut a:A)->Result<Strict,A::Error>{
                let mut map=Map::new();while let Some(key)=a.next_key::<String>()? {
                    if map.contains_key(&key){return Err(A::Error::custom("duplicate key"));}
                    let Strict(v)=a.next_value()?;map.insert(key,v);
                }Ok(Strict(Value::Object(map)))
            }
        }
        d.deserialize_any(V)
    }
}
pub(super) fn decode(bytes:&[u8])->Result<Value,serde_json::Error>{serde_json::from_slice::<Strict>(bytes).map(|v|v.0)}
