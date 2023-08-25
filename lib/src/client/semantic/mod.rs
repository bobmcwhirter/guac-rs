use crate::client::intrinsic::IntrinsicGuacClient;
use crate::client::GuacClient;

pub struct SemanticGuacClient<'c> {
    client: &'c GuacClient,
}

impl<'c> SemanticGuacClient<'c> {
    pub(crate) fn new(client: &'c GuacClient) -> Self {
        Self { client }
    }

    pub fn intrinsic(&self) -> IntrinsicGuacClient {
        IntrinsicGuacClient::new(self.client)
    }
}
