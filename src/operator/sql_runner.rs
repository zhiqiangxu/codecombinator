use super::sql::SQL;
use async_std::stream::StreamExt;
use async_std::sync::{Arc, Weak};
use chrono;
use serde_json::{value::RawValue as JsonRawValue, Value as JsonValue};
use sqlx::decode::Decode;
use sqlx::mysql::{MySql, MySqlTypeInfo, MySqlValueRef};
use sqlx::Column;
use sqlx::{Row, TypeInfo};

pub struct SqlRunner<DB> {
    w: Weak<SQL<DB>>,
    config: Config,
}

pub struct Config {
    pub sql: String,
}

impl<DB> super::Operator for SqlRunner<DB> {}

impl SqlRunner<MySql> {
    pub fn new(config: Config) -> Self {
        SqlRunner {
            w: Weak::default(),
            config,
        }
    }

    fn get_as_json_value(vref: MySqlValueRef, type_info: &MySqlTypeInfo) -> JsonValue {
        match type_info.name() {
            "INT UNSIGNED" => {
                JsonValue::from(<u32 as Decode::<'_,MySql>>::decode(vref).unwrap())
            },
            "INT" => {
                JsonValue::from(<i32 as Decode::<'_,MySql>>::decode(vref).unwrap())
            },
            "SMALLINT UNSIGNED" => {
                JsonValue::from(<u16 as Decode::<'_,MySql>>::decode(vref).unwrap())
            },
            "SMALLINT" => {
                JsonValue::from(<i16 as Decode::<'_,MySql>>::decode(vref).unwrap())
            },
            "BIGINT UNSIGNED" => {
                JsonValue::from(<u64 as Decode::<'_,MySql>>::decode(vref).unwrap())
            },
            "BIGINT" => {
                JsonValue::from(<i64 as Decode::<'_,MySql>>::decode(vref).unwrap())
            },
            "FLOAT" => {
                JsonValue::from(<f32 as Decode::<'_,MySql>>::decode(vref).unwrap())
            },
            "DOUBLE" => {
                JsonValue::from(<f64 as Decode::<'_,MySql>>::decode(vref).unwrap())
            },
            "NULL" => {
                JsonValue::Null
            },
            "TINYINT UNSIGNED" => {
                JsonValue::from(<u8 as Decode::<'_,MySql>>::decode(vref).unwrap())
            },
            "TINYINT" => {
                JsonValue::from(<i8 as Decode::<'_,MySql>>::decode(vref).unwrap())
            },
            // "TIMESTAMP"|"DATETIME" => {
            //     let r = <chrono::NaiveDateTime as Decode::<'_,MySql>>::decode(vref).unwrap();

            //     let r = serde_json::to_value(&r);

            //     r.unwrap_or(serde_json::Value::Null)
            // },
            // "DATE" => {
            //     let r = <chrono::NaiveDate as Decode::<'_,MySql>>::decode(vref).unwrap();

            //     let r = serde_json::to_value(&r);

            //     r.unwrap_or(serde_json::Value::Null)
            // },
            // "TIME"  => {
            //     let r = <chrono::NaiveTime as Decode::<'_,MySql>>::decode(vref).unwrap();

            //     let r = serde_json::to_value(&r);

            //     r.unwrap_or(serde_json::Value::Null)
            // },
            "JSON" => {
                <JsonValue as Decode::<'_,MySql>>::decode(vref).unwrap()
            },
            _ => {
                JsonValue::from(<String as Decode::<'_,MySql>>::decode(vref).unwrap())
            }
            // "BOOLEAN" 
            // ColumnType::Int24 if is_unsigned => "MEDIUMINT UNSIGNED",
            // ColumnType::Int24 => "MEDIUMINT",
            // ColumnType::Year => "YEAR",
            // ColumnType::Bit => "BIT",
            // ColumnType::Enum => "ENUM",
            // ColumnType::Set => "SET",
            // ColumnType::Decimal | ColumnType::NewDecimal => "DECIMAL",
            // ColumnType::Geometry => "GEOMETRY",
        }
    }

    pub async fn run_sql(&self) -> Option<JsonValue> {
        match self.w.upgrade() {
            Some(a) => {
                let mut arr = vec![];
                let mut con = a.get_executor().await.unwrap();
                let q = sqlx::query(self.config.sql.as_str());
                let mut cursor = a.call(q, &mut con).await;
                while let Some(row) = cursor.next().await {
                    let row = row.unwrap();
                    let mut m = serde_json::Map::new();
                    for (i, column) in row.columns().iter().enumerate() {
                        let type_info = column.type_info();

                        let raw_value = row.try_get_raw(i).unwrap();

                        m.insert(
                            column.name().to_string(),
                            Self::get_as_json_value(raw_value, type_info),
                        );
                    }
                    arr.push(serde_json::Value::Object(m));
                }
                Some(arr.into())
            }
            None => None,
        }
    }
}

impl<DB> super::Monad<super::sql::SQL<DB>> for SqlRunner<DB> {
    type Result = ();

    fn apply(&mut self, w: Weak<super::sql::SQL<DB>>) -> Self::Result {
        self.w = w
    }
}

#[cfg(test)]
mod tests {
    use super::super::sql;
    use super::*;
    use crate::operator::Monad;

    #[async_std::test]
    async fn execute_sql() {
        let mut e = SqlRunner::<MySql>::new(Config {
            sql: "select * from user limit 10".to_string(),
        });
        let sql = sql::SQL::<MySql>::new(sql::Config {
            dsn: "mysql://openkg:some_pass@172.168.3.46:3307/openkg?readTimeout=3s&charset=utf8mb4"
                .to_string(),
        })
        .await;

        let sql = Arc::new(sql);
        e.apply(Arc::downgrade(&sql));
        let result = e.run_sql().await;
        assert!(result.is_some());
        println!("result is {:?}", result.unwrap().to_string());
    }
}
