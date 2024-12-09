// #[allow(warnings)]
// mod bindings;
// use serde_json::{json, Map as JsonMap, Value as JsonValue};
//
// use bindings::{
//     exports::supabase::wrappers::routines::Guest,
//     supabase::wrappers::{
//         http, time,
//         types::{Cell, Column, Context, FdwError, FdwResult, OptionsType, Row, TypeOid, Value},
//         utils,
//     },
// };
//
// #[derive(Debug, Default)]
// struct FhirFdw {
//     base_url: String,
//     url: Option<String>,
//     headers: Vec<(String, String)>,
//     object: String,
//     src_rows: Vec<JsonValue>,
//     src_idx: usize,
// }
//
// // pointer for the static FDW instance
// static mut INSTANCE: *mut FhirFdw = std::ptr::null_mut::<FhirFdw>();
//
// impl FhirFdw {
//     // initialise FDW instance
//     fn init_instance() {
//         let instance = Self::default();
//         unsafe {
//             INSTANCE = Box::leak(Box::new(instance));
//         }
//     }
//
//     fn this_mut() -> &'static mut Self {
//         unsafe { &mut (*INSTANCE) }
//     }
//
//     fn can_pushdown_id(&self) -> bool {
//         self.object.starts_with("Observation")
//     }
//
//     fn page_size(&self) -> usize {
//         match self.object.as_str() {
//             "Observation" => 20,
//             _ => 200,
//         }
//     }
//
//     fn src_to_cell(&self, src_row: &JsonValue, tgt_col: &Column) -> Result<Option<Cell>, FdwError> {
//         let tgt_col_name = tgt_col.name();
//
//         if &tgt_col_name == "attrs" {
//             return Ok(Some(Cell::Json(src_row.to_string())));
//         }
//
//         let mut src = &Default::default();
//
//         match tgt_col_name.as_str() {
//             "effectivestart" => {
//                 src = src_row
//                     .as_object()
//                     .and_then(|v| v.get("resource"))
//                     .and_then(|v| {
//                         v.get("effectiveDateTime")
//                             .or_else(|| v.get("effectivePeriod").and_then(|ep| ep.get("start")))
//                     })
//                     .ok_or(format!("source column '{}' not found", tgt_col_name))?;
//             }
//             "effectiveend" => {
//                 src = src_row
//                     .as_object()
//                     .and_then(|v| v.get("resource"))
//                     .and_then(|v| v.get("effectivePeriod"))
//                     .and_then(|v| v.get("end"))
//                     .ok_or(format!("source column '{}' not found", tgt_col_name))?;
//             }
//             "subject" => {
//                 src = src_row
//                     .as_object()
//                     .and_then(|v| v.get("resource"))
//                     .and_then(|v| v.get("subject"))
//                     .and_then(|v| v.get("reference"))
//                     .ok_or(format!("source column '{}' not found", tgt_col_name))?;
//             }
//             "loinccode" => {
//                 if let Some(coding_array) = src_row
//                     .as_object()
//                     .and_then(|v| v.get("resource"))
//                     .and_then(|v| v.get("code"))
//                     .and_then(|v| v.get("coding"))
//                     .and_then(|v| v.as_array())
//                 {
//                     for coding in coding_array {
//                         if let Some(system) = coding.get("system") {
//                             if system == "http://loinc.org" {
//                                 src = coding.get("code").ok_or(format!(
//                                     "Cannot extract 'code' when 'system' is 'http://loinc.org'"
//                                 ))?;
//                                 break;
//                             }
//                         }
//                     }
//                 }
//             }
//             "value" => {
//                 if let Some(quantity) = src_row
//                     .as_object()
//                     .and_then(|v| v.get("resource"))
//                     .and_then(|v| v.get("valueQuantity"))
//                 {
//                     src = quantity.get("value").ok_or(format!(
//                         "Cannot extract 'value' from 'valueQuantity' for column '{}'",
//                         tgt_col_name
//                     ))?;
//                 }
//             }
//             "unit" => {
//                 if let Some(quantity) = src_row
//                     .as_object()
//                     .and_then(|v| v.get("resource"))
//                     .and_then(|v| v.get("valueQuantity"))
//                 {
//                     src = quantity.get("unit").ok_or(format!(
//                         "Cannot extract 'unit' from 'valueQuantity' for column '{}'",
//                         tgt_col_name
//                     ))?;
//                 }
//             }
//             _ => {
//                 src = src_row
//                     .as_object()
//                     .and_then(|v| v.get("resource"))
//                     .and_then(|v| v.get(&tgt_col_name))
//                     .ok_or(format!("source column '{}' not found", tgt_col_name))?;
//             }
//         }
//
//         let cell = match tgt_col.type_oid() {
//             TypeOid::Bool => src.as_bool().map(Cell::Bool),
//             TypeOid::I8 => src.as_i64().map(|v| Cell::I8(v as i8)),
//             TypeOid::I16 => src.as_i64().map(|v| Cell::I16(v as i16)),
//             TypeOid::F32 => src.as_f64().map(|v| Cell::F32(v as f32)),
//             TypeOid::I32 => src.as_i64().map(|v| Cell::I32(v as i32)),
//             TypeOid::F64 => src.as_f64().map(Cell::F64),
//             TypeOid::I64 => src.as_i64().map(Cell::I64),
//             TypeOid::Numeric => src.as_f64().map(Cell::Numeric),
//             TypeOid::String => src.as_str().map(|v| Cell::String(v.to_owned())),
//             TypeOid::Date => {
//                 if let Some(s) = src.as_str() {
//                     let ts = time::parse_from_rfc3339(s)?;
//                     Some(Cell::Date(ts / 1_000_000))
//                 } else {
//                     None
//                 }
//             }
//             TypeOid::Timestamp => {
//                 if let Some(s) = src.as_str() {
//                     let ts = time::parse_from_rfc3339(s)?;
//                     Some(Cell::Timestamp(ts))
//                 } else {
//                     None
//                 }
//             }
//             TypeOid::Timestamptz => {
//                 if let Some(s) = src.as_str() {
//                     let ts = time::parse_from_rfc3339(s)?;
//                     Some(Cell::Timestamptz(ts))
//                 } else {
//                     None
//                 }
//             }
//             TypeOid::Json => src.as_object().map(|_| Cell::Json(src.to_string())),
//         };
//
//         Ok(cell)
//     }
//
//     fn make_request(&mut self, ctx: &Context) -> FdwResult {
//         let quals = ctx.get_quals();
//
//         let url = if let Some(ref url) = self.url {
//             url.clone()
//         } else {
//             let object = quals
//                 .iter()
//                 .find(|q| q.field() == "id")
//                 .and_then(|id| {
//                     if !self.can_pushdown_id() {
//                         return None;
//                     }
//
//                     // push down id filter
//                     match id.value() {
//                         Value::Cell(Cell::String(s)) => Some(format!("{}/{}", self.object, s)),
//                         _ => None,
//                     }
//                 })
//                 .unwrap_or_else(|| self.object.clone());
//             format!("{}/{}?_count={}", self.base_url, object, self.page_size())
//         };
//         let req = http::Request {
//             method: http::Method::Get,
//             url,
//             headers: self.headers.clone(),
//             body: String::default(),
//         };
//         let resp = http::get(&req)?;
//         let resp_json: JsonValue = serde_json::from_str(&resp.body).map_err(|e| e.to_string())?;
//
//         // if the 404 is caused by no object found, we shouldn't take it as an error
//         // if resp.status_code == 404 && resp_json.pointer("/error/code") == Some(&json!("not_found"))
//         // {
//         //     self.src_rows = Vec::default();
//         //     self.src_idx = 0;
//         //     self.url = None;
//         //     return Ok(());
//         // }
//
//         http::error_for_status(&resp).map_err(|err| format!("{}: {}", err, resp.body))?;
//
//         // save source rows
//         self.src_rows = resp_json
//             .as_object()
//             .and_then(|v| v.get("entry"))
//             .and_then(|v| {
//                 // convert a single object response to an array
//                 if v.is_object() {
//                     Some(vec![v.to_owned()])
//                 } else {
//                     v.as_array().cloned()
//                 }
//             })
//             .ok_or("cannot get query result data")?;
//
//         self.src_idx = 0;
//
//         // let pagination = resp_json.pointer("/link").and_then(|v| v.as_array());
//         //
//         // if let Some(next_link) = pagination.and_then(|array| {
//         //     array.iter().find_map(|item| {
//         //         if item.get("relation").and_then(|r| r.as_str()) == Some("next") {
//         //             item.get("url")
//         //                 .and_then(|href| href.as_str())
//         //                 .map(|href| href.to_owned())
//         //         } else {
//         //             None
//         //         }
//         //     })
//         // }) {
//         //     self.url = next_link
//         // }
//
//         self.url = None;
//         Ok(())
//     }
// }
//
// impl Guest for FhirFdw {
//     fn host_version_requirement() -> String {
//         // semver expression for Wasm FDW host version requirement
//         // ref: https://docs.rs/semver/latest/semver/enum.Op.html
//         "^0.1.0".to_string()
//     }
//
//     fn init(ctx: &Context) -> FdwResult {
//         Self::init_instance();
//         let this = Self::this_mut();
//
//         let opts = ctx.get_options(OptionsType::Server);
//         this.base_url = opts.require_or("fhir_url", "https://hapi.fhir.org/baseR4");
//         // let api_key = match opts.get("api_key") {
//         //     Some(key) => key,
//         //     None => {
//         //         let key_id = opts.require("api_key_id")?;
//         //         utils::get_vault_secret(&key_id).unwrap_or_default()
//         //     }
//         // };
//
//         this.headers
//             .push(("content-type".to_owned(), "application/json".to_string()));
//         // this.headers
//         //     .push(("authorization".to_owned(), format!("Bearer {}", api_key)));
//
//         Ok(())
//     }
//
//     fn begin_scan(ctx: &Context) -> FdwResult {
//         let this = Self::this_mut();
//         let opts = ctx.get_options(OptionsType::Table);
//         this.object = opts.require("object")?;
//
//         this.url = None;
//         this.make_request(ctx)?;
//
//         Ok(())
//     }
//
//     fn iter_scan(ctx: &Context, row: &Row) -> Result<Option<u32>, FdwError> {
//         let this = Self::this_mut();
//
//         // if this.src_idx >= this.src_rows.len() {
//         //     if this.url.is_none() {
//         //         return Ok(None);
//         //     }
//         //
//         //     this.make_request(ctx)?;
//         // }
//         //
//         let src_row = &this.src_rows[this.src_idx];
//         for tgt_col in ctx.get_columns() {
//             let cell = this.src_to_cell(src_row, &tgt_col)?;
//             row.push(cell.as_ref());
//         }
//
//         this.src_idx += 1;
//
//         Ok(Some(0))
//     }
//
//     fn re_scan(ctx: &Context) -> FdwResult {
//         let this = Self::this_mut();
//         this.url = None;
//         this.make_request(ctx)
//     }
//
//     fn end_scan(_ctx: &Context) -> FdwResult {
//         let this = Self::this_mut();
//         this.src_rows.clear();
//         Ok(())
//     }
//
//     fn begin_modify(_ctx: &Context) -> FdwResult {
//         Err("modify on foreign table is not supported".to_owned())
//     }
//
//     fn insert(_ctx: &Context, _row: &Row) -> FdwResult {
//         Ok(())
//     }
//
//     fn update(_ctx: &Context, _rowid: Cell, _row: &Row) -> FdwResult {
//         Ok(())
//     }
//
//     fn delete(_ctx: &Context, _rowid: Cell) -> FdwResult {
//         Ok(())
//     }
//
//     fn end_modify(_ctx: &Context) -> FdwResult {
//         Ok(())
//     }
// }
//
// bindings::export!(FhirFdw with_types_in bindings);
#[allow(warnings)]
mod bindings;
use serde_json::{json, Map as JsonMap, Value as JsonValue};

