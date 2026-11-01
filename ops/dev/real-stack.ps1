<#
.SYNOPSIS
  Monte la PILE RÉELLE d'Immo Suite sur le poste, joue les parcours de navigateur dessus, puis la démonte (étape 02).

.DESCRIPTION
  1. base jetable sur le PostgreSQL local (port 5433), rôles de production (ops/sql/roles.sql), migrations (immo migrate) ;
  2. comptes et organisation créés par la ligne de commande (immo user create, immo org create) ;
  3. serveur Rust (immo serve) et front (vite preview, qui relaie /api) lancés par task.ps1 ;
  4. Playwright (web/e2e-real) à 390 px et 1440 px ;
  5. arrêt des deux serveurs par task.ps1 et suppression de la base.

  Prérequis : PostgreSQL local démarré (spikes/REPRISE.md), Rust GNU, Node, Chrome, `cargo build -p immo-server`.
  Ne touche ni au PostgreSQL du port 5432 ni à Neon. Aucun secret n'est écrit dans le dépôt : tout est tiré au hasard à chaque essai.

.PARAMETER ShortIdle
  Lance le serveur avec SESSION_IDLE_MINUTES=1 et joue aussi le test d'expiration d'inactivité (environ 70 s de plus).

.PARAMETER KeepStack
  Laisse la pile en marche à la fin (pour une démonstration à la main) : à arrêter ensuite par task.ps1 stop (noms real-api, real-web).
#>
param([switch]$ShortIdle, [switch]$KeepStack)

$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$task = Join-Path $repo 'spikes\tools\task.ps1'
$bin = Join-Path $repo 'target\debug\immo.exe'
$psql = if ($env:PSQL) { $env:PSQL } else { 'C:\Program Files\PostgreSQL\18\bin\psql.exe' }
if (-not (Test-Path $bin)) { throw "binaire absent : cargo build -p immo-server" }
if (-not (Test-Path $psql)) { throw "psql introuvable ($psql)" }

function Pw { -join ((48..57 + 97..122) | Get-Random -Count 28 | ForEach-Object { [char]$_ }) }
$pw = @{ owner = Pw; app = Pw; platform = Pw; backup = Pw }
$userPassword = 'un mot de passe correct'
$db = 'immo_real_e2e'
$apiPort = 18080
$webPort = 4173

function Psql([string]$database, [string[]]$arguments, [string]$user = 'postgres', [string]$password = $null) {
    if ($password) { $env:PGPASSWORD = $password } else { Remove-Item Env:PGPASSWORD -ErrorAction SilentlyContinue }
    & $psql -h 127.0.0.1 -p 5433 -U $user -d $database -v ON_ERROR_STOP=1 -q -X -At @arguments
    if ($LASTEXITCODE -ne 0) { throw "psql a échoué ($database)" }
}

