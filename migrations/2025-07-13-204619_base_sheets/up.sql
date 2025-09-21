CREATE TABLE timesheet (
    day text not null primary key, -- 'YYYY-MM-dd'
    status text not null check (status in ('OPEN', 'EXPORTED'))
);

CREATE TABLE time_entry (
    id text not null primary key, -- 'TypeID ent_'

    timesheet_day text not null, -- 'YYYY-MM-dd'

    start_time text not null, -- 'HH:mm'
    duration_mins int not null,
    description text not null,

    project_key text not null,
    ticket_key text null,

    foreign key(timesheet_day) references timesheet(day)
);
