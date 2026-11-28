-- Restore codes (SPEC-3 T3): one-time codes to move a player's account to another device.
-- Only the edge function (service role) touches these tables; RLS on, no policies.
create table public.restore_codes (
  player_id uuid primary key references public.players (id) on delete cascade,
  salt text not null,
  code_hash text not null,
  created_at timestamptz not null default now()
);
create table public.restore_attempts (
  id bigint generated always as identity primary key,
  who text not null,
  at timestamptz not null default now()
);
create index restore_attempts_who_at on public.restore_attempts (who, at);
alter table public.restore_codes enable row level security;
alter table public.restore_attempts enable row level security;
