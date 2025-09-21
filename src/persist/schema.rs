// @generated automatically by Diesel CLI.

diesel::table! {
    time_entry (id) {
        id -> Text,
        timesheet_day -> Text,
        start_time -> Text,
        duration_mins -> Integer,
        description -> Text,
        project_key -> Text,
        ticket_key -> Nullable<Text>,
    }
}

diesel::table! {
    timesheet (day) {
        day -> Text,
        status -> Text,
    }
}

diesel::joinable!(time_entry -> timesheet (timesheet_day));

diesel::allow_tables_to_appear_in_same_query!(time_entry, timesheet,);
