use serde::{Deserialize, Serialize};
use std::fmt;

const THEME_MODE_DARK_LABEL: &str = "Dark";
const THEME_MODE_LIGHT_LABEL: &str = "Light";
const AUTH_MODE_QUERY_LABEL: &str = "Query";
const AUTH_MODE_HEADER_LABEL: &str = "Header";
const HTTP_METHOD_POST_LABEL: &str = "POST";
const HTTP_METHOD_GET_LABEL: &str = "GET";
const HTTP_METHOD_PUT_LABEL: &str = "PUT";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl fmt::Display for ThemeMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ThemeMode::Dark => THEME_MODE_DARK_LABEL,
            ThemeMode::Light => THEME_MODE_LIGHT_LABEL,
        };
        formatter.write_str(label)
    }
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::Dark
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImageHostingAuthMode {
    Query,
    Header,
}

impl fmt::Display for ImageHostingAuthMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ImageHostingAuthMode::Query => AUTH_MODE_QUERY_LABEL,
            ImageHostingAuthMode::Header => AUTH_MODE_HEADER_LABEL,
        };
        formatter.write_str(label)
    }
}

impl Default for ImageHostingAuthMode {
    fn default() -> Self {
        ImageHostingAuthMode::Query
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImageUploadHttpMethod {
    Post,
    Get,
    Put,
}

impl fmt::Display for ImageUploadHttpMethod {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ImageUploadHttpMethod::Post => HTTP_METHOD_POST_LABEL,
            ImageUploadHttpMethod::Get => HTTP_METHOD_GET_LABEL,
            ImageUploadHttpMethod::Put => HTTP_METHOD_PUT_LABEL,
        };
        formatter.write_str(label)
    }
}

impl Default for ImageUploadHttpMethod {
    fn default() -> Self {
        ImageUploadHttpMethod::Post
    }
}
