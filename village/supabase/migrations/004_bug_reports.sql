-- Bug reports from the game: the player id fills itself in; the save travels along to reproduce it.
alter table public.bug_reports alter column player_id set default auth.uid();
alter table public.bug_reports add column save text check (length(save) <= 20000);
