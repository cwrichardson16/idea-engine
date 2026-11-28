-- A restore code moves a village: saves from sign-ins made before the move are refused.
alter table public.players add column moved_at timestamptz;
create or replace function public.put_save(p_text text, p_contract int, p_base int, p_event_ids bigint[] default '{}')
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
  -- A village moved by a restore code: sign-ins from before the move can't save.
  if exists (select 1 from public.players p where p.id = me and p.moved_at is not null
             and to_timestamp((auth.jwt() ->> 'iat')::double precision) < date_trunc('second', p.moved_at)) then
    raise exception 'village moved to another device' using errcode = '42501';
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
revoke all on function public.put_save(text, int, int, bigint[]) from public, anon;
grant execute on function public.put_save(text, int, int, bigint[]) to authenticated;
