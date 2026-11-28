-- A restore code moves a village: saves from sign-ins made before the move are refused.
alter table public.players add column moved_at timestamptz;
-- put_save gains, right after the signed-in check:
--   if exists (select 1 from public.players p where p.id = me and p.moved_at is not null
--              and to_timestamp((auth.jwt() ->> 'iat')::double precision) < date_trunc('second', p.moved_at)) then
--     raise exception 'village moved to another device' using errcode = '42501';
--   end if;
-- (applied to the test project as migration "restore_signs_out" with the full function body)
