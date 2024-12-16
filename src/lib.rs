#[allow(warnings)]
mod bindings;
use base64;
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
struct LimeFdw {
    base_url: String,
    username: String,
    password: String,
    url: Option<String>,
    headers: Vec<(String, String)>,
    object: String,
    src_rows: Vec<JsonValue>,
    row_cnt: usize,
    session_key: String,
    survey_ids: Vec<u32>,
    responses: Vec<Response>,
    total_answers_count: usize,
}

#[derive(Debug, Default)]
struct Question {
    question: String,
    code: String,
    sid: u32,
}

#[derive(Debug, Default)]
struct Answer {
    question: Question,
    answer: String,
}

#[derive(Debug, Default)]
struct Response {
    answers: Vec<Answer>,
}

static mut INSTANCE: *mut LimeFdw = std::ptr::null_mut::<LimeFdw>();

impl LimeFdw {
    fn init() {
        let instance = Self::default();
        unsafe {
            INSTANCE = Box::leak(Box::new(instance));
        }
    }

    fn this_mut() -> &'static mut Self {
        unsafe { &mut (*INSTANCE) }
    }

    fn answer_to_cell(&self, answer: &Answer, tgt_col: &Column) -> Result<Option<Cell>, FdwError> {
        let tgt_col_name = tgt_col.name();

        // if &tgt_col_name == "attrs" {
        //     return Ok(Some(Cell::Json(src_row.to_string())));
        // }

        if &tgt_col_name == "question" {
            return Ok(Some(Cell::String(answer.question.question.clone())));
        }

        if &tgt_col_name == "questioncode" {
            return Ok(Some(Cell::String(answer.question.code.clone())));
        }

        if &tgt_col_name == "answer" {
            return Ok(Some(Cell::String(answer.answer.clone())));
        }

        if &tgt_col_name == "surveyid" {
            return Ok(Some(Cell::String(answer.question.sid.to_string())));
        }

        Ok(None)
    }

    fn get_session_key(&mut self, ctx: &Context) -> Result<String, FdwError> {
        let quals = ctx.get_quals();
        let url = format!("{}", self.base_url);

        let body = serde_json::json!({
            "method": "get_session_key",
            "params": ["mmuzny", "sytjix-xubFug-9hidva","Authdb"],
            "id": 1
        });

        let req = http::Request {
            method: http::Method::Post,
            url,
            headers: self.headers.clone(),
            body: body.to_string(),
        };
        let resp = http::get(&req)?;
        let resp_json: JsonValue = serde_json::from_str(&resp.body).map_err(|e| e.to_string())?;

        let result = resp_json
            .as_object()
            .and_then(|v| v.get("result"))
            .ok_or("cannot find result in response")?;

        Ok(result.to_string())
    }

    fn get_surveys(&mut self, ctx: &Context) -> Result<Vec<u32>, FdwError> {
        let quals = ctx.get_quals();
        let url = format!("{}", self.base_url);
        let body_list_surveys = serde_json::json!({
            "method": "list_surveys",
            "params": [
                self.session_key.clone(),
            ],
            "id": 2
        });

        let url = format!("{}", self.base_url);
        let req = http::Request {
            method: http::Method::Post,
            url,
            headers: self.headers.clone(),
            body: body_list_surveys.to_string(),
        };
        let resp = http::get(&req)?;
        let resp_json: JsonValue = serde_json::from_str(&resp.body).map_err(|e| e.to_string())?;
        let mut sids: Vec<u32> = vec![];
        if let Some(results) = resp_json
            .as_object()
            .and_then(|v| v.get("result"))
            .and_then(|v| v.as_array())
        {
            sids = results
                .iter()
                .filter_map(|item| {
                    item.as_object()
                        .and_then(|obj| obj.get("sid"))
                        .and_then(|sid| sid.as_u64().map(|v| v as u32))
                })
                .collect();
        }
        Ok(sids)
    }

    fn get_questions(&mut self, ctx: &Context, sid: u32) -> Result<Vec<Question>, FdwError> {
        let quals = ctx.get_quals();
        let url = format!("{}", self.base_url);
        let body_list_surveys = serde_json::json!({
            "method": "list_questions",
            "params": [
                self.session_key.clone(),
                sid
            ],
            "id": 2
        });

        let url = format!("{}", self.base_url);
        let req = http::Request {
            method: http::Method::Post,
            url,
            headers: self.headers.clone(),
            body: body_list_surveys.to_string(),
        };
        let resp = http::get(&req)?;
        let resp_json: JsonValue = serde_json::from_str(&resp.body).map_err(|e| e.to_string())?;
        let mut questions: Vec<Question> = vec![];
        if let Some(results) = resp_json
            .as_object()
            .and_then(|v| v.get("result"))
            .and_then(|v| v.as_array())
        {
            questions = results
                .iter()
                .filter_map(|item| {
                    let question = item.as_object()?;
                    let sid = question
                        .get("sid")
                        .and_then(|qid| qid.as_u64())
                        .map(|v| v as u32)?;

                    let code = question
                        .get("title")
                        .and_then(|title| title.as_str())
                        .map(String::from)?;

                    let question = question
                        .get("question")
                        .and_then(|qtext| qtext.as_str())
                        .map(String::from)?;

                    Some(Question {
                        code,
                        sid,
                        question,
                    })
                })
                .collect();
        }
        Ok(questions)
    }

    fn get_results(&mut self, ctx: &Context, sid: u32) -> Result<Vec<Response>, FdwError> {
        let questions = self.get_questions(ctx, sid)?;

        let body_export_responses = serde_json::json!({
            "method": "export_responses",
            "params": [
                self.session_key.clone(),
                sid,
                "json",
                "en",
                "all",
                "code"
            ],
            "id": 2
        });

        let url = format!("{}", self.base_url);
        let req = http::Request {
            method: http::Method::Post,
            url,
            headers: self.headers.clone(),
            body: body_export_responses.to_string(),
        };
        let resp = http::get(&req)?;
        let resp_json: JsonValue = serde_json::from_str(&resp.body).map_err(|e| e.to_string())?;

        let result = resp_json
            .as_object()
            .and_then(|v| v.get("result"))
            .ok_or("cannot find result in response")?;

        let result_decoded = if let Some(result_base64) = result.as_str() {
            let decoded_bytes = base64::decode(result_base64).map_err(|e| e.to_string())?;
            String::from_utf8(decoded_bytes).map_err(|e| e.to_string())?
        } else {
            return Err("result is not a string".to_string());
        };

        let result_json: JsonValue =
            serde_json::from_str(&result_decoded).map_err(|e| e.to_string())?;

        let answers: Vec<Response> = result_json
            .as_array()
            .ok_or("result_json is not an array")?
            .iter()
            .filter_map(|item| {
                let answer_object = item.as_object()?;

                let mut response: Response = Response { answers: vec![] };

                for (key, value) in answer_object.iter() {
                    // Skip specified keys
                    if ["id", "submitdate", "lastpage", "startlanguage", "seed"]
                        .contains(&key.as_str())
                    {
                        continue;
                    }

                    let answer_text = answer_object
                        .get(key)
                        .and_then(|a| a.as_str())
                        .unwrap_or("");

                    let matching_question = questions.iter().find(|q| q.code == *key);

                    if let Some(question) = matching_question {
                        response.answers.push(Answer {
                            question: Question {
                                code: question.code.clone(),
                                sid: question.sid,
                                question: question.question.clone(),
                            },
                            answer: answer_text.to_string(),
                        });
                    }
                }

                Some(response)
            })
            .collect();
        Ok(answers)
    }
}

