<#
.SYNOPSIS
  Gestion des tâches longues des prototypes (spikes/) : délai maximal, arrêt de l'arbre de processus,
  plafond de taille des journaux, code de sortie enregistré, sorties récupérables, audit et nettoyage.

.DESCRIPTION
  Verbes :
    run     exécute une commande au premier plan avec délai, plafond de journal, nettoyage des restes
    start   la même chose en arrière-plan : un superviseur détaché exécute « run » ; rend la main tout de suite
    status  état des tâches enregistrées (en cours, terminée, expirée, arrêtée, perdue)
    stop    arrête une tâche enregistrée (arbre complet), après vérification qu'il s'agit bien d'elle
    audit   lecture seule : tâches, processus des prototypes, ports, fichiers de sortie démesurés
    clean   propose (par défaut) ou applique (-Apply) l'arrêt des seuls processus clairement abandonnés

  Les journaux vont dans des FICHIERS (jamais des tubes) : un processus enfant qui garde un tube
  ouvert, comme PostgreSQL lancé par pg_ctl, ne peut plus bloquer l'appelant.
  Jamais d'interpréteur lancé sans script : pas de « python - » ni de « python » nu (voir README.md).

  État : spikes/out/tasks/<nom>.json, <nom>.out.log, <nom>.err.log (dossier hors Git).

.EXAMPLE
  .\task.ps1 run -Name build-b -TimeoutSec 900 -Exe cargo -ArgList build,--release -WorkDir ..\isolation-pool
  .\task.ps1 start -Name neon-c -TimeoutSec 7200 -Exe .\run-c.cmd
  .\task.ps1 start -Name srv-c-neon -Persistent -ReadyPort 3001 -Exe C:\...\spike-numerotation.exe -ArgList serve,neon,3001
  .\task.ps1 start -Name pg-spikes -Daemonizes -ReadyPort 5433 -Exe pg_ctl.exe -ArgList -D,<data>,-l,<log>,start
#>
param(
  [Parameter(Position = 0, Mandatory = $true)][ValidateSet('run', 'start', 'status', 'stop', 'audit', 'clean')][string]$Verb,
  [string]$Name,
  [string]$Exe,
  [string[]]$ArgList = @(),
  [string]$ArgJson,                 # arguments en JSON (ancien mode, conservé pour compatibilité)
  [string]$ArgFile,                 # fichier JSON des arguments (transmis au superviseur de « start » : aucun souci de guillemets)
  [string]$WorkDir = (Get-Location).Path,
  [int]$TimeoutSec = 600,           # délai maximal ; ignoré avec -Persistent
  [int]$MaxLogMB = 200,             # au-delà, la tâche est arrêtée (cause : boucle d'erreurs infinie)
  [switch]$Persistent,              # serveur volontairement durable : pas de délai
  [int]$ReadyPort = 0,              # port à attendre en écoute avant de déclarer « prêt »
  [int]$ReadyTimeoutSec = 60,
  [switch]$Daemonizes,              # la commande lance un service puis se termine (pg_ctl start)
  [int]$TailLines = 25,
  [switch]$Apply
)
$ErrorActionPreference = 'Stop'
$State = Join-Path (Split-Path $PSScriptRoot -Parent) 'out\tasks'
New-Item -ItemType Directory -Force $State | Out-Null

