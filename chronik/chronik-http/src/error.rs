// Copyright (c) 2023 The Bitcoin developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

//! Module for [`ReportError`].

use abc_rust_error::{parse_error_status, Report};
use axum::response::{IntoResponse, Response};
use chronik_util::{log, log_chronik};
use hyper::StatusCode;

use crate::{proto, protobuf::Protobuf, server::ChronikServerError};

/// Wrapper around [`Report`] which can be converted into a [`Response`].
#[derive(Debug)]
pub struct ReportError(pub Report);

impl From<Report> for ReportError {
    fn from(err: Report) -> Self {
        ReportError(err)
    }
}

impl From<ChronikServerError> for ReportError {
    fn from(err: ChronikServerError) -> Self {
        ReportError(err.into())
    }
}

impl IntoResponse for ReportError {
    fn into_response(self) -> Response {
        let ReportError(report) = self;
        let msg = report.to_string();
        let (code, response_msg) = match parse_error_status(&msg) {
            None => {
                // Unknown internal error: don't expose potential
                // vulnerabilities through HTTP, only log to node.
                log_chronik!("{report:?}");
                log!("Chronik HTTP server got an unknown error: {report:#}");
                let unknown_msg = "Unknown error, contact admins".to_string();
                (StatusCode::INTERNAL_SERVER_ERROR, unknown_msg)
            }
            Some(status) if status.is_server_error() => {
                // Internal server error, but explicitly marked with "5xx: ", so
                // we expose it through HTTP (and also log to node).
                log_chronik!("{report:?}");
                log!(
                    "Chronik HTTP server got an internal server error: \
                     {report:#}"
                );
                (status, msg)
            }
            Some(status) => {
                // "Normal" error (400, 404, etc.), expose, but don't log.
                (status, msg)
            }
        };
        let proto_response = proto::Error { msg: response_msg };
        (code, Protobuf(proto_response)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use abc_rust_error::Result;
    use axum::response::IntoResponse;
    use hyper::{body::to_bytes, StatusCode};
    use prost::Message;
    use thiserror::Error;

    use crate::{error::ReportError, proto};

    #[tokio::test]
    async fn test_report_error() -> Result<()> {
        #[derive(Debug, Error)]
        enum TestError {
            #[error("Something obsure")]
            Obsure,
            #[error("500: Cable eaten by cat")]
            CableEaten,
            #[error("501: Never implemented")]
            NeverImplemented,
            #[error("400: Request encoding not code page 65001")]
            NotCp65001,
        }

        async fn make_error_response(
            err: TestError,
        ) -> Result<(StatusCode, proto::Error)> {
            let report_err = ReportError(err.into());
            let response = report_err.into_response();
            let status = response.status();
            let body = to_bytes(response.into_body()).await?;
            let proto_error = proto::Error::decode(body)?;
            Ok((status, proto_error))
        }

        assert_eq!(
            make_error_response(TestError::Obsure).await?,
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                proto::Error {
                    msg: "Unknown error, contact admins".to_string(),
                },
            ),
        );

        assert_eq!(
            make_error_response(TestError::CableEaten).await?,
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                proto::Error {
                    msg: "500: Cable eaten by cat".to_string(),
                },
            ),
        );

        assert_eq!(
            make_error_response(TestError::NeverImplemented).await?,
            (
                StatusCode::NOT_IMPLEMENTED,
                proto::Error {
                    msg: "501: Never implemented".to_string(),
                },
            ),
        );

        assert_eq!(
            make_error_response(TestError::NotCp65001).await?,
            (
                StatusCode::BAD_REQUEST,
                proto::Error {
                    msg: "400: Request encoding not code page 65001"
                        .to_string(),
                },
            ),
        );

        Ok(())
    }
}
