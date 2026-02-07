#[cfg(test)]
mod tests {
    use crate::error::{ApiError, ErrorCode};

    #[test]
    fn sqlx_row_not_found_maps_to_not_found() {
        let err: ApiError = sqlx::Error::RowNotFound.into();
        assert_eq!(err.code, ErrorCode::NotFound);
    }
}
