-- Grand Rounds playtest (SPEC-3 T3). Every table has RLS; players read only their own rows.
-- Saves are written ONLY through put_save (versioned); server effects arrive as ledger rows.

create table public.players (
  id uuid primary key references auth.users (id) on delete cascade,
  display_name text not null default ('Resident ' || floor(random() * 9000 + 1000)::int),
  level int not null default 1,
  shield_until timestamptz,
  new_until timestamptz not null default now() + interval '48 hours',
  created_at timestamptz not null default now()
);

create table public.saves (
  player_id uuid primary key references public.players (id) on delete cascade,
  contract int not null,
  text text not null check (length(text) < 200000),
  version int not null default 0,
  updated_at timestamptz not null default now()
);

-- Reviews and other server-side grants for a player; the client applies each once.
create table public.events (
  id bigint generated always as identity primary key,
  player_id uuid not null references public.players (id) on delete cascade,
  kind text not null,
  n int not null check (n >= 0),
  source text not null default 'game',
  day int,
  applied boolean not null default false,
  at timestamptz not null default now()
);
create index events_player_unapplied on public.events (player_id) where not applied;

create table public.bug_reports (
  id bigint generated always as identity primary key,
  player_id uuid not null references public.players (id) on delete cascade,
  text text not null check (length(text) between 1 and 4000),
  at timestamptz not null default now()
);

create table public.survey (
  player_id uuid primary key references public.players (id) on delete cascade,
  answers jsonb not null,
  at timestamptz not null default now()
);

alter table public.players enable row level security;
alter table public.saves enable row level security;
alter table public.events enable row level security;
alter table public.bug_reports enable row level security;
alter table public.survey enable row level security;

create policy "own player" on public.players for select to authenticated using (id = (select auth.uid()));
create policy "own save" on public.saves for select to authenticated using (player_id = (select auth.uid()));
create policy "own events" on public.events for select to authenticated using (player_id = (select auth.uid()));
create policy "file a bug" on public.bug_reports for insert to authenticated with check (player_id = (select auth.uid()));
create policy "answer survey" on public.survey for insert to authenticated with check (player_id = (select auth.uid()));

-- The one way to write a save: only when the caller's copy is current.
create function public.put_save(p_text text, p_contract int, p_base int, p_event_ids bigint[] default '{}')
returns jsonb
language plpgsql
security definer
set search_path = ''
as $$
declare
  me uuid := auth.uid();
  cur int;
begin
  if me is null then
    raise exception 'not signed in';
  end if;
  insert into public.players (id) values (me) on conflict (id) do nothing;
  select version into cur from public.saves where player_id = me for update;
  if cur is null then
    if p_base <> 0 then
      return jsonb_build_object('ok', false, 'conflict', true, 'version', 0);
    end if;
    insert into public.saves (player_id, contract, text, version) values (me, p_contract, p_text, 1);
    cur := 0;
  elsif cur <> p_base then
    return (select jsonb_build_object('ok', false, 'conflict', true, 'version', s.version, 'text', s.text)
            from public.saves s where s.player_id = me);
  else
    update public.saves set text = p_text, contract = p_contract, version = cur + 1, updated_at = now()
      where player_id = me;
  end if;
  update public.events set applied = true where player_id = me and id = any (p_event_ids);
  return jsonb_build_object('ok', true, 'version', cur + 1);
end;
$$;

-- Everything the game needs on Play: the save and what to apply.
create function public.get_state()
returns jsonb
language plpgsql
security definer
set search_path = ''
as $$
declare
  me uuid := auth.uid();
begin
  if me is null then
    raise exception 'not signed in';
  end if;
  insert into public.players (id) values (me) on conflict (id) do nothing;
  return jsonb_build_object(
    'save', (select jsonb_build_object('version', s.version, 'text', s.text, 'contract', s.contract)
             from public.saves s where s.player_id = me),
    'events', coalesce((select jsonb_agg(jsonb_build_object('id', e.id, 'kind', e.kind, 'n', e.n, 'day', e.day) order by e.id)
                        from public.events e where e.player_id = me and not e.applied), '[]'::jsonb)
  );
end;
$$;

revoke all on function public.put_save(text, int, int, bigint[]) from public, anon;
revoke all on function public.get_state() from public, anon;
grant execute on function public.put_save(text, int, int, bigint[]) to authenticated;
grant execute on function public.get_state() to authenticated;
