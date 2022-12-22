// @generated automatically by Diesel CLI.

diesel::table! {
    dimensions (dimension) {
        uuid -> Uuid,
        dimension -> Varchar,
        priority -> Int4,
        last_modified -> Timestamptz,
        created_on -> Timestamptz,
    }
}

diesel::table! {
    global_config (key) {
        uuid -> Uuid,
        key -> Varchar,
        value -> Json,
        last_modified -> Timestamptz,
        created_on -> Timestamptz,
    }
}

diesel::table! {
    overrides (key) {
        key -> Varchar,
        value -> Json,
        last_modified -> Timestamptz,
        created_on -> Timestamptz,
    }
}

diesel::table! {
    contexts (key) {
        key -> Varchar,
        value -> Json,
        last_modified -> Timestamptz,
        created_on -> Timestamptz,
    }
}

diesel::table! {
    ctxoverrides (key) {
        key -> Varchar,
        context_id -> Varchar,
        override_id -> Varchar,
        last_modified -> Timestamptz,
        created_on -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    contexts,
    dimensions,
    global_config,
    overrides,
    ctxoverrides
);