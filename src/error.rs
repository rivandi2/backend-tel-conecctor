use thiserror::Error;

#[derive(Error, Debug)]
pub enum JiraError {
    #[error("Placehold error")] ProjectFound(#[from] reqwest::Error),
    #[error("Request dapat ditemukan")] RequestFail,
    #[error("Tidak dapat diubah ke JSON text")] TextChange,
    #[error("Tidak bisa masukin ke vector")] VectorFail(#[from] serde_json::Error),
}