function Read-State($n) { $f = Join-Path $State "$n.json"; if (Test-Path $f) { Get-Content $f -Raw | ConvertFrom-Json } }
function Write-State($n, $o) { ($o | ConvertTo-Json -Depth 5) | Set-Content (Join-Path $State "$n.json") -Encoding UTF8 }
function Alive($procId, $started) {
  $p = Get-Process -Id $procId -ErrorAction SilentlyContinue
  if (-not $p) { return $false }
  if (-not $started) { return $true }
  # même PID ET même heure de démarrage : protège contre la réutilisation d'un numéro de processus.
  # Incident du 01/11/2026 : le numéro d'un superviseur terminé a été réutilisé par un processus protégé dont l'heure de démarrage est
  # illisible (StartTime nul) ; l'ancien code plantait. Une identité non prouvée n'est jamais « vivante » : on ne touche à rien.
  $startTime = $null
  try { $startTime = $p.StartTime } catch { $startTime = $null }
  if (-not $startTime) { return $false }
  return [bool]([math]::Abs(($startTime.ToUniversalTime() - ([datetime]$started).ToUniversalTime()).TotalSeconds) -lt 2)
}
function Descendants($rootId) {
  # Descendants PROUVÉS : un enfant n'est retenu que s'il a été créé APRÈS son parent. Sans cette vérification, la
  # réutilisation des numéros de processus rattache à tort des processus sans lien (incident du 31/10/2026).
  $all = @(Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, CreationDate)
  $byId = @{}; foreach ($x in $all) { $byId[[int]$x.ProcessId] = $x }
  if (-not $byId.ContainsKey([int]$rootId)) { return @() }
  $out = New-Object System.Collections.ArrayList; $q = New-Object System.Collections.Queue; $q.Enqueue([int]$rootId)
  $seen = @{ ([int]$rootId) = $true }
  while ($q.Count) {
    $c = $q.Dequeue(); $parent = $byId[$c]
    foreach ($x in $all) {
      if ([int]$x.ParentProcessId -eq $c -and [int]$x.ProcessId -ne $c -and -not $seen.ContainsKey([int]$x.ProcessId) -and $x.CreationDate -and $parent.CreationDate -and $x.CreationDate -ge $parent.CreationDate) {
        $seen[[int]$x.ProcessId] = $true; [void]$out.Add([int]$x.ProcessId); $q.Enqueue([int]$x.ProcessId)
      }
    }
  }
  return $out
}
function Stop-Tree($rootId) {
  # arrêt des seuls descendants prouvés (enfants d'abord), puis de la racine ; jamais taskkill /T
  $kids = @(Descendants $rootId)
  [array]::Reverse($kids)
  foreach ($k in $kids) { Stop-Process -Id $k -Force -ErrorAction SilentlyContinue }
  Stop-Process -Id $rootId -Force -ErrorAction SilentlyContinue
}
function Tail($f, $n) { if (Test-Path $f) { Get-Content $f -Tail $n -Encoding UTF8 -ErrorAction SilentlyContinue } }   # les prototypes écrivent en UTF-8
function LogMB($n) { $s = 0; foreach ($e in 'out', 'err') { $f = Join-Path $State "$n.$e.log"; if (Test-Path $f) { $s += (Get-Item $f).Length } }; return [math]::Round($s / 1MB, 1) }
function PortOwner($port) { (Get-NetTCPConnection -State Listen -LocalPort $port -ErrorAction SilentlyContinue | Select-Object -First 1).OwningProcess }

