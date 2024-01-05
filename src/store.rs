use std::{
    any::TypeId,
    cell::RefCell,
    collections::{HashMap, VecDeque},
};

use crate::MockReturn;

#[doc(hidden)]
#[derive(Default)]
pub struct MockStore {
    // (fn type_id) -> return_value
    mocks: RefCell<HashMap<TypeId, VecDeque<MockReturn>>>,
}

impl MockStore {
    pub(crate) fn add(&self, id: TypeId, value: MockReturn) {
        {
            if let Some(returns) = self.mocks.borrow_mut().get_mut(&id) {
                returns.push_back(value);
                return;
            }
        }

        self.mocks.borrow_mut().insert(id, vec![value].into());
    }

    pub(crate) fn mock_exists(&self, id: TypeId) -> bool {
        self.mocks
            .borrow()
            .get(&id)
            .map(|m| !m.is_empty())
            .unwrap_or(false)
    }

    pub(crate) fn get(&self, id: TypeId) -> Option<MockReturn> {
        self.mocks
            .borrow_mut()
            .get_mut(&id)
            .and_then(|returns| returns.pop_front())
    }

    pub(crate) fn remove(&self, id: TypeId) {
        self.mocks.borrow_mut().remove(&id);
    }

    pub(crate) fn clear(&self) {
        self.mocks.borrow_mut().clear()
    }
}
