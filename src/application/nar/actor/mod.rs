mod runner;

pub use runner::{NarActor, NarRequest, ResolveNarInfoResponse};

use selector4nix_actor::registry::Registry;

use crate::domain::nar_info::model::StorePathHash;

pub type NarActorRegistry = Registry<StorePathHash, NarActor>;