impl Guest for LimeFdw {
    fn host_version_requirement() -> String {
        // semver ref: https://docs.rs/semver/latest/semver/enum.Op.html
        "^0.1.0".to_string()
    }

    fn init(_ctx: &Context) -> FdwResult {
        Self::init();
        let this = Self::this_mut();
        let opts = _ctx.get_options(OptionsType::Server);
        this.base_url = opts.require_or(
            "limesurvey_url",
            "https://warifatest.limesurvey.net/admin/remotecontrol",
        );

        this.username = opts.require_or("username", "mmuzny");

        this.password = opts.require_or("password", "sytjix-xubFug-9hidva");
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

        this.session_key = this.get_session_key(_ctx)?;
        this.survey_ids = this.get_surveys(_ctx)?;
        for sid in this.survey_ids.clone() {
            match this.get_results(_ctx, sid) {
                Ok(results) => this.responses.extend(results),
                Err(e) => return Err(e),
            }
        }

        this.total_answers_count = this
            .responses
            .iter()
            .map(|response| response.answers.len())
            .sum();

        this.row_cnt = 0;

        Ok(())
    }

    fn iter_scan(ctx: &Context, row: &Row) -> Result<Option<u32>, FdwError> {
        let this = Self::this_mut();

        if this.row_cnt >= this.total_answers_count {
            return Ok(None);
        }

        let answer = this.responses.get_mut(0).unwrap().answers.pop().unwrap();

        for tgt_col in ctx.get_columns() {
            let cell = this.answer_to_cell(&answer, &tgt_col).unwrap_or_default();
            row.push(cell.as_ref());
        }

        if (this.responses.get_mut(0).iter().count() == 0) {
            this.responses.remove(0);
        }

        this.row_cnt += 1;

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

bindings::export!(LimeFdw with_types_in bindings);
