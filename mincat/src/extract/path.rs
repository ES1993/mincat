use std::{any::type_name, slice::Iter};

use mincat_core::request::{FromRequest, Request};
use serde::{
    de::{DeserializeOwned, DeserializeSeed, MapAccess, SeqAccess, Visitor},
    forward_to_deserialize_any, Deserializer,
};

use crate::app::MincatRoutePath;

use super::ExtractError;

fn get_path_args(path: &str) -> Vec<String> {
    let mut res = vec![];
    path.split('/').for_each(|item| {
        if let Some(key) = item.strip_prefix(':') {
            res.push(key.into());
        }
    });

    res
}

fn get_path_args_tuple(
    define_path: &str,
    real_path: &str,
) -> Result<Vec<(String, String)>, ExtractError> {
    let mut res = vec![];
    let mut router = matchit::Router::new();
    router
        .insert(define_path, true)
        .map_err(ExtractError::from)?;
    let matched = router.at(real_path).map_err(ExtractError::from)?;
    let path_args = get_path_args(define_path);

    for arg in path_args {
        let value = matched.params.get(&arg).unwrap_or("");
        res.push((arg, value.to_string()));
    }

    Ok(res)
}

#[derive(Clone, Debug)]
pub struct Path<T>(pub T);

#[async_trait::async_trait]
impl<T> FromRequest for Path<T>
where
    T: Default + DeserializeOwned + Clone + Send + 'static,
{
    type Error = ExtractError;

    async fn from_request(request: &mut Request) -> Result<Self, Self::Error> {
        let mut res = T::default();

        if let Some(MincatRoutePath(path)) = request.extensions().get::<MincatRoutePath>() {
            let path_args = get_path_args_tuple(path, request.uri().path())?;
            res = T::deserialize(PathDeserializer(path_args.iter()))?;
        }

        Ok(Path(res))
    }
}

macro_rules! unsupport_type {
    ($trait_fn:ident) => {
        fn $trait_fn<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(ExtractError(format!(
                "unsupport_type: {}",
                type_name::<V::Value>()
            )))
        }
    };

    ($trait_fn:ident, name) => {
        fn $trait_fn<V>(self, _: &'static str, _: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(ExtractError(format!(
                "unsupport_type: {}",
                type_name::<V::Value>()
            )))
        }
    };

    ($trait_fn:ident, len) => {
        fn $trait_fn<V>(self, _: usize, _: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(ExtractError(format!(
                "unsupport_type: {}",
                type_name::<V::Value>()
            )))
        }
    };

    ($trait_fn:ident, name_len) => {
        fn $trait_fn<V>(self, _: &'static str, _: usize, _: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(ExtractError(format!(
                "unsupport_type: {}",
                type_name::<V::Value>()
            )))
        }
    };

    ($trait_fn:ident, name_arr) => {
        fn $trait_fn<V>(
            self,
            _: &'static str,
            _: &'static [&'static str],
            _: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(ExtractError(format!(
                "unsupport_type: {}",
                type_name::<V::Value>()
            )))
        }
    };
}

#[derive(Debug)]
struct PathDeserializer<'de>(Iter<'de, (String, String)>);

impl<'de> Deserializer<'de> for PathDeserializer<'de> {
    type Error = ExtractError;

    unsupport_type!(deserialize_bytes);
    unsupport_type!(deserialize_option);
    unsupport_type!(deserialize_identifier);
    unsupport_type!(deserialize_ignored_any);
    unsupport_type!(deserialize_any);
    unsupport_type!(deserialize_bool);
    unsupport_type!(deserialize_i8);
    unsupport_type!(deserialize_i16);
    unsupport_type!(deserialize_i32);
    unsupport_type!(deserialize_i64);
    unsupport_type!(deserialize_u8);
    unsupport_type!(deserialize_u16);
    unsupport_type!(deserialize_u32);
    unsupport_type!(deserialize_u64);
    unsupport_type!(deserialize_f32);
    unsupport_type!(deserialize_f64);
    unsupport_type!(deserialize_char);
    unsupport_type!(deserialize_str);
    unsupport_type!(deserialize_string);
    unsupport_type!(deserialize_byte_buf);
    unsupport_type!(deserialize_unit);
    unsupport_type!(deserialize_unit_struct, name);
    unsupport_type!(deserialize_newtype_struct, name);
    unsupport_type!(deserialize_seq);
    unsupport_type!(deserialize_tuple_struct, name_len);
    unsupport_type!(deserialize_enum, name_arr);

    fn deserialize_tuple<V>(self, _: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(SeqDeserializer(self.0))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(MapDeserializer(self.0, None))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }
}

