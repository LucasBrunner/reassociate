use async_trait::async_trait;

use crate::DB;

#[async_trait]
pub trait Executable {
    type Result;
    async fn execute(self, db: &DB, variables: Option<()>, strict: bool) -> Self::Result;
}

#[async_trait]
impl Executable for &str {
    type Result = Result<Vec<surrealdb::dbs::Response>, surrealdb::error::Db>;

    async fn execute(self, (ds, ses): &DB, variables: Option<()>, strict: bool) -> Self::Result {
        ds.execute(self, ses, None, strict).await
    }
}

#[async_trait(?Send)]
pub trait ExecuteMany<T>
where
    T: Executable,
{
    async fn execute(self, db: &DB, variables: Option<()>, strict: bool) -> Vec<T::Result>;
}

#[async_trait(?Send)]
impl<T, U> ExecuteMany<T> for U
where
    T: Executable,
    U: IntoIterator<Item = T>,
{
    async fn execute(
        self,
        db: &DB,
        variables: Option<()>,
        strict: bool,
    ) -> Vec<<T as Executable>::Result> {
        let mut results = Vec::new();
        for executable in self.into_iter() {
            results.push(executable.execute(db, variables, strict).await);
        }
        results
    }
}

pub mod prelude {
    pub use super::Executable;
    pub use super::ExecuteMany;
}
