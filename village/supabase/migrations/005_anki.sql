-- Anki link (SPEC-3 T4). The add-on posts review counts with a pairing code;
-- the honour button logs phone reviews. Each review day credits the LARGER of
-- the two counts (never the sum), as `review` events the game applies once.
create table public.pairings (
  player_id uuid primary key references public.players (id) on delete cascade,
  salt text not null,
  code_hash text not null,
  revoked boolean not null default false,
  last_review_id bigint not null default 0,   -- newest Anki revlog id counted (ms)
  baseline jsonb,                             -- {day: count} for the 30 days before pairing
  created_at timestamptz not null default now()
);
create table public.day_totals (
  player_id uuid not null references public.players (id) on delete cascade,
  day int not null,
  anki int not null default 0,
  honour int not null default 0,
  credited int not null default 0,
  primary key (player_id, day)
);
alter table public.pairings enable row level security;
alter table public.day_totals enable row level security;
create policy "own days" on public.day_totals for select to authenticated using (player_id = (select auth.uid()));

-- Credits a day's new total (max of anki/honour, capped) as one review event.
-- Called only by the two functions below.
create function public.credit_day(p_player uuid, p_day int, p_anki int, p_honour int)
returns int
language plpgsql
security definer
set search_path = ''
as $$
declare
  row public.day_totals;
  want int;
  rolling int;
  delta int;
begin
  insert into public.day_totals (player_id, day) values (p_player, p_day) on conflict do nothing;
  select * into row from public.day_totals where player_id = p_player and day = p_day for update;
  row.anki := greatest(row.anki, least(coalesce(p_anki, 0), 1500));
  row.honour := greatest(row.honour, least(coalesce(p_honour, 0), 200));
  want := least(greatest(row.anki, row.honour), 1500);
  delta := greatest(want - row.credited, 0);
  -- Rolling 24 h cap on the server's own clock.
  select coalesce(sum(n), 0) into rolling from public.events
    where player_id = p_player and kind = 'review' and at > now() - interval '24 hours';
  delta := least(delta, greatest(1500 - rolling, 0));
  update public.day_totals set anki = row.anki, honour = row.honour, credited = row.credited + delta
    where player_id = p_player and day = p_day;
  if delta > 0 then
    insert into public.events (player_id, kind, n, source, day) values (p_player, 'review', delta, 'study', p_day);
  end if;
  return delta;
end;
$$;
revoke all on function public.credit_day(uuid, int, int, int) from public, anon, authenticated;
grant execute on function public.credit_day(uuid, int, int, int) to service_role;

-- The honour button: today's phone reviews (capped at 200 by credit_day).
create function public.log_reviews(p_day int, p_n int)
returns jsonb
language plpgsql
security definer
set search_path = ''
as $$
declare
  me uuid := auth.uid();
  got int;
begin
  if me is null then raise exception 'not signed in'; end if;
  if p_n < 0 or p_n > 200 then raise exception 'between 0 and 200'; end if;
  -- Only today or yesterday (a late-night entry), never far back.
  if p_day < (extract(epoch from now()) / 86400)::int - 2 or p_day > (extract(epoch from now()) / 86400)::int + 1 then
    raise exception 'that day is out of range';
  end if;
  got := public.credit_day(me, p_day, null, p_n);
  return jsonb_build_object('ok', true, 'counted', got);
end;
$$;
revoke all on function public.log_reviews(int, int) from public, anon;
grant execute on function public.log_reviews(int, int) to authenticated;
