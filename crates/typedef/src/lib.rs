use salvo::prelude::*;
use serde::Serialize;
use app_error::AppError;

#[derive(Serialize, ToSchema, Clone, Copy, Debug)]
pub struct Empty {}

pub type AppResult<T> = Result<T, AppError>;
pub type JsonResult<T> = Result<Json<T>, AppError>;
pub type EmptyResult = Result<Json<Empty>, AppError>;