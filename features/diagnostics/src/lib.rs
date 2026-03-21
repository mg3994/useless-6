use salvo::prelude::*;
use salvo::oapi::extract::*;
use serde::{Deserialize, Serialize};
use askama::Template;
use sqlx::{Executor, AssertSqlSafe, SqlSafeStr};
use services::email_service::EmailService;
// Infrastructure imports

use services::s3_service::StorageService;
use typedef::{AppResult, AppError};

// --- OAPI SCHEMAS ---

#[derive(Serialize, ToSchema, Debug)]
pub struct LocalUploadResult {
    pub url: String,
    pub key: String,
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct QueryReq { pub sql: String }

#[derive(Deserialize, ToSchema, Debug)]
pub struct S3KeyReq { pub key: String }

#[derive(Deserialize, ToSchema, Debug)]
pub struct SmtpTestReq {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub is_html: Option<bool>, // 👈 Now optional in JSON
}

#[derive(Template)]
#[template(path = "diagnostics.html")]
struct DiagTemplate { db_alive: bool }

// --- HANDLERS ---

#[handler]
pub async fn render_page(res: &mut Response) -> AppResult<()> {
    let db_alive = db::SQLX_POOL.get().map(|p| !p.is_closed()).unwrap_or(false);
    let tmpl = DiagTemplate { db_alive };
    let html = tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?;
    res.render(Text::Html(html));
    Ok(())
}

#[endpoint(tags("diagnostics"))]
pub async fn handle_s3_upload(req: &mut Request) -> AppResult<Json<LocalUploadResult>> {
    let bytes = req.payload().await
        .map_err(|_| AppError::Internal("RAM Stream Failed".into()))?.to_vec();

    let filename = req.header::<String>("x-filename").unwrap_or_else(|| "upload.bin".into());
    let content_type = req.content_type().map(|c| c.to_string()).unwrap_or_else(|| "application/octet-stream".into());
    let key = format!("diagnostics/{}", filename);

    let result = s3::get().upload(&key, bytes, &content_type).await?;
    Ok(Json(LocalUploadResult { url: result.url, key }))
}

#[endpoint(tags("diagnostics"))]
pub async fn handle_s3_get_url(key: QueryParam<String>) -> AppResult<Json<String>> {
    Ok(Json(s3::get().get_url(&key.into_inner())))
}

#[endpoint(tags("diagnostics"))]
pub async fn handle_s3_delete(body: JsonBody<S3KeyReq>) -> AppResult<StatusCode> {
    s3::get().delete(&body.key).await?;
    Ok(StatusCode::OK)
}

#[handler]
pub async fn handle_db_query(idata: JsonBody<QueryReq>, res: &mut Response) -> AppResult<()> {
    let pool = db::pool();
    let sql_string = idata.into_inner().sql;

    // ✅ Bypasses E0277 (SqlSafeStr) and E0597 (Lifetime)
    let raw_query = AssertSqlSafe(sql_string).into_sql_str();

    pool.execute(raw_query).await
        .map_err(|e| AppError::Internal(format!("SQLx Error: {}", e)))?;

    res.render(Json("Query Successful".to_string()));
    Ok(())
}

#[endpoint(tags("diagnostics"))]
pub async fn handle_smtp_send(body: JsonBody<SmtpTestReq>) -> AppResult<Json<String>> {
    let data = body.into_inner();

    // Assuming your mail service has a send method
    smtp::get()
        .send_email(&data.to, &data.subject, &data.body,data.is_html )
        .await
        .map_err(|e| AppError::Internal(format!("SMTP Error: {}", e)))?;

    Ok(Json("Email sent successfully".into()))
}