# ───────────────────────── run ─────────────────────────
function Verb-Run {
  if (-not $Name -or -not $Exe) { throw '-Name et -Exe sont obligatoires' }
  # ConvertFrom-Json (Windows PowerShell 5.1) rend un tableau comme UN SEUL objet : on l'énumère explicitement
  $parsed = $null
  if ($ArgFile) { $parsed = Get-Content $ArgFile -Raw -Encoding UTF8 | ConvertFrom-Json } elseif ($ArgJson) { $parsed = $ArgJson | ConvertFrom-Json }
  if ($ArgFile -or $ArgJson) { $ArgList = @(); foreach ($x in $parsed) { $ArgList += [string]$x } }
  $outLog = Join-Path $State "$Name.out.log"; $errLog = Join-Path $State "$Name.err.log"
  Remove-Item $outLog, $errLog -ErrorAction SilentlyContinue
  $st = [ordered]@{ name = $Name; status = 'running'; exe = $Exe; args = $ArgList; kind = $(if ($Persistent) { 'server' } else { 'task' }); supervisor = $PID
    supervisor_started = (Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o'); pid = $null; started = (Get-Date).ToUniversalTime().ToString('o')
    deadline = $(if ($Persistent -or $Daemonizes) { $null } else { (Get-Date).AddSeconds($TimeoutSec).ToUniversalTime().ToString('o') }); ready_port = $ReadyPort; exit_code = $null; ended = $null; note = '' }
  Write-State $Name $st
  $sp = @{ FilePath = $Exe; WorkingDirectory = $WorkDir; RedirectStandardOutput = $outLog; RedirectStandardError = $errLog; PassThru = $true; WindowStyle = 'Hidden' }
  if ($ArgList.Count) { $sp.ArgumentList = @($ArgList | ForEach-Object { if ($_ -match '\s' -and $_ -notmatch '^".*"$') { '"' + $_ + '"' } else { $_ } }) }
  $p = Start-Process @sp
  $null = $p.Handle   # sans cet accès, ExitCode reste nul dans Windows PowerShell 5.1
  $st.pid = $p.Id; $st.pid_started = $p.StartTime.ToUniversalTime().ToString('o'); Write-State $Name $st
  $seen = New-Object System.Collections.Generic.HashSet[int]
  $t0 = Get-Date; $ready = ($ReadyPort -eq 0)
  while (-not $p.HasExited) {
    foreach ($d in Descendants $p.Id) { [void]$seen.Add($d) }
    if (-not $ready -and (PortOwner $ReadyPort)) { $ready = $true; $st.status = 'ready'; Write-State $Name $st }
    if (-not $Persistent -and -not $Daemonizes -and ((Get-Date) - $t0).TotalSeconds -gt $TimeoutSec) { $st.status = 'timeout'; $st.note = "délai de $TimeoutSec s dépassé"; break }
    if ((LogMB $Name) -gt $MaxLogMB) { $st.status = 'log_overflow'; $st.note = "journaux au-delà de $MaxLogMB Mo (boucle d'erreurs ?)"; break }
    if ($ReadyPort -and -not $ready -and ((Get-Date) - $t0).TotalSeconds -gt $ReadyTimeoutSec) { $st.status = 'not_ready'; $st.note = "port $ReadyPort non ouvert après $ReadyTimeoutSec s"; break }
    Start-Sleep -Milliseconds 500
    $p.Refresh()
  }
  $exit = $null
  if ($st.status -in 'timeout', 'log_overflow', 'not_ready') { Stop-Tree $p.Id; $exit = if ($st.status -eq 'timeout') { 124 } else { 125 } }
  else { $p.WaitForExit(); $exit = $p.ExitCode; $st.status = if ($exit -eq 0) { 'done' } else { 'failed' } }
  if ($Daemonizes -and $exit -eq 0) {
    # le service a pris le relais : on attend le port, on enregistre son PID, on ne tue rien
    $w = Get-Date
    while (-not (PortOwner $ReadyPort) -and ((Get-Date) - $w).TotalSeconds -lt $ReadyTimeoutSec) { Start-Sleep -Milliseconds 500 }
    $owner = PortOwner $ReadyPort
    if ($owner) { $sp2 = Get-Process -Id $owner; $st.status = 'service'; $st.pid = $owner; $st.pid_started = $sp2.StartTime.ToUniversalTime().ToString('o'); $st.note = "service sur le port $ReadyPort (pid $owner)" }
    else { $st.status = 'not_ready'; $st.note = "port $ReadyPort non ouvert après $ReadyTimeoutSec s" }
  }
  elseif ($st.status -ne 'ready') {
    # DÉSACTIVÉ le 31/10/2026 : l'arrêt des « restes » par parenté de PID a tué des processus sans lien (réutilisation des PID).
    # On se contente de signaler les descendants encore vivants, sans rien arrêter.
    $left = @($seen | Where-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue } | Select-Object -First 5)
    if ($left.Count) { $st.note += ' descendants encore vivants (non arrêtés) : ' + ($left -join ',') }
  }
  $st.exit_code = $exit; $st.ended = (Get-Date).ToUniversalTime().ToString('o'); Write-State $Name $st
  if ($Verb -eq 'run') {
    "[$($st.status)] $Name — code de sortie $exit — $([int]((Get-Date) - $t0).TotalSeconds) s — journaux : $outLog / $errLog $($st.note)"
    "--- fin de stdout"; Tail $outLog $TailLines; "--- fin de stderr"; Tail $errLog $TailLines
    exit $exit
  }
}

