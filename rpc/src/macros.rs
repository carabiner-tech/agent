#[macro_export]
macro_rules! define_rpc {
    ($($variant:ident($req_type:ty, $res_type:ty)),* $(,)?) => {
        #[derive(Debug, Serialize, Deserialize)]
        pub enum RpcRequest {
            $($variant($req_type),)*
        }

        #[derive(Debug, Serialize, Deserialize, EnumAsInner)]
        pub enum RpcResponse {
            $($variant($res_type),)*
            RpcError(String),
        }

        impl RpcRequest {
            pub async fn process(self) -> RpcResponse {
                match self {
                    $(
                        RpcRequest::$variant(req) => {
                            match req.process().await {
                                Ok(resp) => RpcResponse::$variant(resp),
                                Err(e) => RpcResponse::RpcError(e.to_string()),
                            }
                        }
                    ),*
                }
            }
        }

        $(
            impl From<$req_type> for RpcRequest {
                fn from(req: $req_type) -> Self {
                    RpcRequest::$variant(req)
                }
            }

            impl From<$res_type> for RpcResponse {
                fn from(res: $res_type) -> Self {
                    RpcResponse::$variant(res)
                }
            }
        )*
    };
}