struct SeqDeserializer<'de>(Iter<'de, (String, String)>);

impl<'de> SeqAccess<'de> for SeqDeserializer<'de> {
    type Error = ExtractError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.0.next() {
            Some(param) => Ok(Some(
                seed.deserialize(ValueDeserializer(&param.1))
                    .map_err(ExtractError::from)?,
            )),
            None => Ok(None),
        }
    }
}

macro_rules! parse_value {
    ($trait_fn:ident, $visit_fn:ident) => {
        fn $trait_fn<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            let v = self.0.parse().map_err(ExtractError::from)?;
            visitor.$visit_fn(v)
        }
    };
}

struct ValueDeserializer<'de>(&'de str);

impl<'de> Deserializer<'de> for ValueDeserializer<'de> {
    type Error = ExtractError;

    unsupport_type!(deserialize_bytes);
    unsupport_type!(deserialize_option);
    unsupport_type!(deserialize_identifier);
    unsupport_type!(deserialize_ignored_any);
    unsupport_type!(deserialize_any);
    unsupport_type!(deserialize_unit);
    unsupport_type!(deserialize_unit_struct, name);
    unsupport_type!(deserialize_newtype_struct, name);
    unsupport_type!(deserialize_seq);
    unsupport_type!(deserialize_tuple_struct, name_len);
    unsupport_type!(deserialize_enum, name_arr);
    unsupport_type!(deserialize_tuple, len);
    unsupport_type!(deserialize_map);
    unsupport_type!(deserialize_struct, name_arr);
    unsupport_type!(deserialize_str);

    parse_value!(deserialize_bool, visit_bool);
    parse_value!(deserialize_i8, visit_i8);
    parse_value!(deserialize_i16, visit_i16);
    parse_value!(deserialize_i32, visit_i32);
    parse_value!(deserialize_i64, visit_i64);
    parse_value!(deserialize_i128, visit_i128);
    parse_value!(deserialize_u8, visit_u8);
    parse_value!(deserialize_u16, visit_u16);
    parse_value!(deserialize_u32, visit_u32);
    parse_value!(deserialize_u64, visit_u64);
    parse_value!(deserialize_u128, visit_u128);
    parse_value!(deserialize_f32, visit_f32);
    parse_value!(deserialize_f64, visit_f64);
    parse_value!(deserialize_string, visit_string);
    parse_value!(deserialize_byte_buf, visit_string);
    parse_value!(deserialize_char, visit_char);
}

struct MapDeserializer<'de>(Iter<'de, (String, String)>, Option<&'de (String, String)>);

impl<'de> MapAccess<'de> for MapDeserializer<'de> {
    type Error = ExtractError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.0.next() {
            Some(param) => {
                self.1 = Some(param);
                seed.deserialize(KeyDeserializer(&param.0)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.1.take() {
            Some((_, ref value)) => seed.deserialize(ValueDeserializer(value)),
            None => Err(ExtractError("value is missing".to_string())),
        }
    }
}

struct KeyDeserializer<'de>(&'de str);

macro_rules! parse_key {
    ($trait_fn:ident) => {
        fn $trait_fn<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            visitor.visit_str(&self.0)
        }
    };
}

impl<'de> Deserializer<'de> for KeyDeserializer<'de> {
    type Error = ExtractError;

    unsupport_type!(deserialize_any);

    parse_key!(deserialize_identifier);
    parse_key!(deserialize_str);
    parse_key!(deserialize_string);

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char bytes
        byte_buf option unit unit_struct seq tuple
        tuple_struct map newtype_struct struct enum ignored_any
    }
}
