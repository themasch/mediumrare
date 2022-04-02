use std::cell::RefCell;
use std::collections::HashMap;
use crate::client::QueryResponse;
use crate::PostDataClient;

pub struct MockClient {
    response_map: RefCell<HashMap<String, Result<QueryResponse, ()>>>
}

impl MockClient {
    fn set_mock_post_data<T: Into<String>>(&self, post_id: T, result: Result<QueryResponse, ()>) {
        self.response_map.borrow_mut().insert(post_id.into(), result);
    }
}

impl PostDataClient for MockClient {
    fn get_post_data(&self, post_id: &str) -> Result<QueryResponse, ()> {
        self.response_map.borrow()[post_id].clone()
    }
}