// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
        }

        impl From<Uuid> for $name {
            fn from(value: Uuid) -> Self {
                Self(value)
            }
        }

        impl From<$name> for Uuid {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

define_id!(WorkspaceId);
define_id!(UserId);
define_id!(WorkspaceUserId);
define_id!(PartyId);
define_id!(AccountId);
define_id!(JournalEntryId);
define_id!(JournalLineId);
define_id!(InvoiceId);
define_id!(InvoiceLineId);
define_id!(DealId);
define_id!(DealStageId);
define_id!(EmployeeId);
define_id!(ActivityId);