$passwordFile = Join-Path $env:TEMP "immo-real-e2e-$([guid]::NewGuid().ToString('N')).txt"
$started = @()
$exit = 1
try {
    Write-Output "== base jetable $db"
    Psql postgres @('-c', "DROP DATABASE IF EXISTS $db")
    Psql postgres @('-c', "CREATE DATABASE $db")
    Psql $db @('-v', "owner_pw=$($pw.owner)", '-v', "app_pw=$($pw.app)", '-v', "platform_pw=$($pw.platform)", '-v', "backup_pw=$($pw.backup)", '-f', (Join-Path $repo 'ops\sql\roles.sql'))

    Write-Output '== migrations'
    $env:MIGRATION_DATABASE_URL = "postgres://immo_owner:$($pw.owner)@127.0.0.1:5433/$db"
    & $bin migrate (Join-Path $repo 'migrations') 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'immo migrate a échoué' }

    Write-Output '== comptes et organisation par la ligne de commande'
    $env:APP_ENV = 'development'
    $env:DATABASE_URL = "postgres://immo_app:$($pw.app)@127.0.0.1:5433/$db"
    [IO.File]::WriteAllText($passwordFile, $userPassword + "`n", (New-Object System.Text.UTF8Encoding($false)))
    $accounts = @(@('+22670123456', 'Awa Kabore'), @('+22670123457', 'Issa Ouedraogo'), @('+22670123458', 'Compte bloque un'), @('+22670123459', 'Compte bloque deux'), @('+22670123460', 'Gerant securite un'), @('+22670123461', 'Gerant securite deux'))
    foreach ($a in $accounts) {
        & $bin user create --phone $a[0] --name $a[1] --password-file $passwordFile 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "immo user create a échoué ($($a[0]))" }
    }
    & $bin org create --name 'Agence Horizon' --plan agency --user '+22670123456' 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'immo org create a échoué' }
    # les autres membres : l'invitation n'existe qu'à l'étape 03 ; l'administrateur du poste les ajoute directement
    Psql $db @('-c', "INSERT INTO memberships (org_id, user_id, role) SELECT o.id, u.id, 'cashier' FROM organizations o, users u WHERE o.name = 'Agence Horizon' AND u.login_phone IN ('+22670123457', '+22670123458', '+22670123459')")
    # deux autres gérants (un par largeur) pour la double authentification et l'accès support de l'étape 03 : le gérant principal reste sans TOTP
    Psql $db @('-c', "INSERT INTO memberships (org_id, user_id, role) SELECT o.id, u.id, 'manager' FROM organizations o, users u WHERE o.name = 'Agence Horizon' AND u.login_phone IN ('+22670123460', '+22670123461')")
    # la création d'un compte est refusée en production
    $env:APP_ENV = 'production'
    $env:DATABASE_URL = "postgres://immo_app:$($pw.app)@127.0.0.1:5433/$db?sslmode=verify-full"
    $env:PUBLIC_BASE_URL = 'https://exemple.invalid'
    & $bin user create --phone '+22670999999' --name 'Refus attendu' 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) { throw 'immo user create aurait dû être refusé en production' }
    Write-Output "   création de compte refusée en production : oui"
    Remove-Item Env:PUBLIC_BASE_URL

    Write-Output '== serveur et front'
    $env:APP_ENV = 'development'
    $env:DATABASE_URL = "postgres://immo_app:$($pw.app)@127.0.0.1:5433/$db"
    $env:SESSION_SECRET = (Pw) + (Pw)
    # clé de chiffrement des champs (secret TOTP) : 64 caractères hexadécimaux tirés au hasard à chaque essai
    $env:FIELD_ENCRYPTION_KEY = -join ((1..64) | ForEach-Object { '{0:x}' -f (Get-Random -Maximum 16) })
    # étape 04 : route d'essai qui provoque une erreur 500 réelle (jamais en production : le serveur la refuse)
    $env:ENABLE_TEST_ROUTES = '1'
    $env:PUBLIC_BASE_URL = "http://127.0.0.1:$webPort"
    $env:BIND_ADDR = "127.0.0.1:$apiPort"
    $env:ALLOWED_ORIGINS = "http://127.0.0.1:$webPort"
    $env:RUST_LOG = 'info'
    if ($ShortIdle) { $env:SESSION_IDLE_MINUTES = '1' } else { Remove-Item Env:SESSION_IDLE_MINUTES -ErrorAction SilentlyContinue }
    & $task start -Name real-api -Persistent -TimeoutSec 3600 -ReadyPort $apiPort -Exe $bin -ArgList serve | Out-Null
    $started += 'real-api'
    # étape 04 : le travailleur envoie l'alerte d'un signalement (platform_events, rôle immo_platform)
    $env:PLATFORM_DATABASE_URL = "postgres://immo_platform:$($pw.platform)@127.0.0.1:5433/$db"
    & $task start -Name real-work -Persistent -TimeoutSec 3600 -Exe $bin -ArgList work | Out-Null
    $started += 'real-work'
    $env:API_URL = "http://127.0.0.1:$apiPort"
    & $task run -Name real-web-build -TimeoutSec 300 -WorkDir (Join-Path $repo 'web') -Exe cmd.exe -ArgList '/c', 'npm run build' | Out-Null
    & $task start -Name real-web -Persistent -TimeoutSec 3600 -ReadyPort $webPort -WorkDir (Join-Path $repo 'web') -Exe cmd.exe -ArgList '/c', "npx vite preview --host 127.0.0.1 --port $webPort --strictPort" | Out-Null
    $started += 'real-web'

    Write-Output '== parcours de navigateur sur la pile réelle (390 px et 1440 px)'
    $env:E2E_PASSWORD = $userPassword
    $env:E2E_PSQL = $psql
    $env:E2E_DB_URL = "postgres://postgres@127.0.0.1:5433/$db"
    $env:E2E_BASE_URL = "http://127.0.0.1:$webPort"
    if ($ShortIdle) { $env:E2E_SHORT_IDLE = '1' } else { Remove-Item Env:E2E_SHORT_IDLE -ErrorAction SilentlyContinue }
    & $task run -Name real-e2e -TimeoutSec 600 -WorkDir (Join-Path $repo 'web') -Exe cmd.exe -ArgList '/c', 'npm run test:e2e:real' 2>&1 | ForEach-Object { "$_" }
    $exit = if ($LASTEXITCODE -eq 0) { 0 } else { 1 }
    Write-Output '== observation laissée par les parcours (étape 04)'
    Psql $db @('-c', "SELECT name, count(*) FROM product_events GROUP BY name ORDER BY name")
    Psql $db @('-c', "SELECT source, route, occurrences FROM error_reports ORDER BY source, route")
    Psql $db @('-c', "SELECT reference, category, status FROM feedback_reports ORDER BY reference")
    Psql $db @('-c', "SELECT action, detail FROM platform_events WHERE action = 'feedback.received' ORDER BY id")
    Write-Output '== audit laissé par les parcours (comptes par type d''action)'
    Psql $db @('-c', "SELECT action, count(*) FROM audit_log GROUP BY action ORDER BY action")
}
finally {
    if (-not $KeepStack) {
        foreach ($name in $started) { & $task stop -Name $name | Out-Null }
        Remove-Item -LiteralPath $passwordFile -ErrorAction SilentlyContinue
        try { Psql postgres @('-c', "DROP DATABASE IF EXISTS $db") } catch { Write-Output "base non supprimée : $_" }
        Write-Output '== pile démontée, base supprimée'
    } else {
        Write-Output "== pile laissée en marche : http://127.0.0.1:$webPort (API 127.0.0.1:$apiPort), base $db ; arrêt : task.ps1 stop -Name real-api / real-work / real-web"
    }
    Remove-Item Env:PGPASSWORD -ErrorAction SilentlyContinue
}
exit $exit