# ───────────────────────── start ─────────────────────────
function Verb-Start {
  if (-not $Name -or -not $Exe) { throw '-Name et -Exe sont obligatoires' }
  $old = Read-State $Name
  if ($old -and $old.status -in 'running', 'ready', 'service' -and (Alive $old.pid $old.pid_started)) { throw "la tâche '$Name' tourne déjà (pid $($old.pid)) ; la voir avec status, l'arrêter avec stop" }
  $a = @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', ('"' + $PSCommandPath + '"'), 'run', '-Name', $Name, '-Exe', ('"' + $Exe + '"'), '-WorkDir', ('"' + $WorkDir + '"'),
    '-TimeoutSec', $TimeoutSec, '-MaxLogMB', $MaxLogMB, '-ReadyPort', $ReadyPort, '-ReadyTimeoutSec', $ReadyTimeoutSec)
  if ($Persistent) { $a += '-Persistent' }
  if ($Daemonizes) { $a += '-Daemonizes' }
  if ($ArgList.Count) {
    $af = Join-Path $State "$Name.args.json"
    [System.IO.File]::WriteAllText($af, (ConvertTo-Json @($ArgList | ForEach-Object { [string]$_ }) -Compress), (New-Object System.Text.UTF8Encoding($false)))
    $a += @('-ArgFile', ('"' + $af + '"'))
  }
  $sup = Start-Process powershell.exe -ArgumentList $a -WindowStyle Hidden -PassThru
  # attend que l'état « running » soit écrit, puis, si un port est demandé, la fin de la phase de démarrage
  $w = Get-Date
  while (-not (Read-State $Name) -or (Read-State $Name).supervisor -ne $sup.Id) { Start-Sleep -Milliseconds 200; if (((Get-Date) - $w).TotalSeconds -gt 60) { throw 'le superviseur ne répond pas' } }
  if ($ReadyPort) {
    # serveur : on rend la main dès que le port est ouvert ; service lancé par une commande qui se termine
    # (pg_ctl) : on attend la fin de cette commande, le port s'ouvre avant que la base accepte les connexions
    $waiting = if ($Daemonizes) { 'running', 'ready' } else { , 'running' }
    while (((Read-State $Name).status -in $waiting) -and ((Get-Date) - $w).TotalSeconds -lt ($ReadyTimeoutSec + 10)) { Start-Sleep -Milliseconds 300 }
  }
  Verb-Status
}

# ───────────────────────── status / stop ─────────────────────────
function Effective($s) {
  if ($s.status -in 'running', 'ready') {
    if (-not (Alive $s.supervisor $s.supervisor_started)) { return 'perdue (superviseur absent)' }
    if ($s.deadline -and (Get-Date).ToUniversalTime() -gt ([datetime]$s.deadline).ToUniversalTime()) { return 'EXPIRÉE (délai dépassé)' }
  }
  if ($s.status -eq 'service' -and -not (Alive $s.pid $s.pid_started)) { return 'service arrêté' }
  return $s.status
}
function Verb-Status {
  $rows = Get-ChildItem $State -Filter *.json -ErrorAction SilentlyContinue | Where-Object { $_.Name -notlike '*.args.json' } | ForEach-Object {
    $s = Get-Content $_.FullName -Raw | ConvertFrom-Json
    if ($Name -and $s.name -ne $Name) { return }
    [pscustomobject]@{ Nom = $s.name; Type = $s.kind; État = Effective $s; PID = $s.pid; Code = $s.exit_code; Minutes = [int](($(if ($s.ended) { ([datetime]$s.ended).ToUniversalTime() } else { (Get-Date).ToUniversalTime() }) - ([datetime]$s.started).ToUniversalTime()).TotalMinutes) ; JournauxMo = LogMB $s.name; Note = $s.note }
  }
  if ($rows) { $rows | Format-Table -AutoSize | Out-String -Width 220 } else { 'aucune tâche enregistrée' }
}
function Verb-Stop {
  if (-not $Name) { throw '-Name obligatoire' }
  $s = Read-State $Name; if (-not $s) { throw "tâche '$Name' inconnue" }
  foreach ($pair in @(@($s.supervisor, $s.supervisor_started), @($s.pid, $s.pid_started))) {
    if ($pair[0] -and (Alive $pair[0] $pair[1])) { Stop-Tree $pair[0]; "arrêté : pid $($pair[0])" }
    elseif ($pair[0]) { "pid $($pair[0]) déjà absent ou réutilisé : non touché" }
  }
  $s.status = 'stopped'; $s.ended = (Get-Date).ToUniversalTime().ToString('o'); Write-State $Name $s
}

