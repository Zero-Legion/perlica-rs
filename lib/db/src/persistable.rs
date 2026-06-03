use crate::error::Result;
use crate::saves::PlayerDb;

pub trait Persistable {
    fn persist(
        &self,
        uid: &str,
        db: &PlayerDb,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}
