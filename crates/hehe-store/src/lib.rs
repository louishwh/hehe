pub mod error;
pub mod local;
pub mod traits;
pub mod router;

pub use error::{Result, StoreError};
pub use router::{StoreHealth, StoreRouter};

pub use traits::{
    CacheStore, 
    Document, IndexSchema, SearchFilter, SearchHit, SearchStore,
    Migration, RelationalStore, Row, Transaction,
    CollectionInfo, SearchResult, VectorFilter, VectorRecord, VectorStore,
};

#[cfg(feature = "sqlite")]
pub use local::SqliteStore;

#[cfg(feature = "memory-cache")]
pub use local::MemoryCache;

pub use local::MemoryVectorStore;

#[cfg(feature = "sqlite")]
pub use local::SqliteFtsStore;

pub mod prelude {
    pub use crate::error::{Result, StoreError};
    pub use crate::router::{StoreHealth, StoreRouter};
    pub use crate::traits::{
        CacheStore, RelationalStore, SearchStore, VectorStore,
        Migration, Row,
        VectorRecord, SearchResult, VectorFilter,
        Document, SearchHit, IndexSchema,
    };
}
