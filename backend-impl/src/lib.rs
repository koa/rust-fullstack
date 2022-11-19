use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub struct Query;

#[Object]
impl Query {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}
pub type GraphqlSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub fn create_schema() -> GraphqlSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription).finish()
}

#[cfg(test)]
mod tests {}
