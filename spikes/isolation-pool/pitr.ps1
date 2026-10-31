# Prototype (b), étape 7 du protocole : restauration à un instant donné sur Neon (branche à T0).
# Aucun secret affiché : les URL viennent de %LOCALAPPDATA%\immo-spikes-neon\.env.local (hors dépôt).
$ErrorActionPreference = 'Stop'
$env:Path = [Environment]::GetEnvironmentVariable('Path', 'User') + ';' + [Environment]::GetEnvironmentVariable('Path', 'Machine')
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$psql = 'C:\Program Files\PostgreSQL\18\bin\psql.exe'
$project = 'royal-lab-36803492'
$url = ((Get-Content "$env:LOCALAPPDATA\immo-spikes-neon\.env.local" | Where-Object { $_ -match '^NEON_B_ADMIN_URL=' }) -replace '^NEON_B_ADMIN_URL=', '')

"== 1. 100 lignes écrites, puis instant T0 lu sur le serveur"
& $psql $url -qAt -c "DROP TABLE IF EXISTS pitr_marker; CREATE TABLE pitr_marker (n int PRIMARY KEY, batch text NOT NULL); INSERT INTO pitr_marker SELECT g, 'avant-T0' FROM generate_series(1,100) g;"
$t0 = (& $psql $url -qAt -c "select to_char(clock_timestamp() at time zone 'utc', 'YYYY-MM-DD') || 'T' || to_char(clock_timestamp() at time zone 'utc', 'HH24:MI:SS.MS') || 'Z'").Trim()
"T0 = $t0"
Start-Sleep -Seconds 8
"== 2. 50 lignes écrites APRÈS T0"
& $psql $url -qAt -c "INSERT INTO pitr_marker SELECT g, 'apres-T0' FROM generate_series(101,150) g;"
"spike-b : " + (& $psql $url -qAt -c "select count(*) || ' lignes (' || count(*) filter (where batch='apres-T0') || ' apres T0)' from pitr_marker")

"== 3. branche créée à T0 : API Neon (parent_id + parent_timestamp), le CLI ne combine pas la branche et l'instant"
$parentId = (& neon branches list --project-id $project --profile agent 2>&1 | Out-String) -split "`n" | Where-Object { $_ -match '^spike-b\s' } | ForEach-Object { ($_ -split '\s+')[1] }
$body = @{ branch = @{ name = 'spike-b-t0'; parent_id = $parentId; parent_timestamp = $t0 }; endpoints = @(@{ type = 'read_write' }) } | ConvertTo-Json -Depth 5 -Compress
$bodyFile = Join-Path $env:TEMP 'pitr-body.json'; Set-Content -Path $bodyFile -Value $body -Encoding ascii
$sw = [Diagnostics.Stopwatch]::StartNew()
$out = & neon api "/projects/$project/branches" -X POST -d "@$bodyFile" --profile agent 2>&1 | Out-String
$sw.Stop(); Remove-Item $bodyFile -ErrorAction SilentlyContinue
try { $j = $out | ConvertFrom-Json; "branche : $($j.branch.name) ($($j.branch.id)), état $($j.branch.current_state), parent $($j.branch.parent_id) à $($j.branch.parent_timestamp)" } catch { ($out -replace 'postgres(ql)?://\S+', '<url masquée>').Substring(0, [Math]::Min(500, $out.Length)) }
"durée de la requête de création : {0:N1} s" -f $sw.Elapsed.TotalSeconds
"== 4. contenu de la branche à T0"
$cs = (& neon connection-string spike-b-t0 --project-id $project --database-name neondb --role-name neondb_owner --profile agent 2>&1 | Out-String).Trim()
$sw2 = [Diagnostics.Stopwatch]::StartNew()
$r = & $psql $cs -qAt -c "select count(*) || ' lignes (' || count(*) filter (where batch='apres-T0') || ' apres T0)' from pitr_marker" 2>&1
"branche à T0 : $r (première requête, réveil du calcul compris : {0:N1} s)" -f $sw2.Elapsed.TotalSeconds