use bindings::{
    exports::supabase::wrappers::routines::Guest,
    supabase::wrappers::{
        http, time,
        types::{Cell, Column, Context, FdwError, FdwResult, OptionsType, Row, TypeOid, Value},
        utils,
    },
};
#[derive(Debug, Default)]
struct FhirFdw {
    base_url: String,
    url: Option<String>,
    headers: Vec<(String, String)>,
    object: String,
    src_rows: Vec<JsonValue>,
    row_cnt: usize
}

// #[derive(Debug, Default)]
// struct FhirFdw {
//     base_url: String,
//     url: Option<String>,
//     headers: Vec<(String, String)>,
//     object: String,

//     src_idx: usize,
// }

static mut INSTANCE: *mut FhirFdw = std::ptr::null_mut::<FhirFdw>();

impl FhirFdw {
    fn init() {
        let instance = Self::default();
        unsafe {
            INSTANCE = Box::leak(Box::new(instance));
        }
    }

    fn this_mut() -> &'static mut Self {
        unsafe { &mut (*INSTANCE) }
    }

    fn src_to_cell(&self, src_row: &JsonValue, tgt_col: &Column) -> Result<Option<Cell>, FdwError> {
        let tgt_col_name = tgt_col.name();

        if &tgt_col_name == "attrs" {
            return Ok(Some(Cell::Json(src_row.to_string())));
        }

        let mut src = &Default::default();

        match tgt_col_name.as_str() {
            "effectivestart" => {
                src = src_row
                    .as_object()
                    .and_then(|v| v.get("resource"))
                    .and_then(|v| {
                        v.get("effectiveDateTime")
                            .or_else(|| v.get("effectivePeriod").and_then(|ep| ep.get("start")))
                    })
                    .ok_or(format!("source column '{}' not found", tgt_col_name))?;
            }
            "effectiveend" => {
                src = src_row
                    .as_object()
                    .and_then(|v| v.get("resource"))
                    .and_then(|v| v.get("effectivePeriod"))
                    .and_then(|v| v.get("end"))
                    .ok_or(format!("source column '{}' not found", tgt_col_name))?;
            }
            "subject" => {
                src = src_row
                    .as_object()
                    .and_then(|v| v.get("resource"))
                    .and_then(|v| v.get("subject"))
                    .and_then(|v| v.get("reference"))
                    .ok_or(format!("source column '{}' not found", tgt_col_name))?;
            }
            "loinccode" => {
                if let Some(coding_array) = src_row
                    .as_object()
                    .and_then(|v| v.get("resource"))
                    .and_then(|v| v.get("code"))
                    .and_then(|v| v.get("coding"))
                    .and_then(|v| v.as_array())
                {
                    for coding in coding_array {
                        if let Some(system) = coding.get("system") {
                            if system == "http://loinc.org" {
                                src = coding.get("code").ok_or(format!(
                                    "Cannot extract 'code' when 'system' is 'http://loinc.org'"
                                ))?;
                                break;
                            }
                        }
                    }
                }
            }
            "value" => {
                if let Some(quantity) = src_row
                    .as_object()
                    .and_then(|v| v.get("resource"))
                    .and_then(|v| v.get("valueQuantity"))
                {
                    src = quantity.get("value").ok_or(format!(
                        "Cannot extract 'value' from 'valueQuantity' for column '{}'",
                        tgt_col_name
                    ))?;
                }
            }
            "unit" => {
                if let Some(quantity) = src_row
                    .as_object()
                    .and_then(|v| v.get("resource"))
                    .and_then(|v| v.get("valueQuantity"))
                {
                    src = quantity.get("unit").ok_or(format!(
                        "Cannot extract 'unit' from 'valueQuantity' for column '{}'",
                        tgt_col_name
                    ))?;
                }
            }
            _ => {
                src = src_row
                    .as_object()
                    .and_then(|v| v.get("resource"))
                    .and_then(|v| v.get(&tgt_col_name))
                    .ok_or(format!("source column '{}' not found", tgt_col_name))?;
            }
        }

        let cell = match tgt_col.type_oid() {
            TypeOid::Bool => src.as_bool().map(Cell::Bool),
            TypeOid::I8 => src.as_i64().map(|v| Cell::I8(v as i8)),
            TypeOid::I16 => src.as_i64().map(|v| Cell::I16(v as i16)),
            TypeOid::F32 => src.as_f64().map(|v| Cell::F32(v as f32)),
            TypeOid::I32 => src.as_i64().map(|v| Cell::I32(v as i32)),
            TypeOid::F64 => src.as_f64().map(Cell::F64),
            TypeOid::I64 => src.as_i64().map(Cell::I64),
            TypeOid::Numeric => src.as_f64().map(Cell::Numeric),
            TypeOid::String => src.as_str().map(|v| Cell::String(v.to_owned())),
            TypeOid::Date => {
                if let Some(s) = src.as_str() {
                    let ts = time::parse_from_rfc3339(s)?;
                    Some(Cell::Date(ts / 1_000_000))
                } else {
                    None
                }
            }
            TypeOid::Timestamp => {
                if let Some(s) = src.as_str() {
                    let ts = time::parse_from_rfc3339(s)?;
                    Some(Cell::Timestamp(ts))
                } else {
                    None
                }
            }
            TypeOid::Timestamptz => {
                if let Some(s) = src.as_str() {
                    let ts = time::parse_from_rfc3339(s)?;
                    Some(Cell::Timestamptz(ts))
                } else {
                    None
                }
            }
            TypeOid::Json => src.as_object().map(|_| Cell::Json(src.to_string())),
        };

        Ok(cell)
    }

    fn make_request(&mut self, ctx: &Context) -> FdwResult {
        let quals = ctx.get_quals();
        let url = format!("{}/{}?_count={}", self.base_url, "Observation", 20);

        // let url = if let Some(ref url) = self.url {
        //     url.clone()
        // } else {
        //     let object = quals
        //         .iter()
        //         .find(|q| q.field() == "id")
        //         .and_then(|id| {
        //             if !self.can_pushdown_id() {
        //                 return None;
        //             }
        //
        //             // push down id filter
        //             match id.value() {
        //                 Value::Cell(Cell::String(s)) => Some(format!("{}/{}", self.object, s)),
        //                 _ => None,
        //             }
        //         })
        //         .unwrap_or_else(|| self.object.clone());
        //     format!("{}/{}?_count={}", self.base_url, object, 20)
        // };
        let req = http::Request {
            method: http::Method::Get,
            url,
            headers: self.headers.clone(),
            body: String::default(),
        };
        let resp = http::get(&req)?;
        let resp_json: JsonValue = serde_json::from_str(&resp.body).map_err(|e| e.to_string())?;

        // save source rows
        self.src_rows = resp_json
            .as_object()
            .and_then(|v| v.get("entry"))
            .and_then(|v| {
                if v.is_object() {
                    Some(vec![v.to_owned()])
                } else {
                    v.as_array().cloned()
                }
            })
            .ok_or("cannot get query result data")?;

        Ok(())
    }
}

