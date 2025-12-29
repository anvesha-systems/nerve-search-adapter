use std::collections::HashSet;
use nerve_protocol::types::RequestId;

pub struct RequestState {
    cancelled: HashSet<RequestId>,
}

impl RequestState{
    pub fn new()->Self{
        Self{
            cancelled : HashSet::new(),
        }
    }

    pub fn cancel(&mut self, id:RequestId){
        self.cancelled.insert(id);
    }

    pub fn is_cancelled(&mut self, id: RequestId) -> bool {
        self.cancelled.contains(&id)
    }
}