# ───────────────────────── audit / clean ─────────────────────────
function Spike-Procs {
  $names = 'python|pg_ctl|postgres|spike-.*|cargo|rustc|node|typst|chrome|powershell|cmd|bash'
  Get-CimInstance Win32_Process | Where-Object { $_.Name -match "^($names)\.exe$" -and $_.CommandLine -notmatch '--type=' -and $_.ProcessId -ne $PID }
}
function Verb-Audit {
  $a0 = @{}; foreach ($x in Spike-Procs) { $p = Get-Process -Id $x.ProcessId -ErrorAction SilentlyContinue; if ($p) { $a0[$x.ProcessId] = $p.TotalProcessorTime.TotalSeconds } }
  Start-Sleep 3
  $now = Get-Date
  # le postmaster des prototypes se reconnaît à son dossier de données ; ses fils (--forkchild) à leur parent
  $spikePg = @(Get-CimInstance Win32_Process -Filter "Name='postgres.exe'" | Where-Object { $_.CommandLine -match 'immo-spikes-pg' } | ForEach-Object { $_.ProcessId })
  '== Processus (CPU mesuré sur 3 s)'
  $rows = foreach ($x in Spike-Procs) {
    $p = Get-Process -Id $x.ProcessId -ErrorAction SilentlyContinue; if (-not $p -or -not $a0.ContainsKey($x.ProcessId)) { continue }
    $started = $p.StartTime; if (-not $started) { continue }   # processus protégé (service Windows) : pas d'heure de démarrage lisible
    $cpu = [math]::Round((($p.TotalProcessorTime.TotalSeconds - $a0[$x.ProcessId]) / 3) * 100)
    $conn = (Get-NetTCPConnection -OwningProcess $x.ProcessId -ErrorAction SilentlyContinue | Measure-Object).Count
    $kids = (Get-CimInstance Win32_Process -Filter "ParentProcessId=$($x.ProcessId)" | Measure-Object).Count
    $flag = ''
    if ($x.Name -eq 'python.exe' -and ($x.CommandLine -match 'python(\.exe)?"?\s+-\s*$' -or $x.CommandLine -match 'python(\.exe)?"?\s*$')) { $flag += ' REPL-SANS-SCRIPT' }
    if ($cpu -ge 90 -and $p.TotalProcessorTime.TotalMinutes -gt 10 -and $x.Name -notmatch 'cargo|rustc') { $flag += ' CHAUFFE' }
    if ($x.Name -eq 'postgres.exe' -and ($x.CommandLine -match 'immo-spikes-pg' -or $spikePg -contains $x.ParentProcessId)) { $flag += ' pg-spikes(5433)' }
    elseif ($x.Name -eq 'postgres.exe') { $flag += ' PG-HORS-SPIKES(ne-pas-toucher)' }
    [pscustomobject]@{ PID = $x.ProcessId; PPID = $x.ParentProcessId; Nom = $x.Name; Min = [int](($now - $started).TotalMinutes); 'CPU%' = $cpu; CPUtotal_s = [int]$p.TotalProcessorTime.TotalSeconds; Enfants = $kids; Conn = $conn; Drapeaux = $flag.Trim() }
  }
  $rows | Where-Object { $_.Nom -notmatch '^(powershell|cmd|bash)\.exe$' -or $_.Drapeaux } | Sort-Object PID | Format-Table -AutoSize | Out-String -Width 220
  '== Ports de test en écoute'
  Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue | Where-Object { $_.LocalPort -in 3000, 3001, 5433 } | ForEach-Object { '{0} -> {1} (pid {2})' -f $_.LocalPort, (Get-Process -Id $_.OwningProcess).ProcessName, $_.OwningProcess }
  '== Connexions vers les bases (Neon distant : port 5432 hors 127.0.0.1 ; local : 5433)'
  Get-NetTCPConnection -State Established -ErrorAction SilentlyContinue | Where-Object { ($_.RemotePort -eq 5432 -and $_.RemoteAddress -notmatch '^(127\.|::1)') -or $_.RemotePort -eq 5433 } |
    Group-Object OwningProcess, RemotePort | ForEach-Object { '{0} (pid {1}) : {2} connexion(s) vers le port {3}' -f (Get-Process -Id $_.Group[0].OwningProcess).ProcessName, $_.Group[0].OwningProcess, $_.Count, $_.Group[0].RemotePort }
  '== Tâches enregistrées'
  Verb-Status
  '== Fichiers de sortie démesurés (> 500 Mo) : dossier de l''outil et sorties de tâches de Claude Code'
  $dirs = @($State, "$env:LOCALAPPDATA\Temp\claude")
  foreach ($d in $dirs) { Get-ChildItem $d -Recurse -File -ErrorAction SilentlyContinue | Where-Object { $_.Length -gt 500MB } | ForEach-Object { '{0:N2} Go  {1}' -f ($_.Length / 1GB), $_.FullName } }
  '== Disque C : {0:N1} Go libres' -f ((Get-PSDrive C).Free / 1GB)
}
function Verb-Clean {
  # Ne vise que : (1) tâches enregistrées expirées ou perdues dont un processus tourne encore ;
  # (2) python.exe lancé sans script (REPL) sans connexion ni enfant. Jamais postgres, chrome, node, spike-* sans tâche enregistrée.
  $plan = @()
  Get-ChildItem $State -Filter *.json -ErrorAction SilentlyContinue | Where-Object { $_.Name -notlike '*.args.json' } | ForEach-Object {
    $s = Get-Content $_.FullName -Raw | ConvertFrom-Json; $e = Effective $s
    if ($s.kind -eq 'task' -and $e -match 'EXPIRÉE|perdue' -and $s.pid -and (Alive $s.pid $s.pid_started)) { $plan += [pscustomobject]@{ Raison = "tâche $($s.name) : $e"; PID = $s.pid } }
  }
  foreach ($x in Get-CimInstance Win32_Process -Filter "Name='python.exe'") {
    if ($x.CommandLine -match 'python(\.exe)?"?\s+-\s*$|python(\.exe)?"?\s*$' -and -not (Get-NetTCPConnection -OwningProcess $x.ProcessId -ErrorAction SilentlyContinue) -and -not (Get-CimInstance Win32_Process -Filter "ParentProcessId=$($x.ProcessId)")) {
      $plan += [pscustomobject]@{ Raison = 'python sans script (REPL)'; PID = $x.ProcessId }
    }
  }
  if (-not $plan) { 'rien à nettoyer'; return }
  $plan | Format-Table -AutoSize | Out-String
  if (-not $Apply) { 'simulation : ajouter -Apply pour arrêter ces processus (arbre complet)'; return }
  foreach ($x in $plan) { Stop-Tree $x.PID; "arrêté : pid $($x.PID) — $($x.Raison)" }
}

switch ($Verb) { 'run' { Verb-Run } 'start' { Verb-Start } 'status' { Verb-Status } 'stop' { Verb-Stop } 'audit' { Verb-Audit } 'clean' { Verb-Clean } }