impl Guest for FhirFdw {
    fn host_version_requirement() -> String {
        // semver ref: https://docs.rs/semver/latest/semver/enum.Op.html
        "^0.1.0".to_string()
    }

    fn init(_ctx: &Context) -> FdwResult {
        Self::init();
        let this = Self::this_mut();
        let opts = _ctx.get_options(OptionsType::Server);
        this.base_url = opts.require_or("fhir_url", "https://hapi.fhir.org/baseR4");
        // let api_key = match opts.get("api_key") {
        //     Some(key) => key,
        //     None => {
        //         let key_id = opts.require("api_key_id")?;
        //         utils::get_vault_secret(&key_id).unwrap_or_default()
        //     }
        // };

        this.headers
            .push(("content-type".to_owned(), "application/json".to_string()));
        Ok(())
    }

    fn begin_scan(_ctx: &Context) -> FdwResult {
        let this = Self::this_mut();

        this.make_request(_ctx)?;

        this.row_cnt = 0;

        Ok(())
    }

    fn iter_scan(ctx: &Context, row: &Row) -> Result<Option<u32>, FdwError> {
        let this = Self::this_mut();

        if this.row_cnt >= this.src_rows.len()  {
            // return 'None' to stop data scans
            return Ok(None);
        }

        let src_row = &this.src_rows[this.row_cnt];
        for tgt_col in ctx.get_columns() {
            let cell = this.src_to_cell(src_row, &tgt_col)?;
            row.push(cell.as_ref());
        }

        // for tgt_col in &ctx.get_columns() {
        //     match tgt_col.name().as_str() {
        //         "id" => {
        //             row.push(Some(&Cell::I64(42)));
        //         }
        //         "col" => {
        //             row.push(Some(&Cell::String("Hello world".to_string())));
        //         }
        //         _ => unreachable!(),
        //     }
        // }

        this.row_cnt += 1;

        // return Some(_) to Postgres and continue data scan
        Ok(Some(0))
    }

    fn re_scan(_ctx: &Context) -> FdwResult {
        // reset row counter
        let this = Self::this_mut();
        this.row_cnt = 0;
        Ok(())
    }

    fn end_scan(_ctx: &Context) -> FdwResult {
        Ok(())
    }

    fn begin_modify(_ctx: &Context) -> FdwResult {
        unimplemented!("update on foreign table is not supported");
    }

    fn insert(_ctx: &Context, _row: &Row) -> FdwResult {
        unimplemented!("update on foreign table is not supported");
    }

    fn update(_ctx: &Context, _rowid: Cell, _row: &Row) -> FdwResult {
        unimplemented!("update on foreign table is not supported");
    }

    fn delete(_ctx: &Context, _rowid: Cell) -> FdwResult {
        unimplemented!("update on foreign table is not supported");
    }

    fn end_modify(_ctx: &Context) -> FdwResult {
        unimplemented!("update on foreign table is not supported");
    }
}

bindings::export!(FhirFdw with_types_in bindings);
