# Outils des prototypes : gestion des tâches et des processus

Écrit le 31/10/2026 après un incident : un `python -` lancé sans script a ouvert le REPL interactif
de Python 3.13 dans un contexte sans console. Il échouait en boucle, avait consommé 4 587 s de
processeur en 82 minutes et avait rempli un fichier de sortie de 3,46 Go (1 Mo/s) avant d'être arrêté.
Aucune donnée Neon n'a été touchée. Six « tâches actives » vues dans l'interface avaient des causes différentes
(un test fautif, des serveurs voulus, des mesures réseau lentes, un shell resté accroché à un service).

## Règles

1. **Aucun interpréteur sans script** : jamais `python -`, `python` nu, `node -`, `node` nu. Toujours
   `python -I script.py`, `python -I -c "..."` ou `node script.mjs`. Test factice : utiliser `cmd /c exit 0`.
2. **Aucun service lancé dans une tâche** : `pg_ctl start -w`, un serveur Axum, un navigateur maintenu ouvert
   gardent les tubes de sortie ouverts et la tâche ne se termine jamais. Utiliser `task.ps1 start -Persistent`
   (serveur) ou `-Daemonizes` (commande qui lance un service puis se termine, comme `pg_ctl start`).
3. **Tout au premier plan avec délai** : `task.ps1 run -TimeoutSec N`. Au-delà, l'arbre de processus est arrêté
   (code 124). Un journal au-delà de `-MaxLogMB` (200 par défaut) arrête aussi la tâche (code 125).
4. **Tout ce qui dure plus de 2 minutes passe en arrière-plan** : `task.ps1 start`. Un superviseur détaché
   applique le délai, enregistre le code de sortie, arrête les restes et écrit l'état.
5. **Journaux dans des fichiers, jamais dans des tubes** : `spikes/out/tasks/<nom>.out.log` et `.err.log`
   (ignorés par Git). Les mesures longues écrivent leur progression ligne par ligne (pas de tampon).
6. **Un serveur de test par base** : le client de charge reçoit toujours l'URL explicitement
   (`load neon <scénario> http://127.0.0.1:3001`) ; le port par défaut 3000 est réservé au serveur local.
7. **Jamais** toucher au PostgreSQL de Mimis (port 5432) ni arrêter un processus dont on n'a pas lu le rôle :
   `task.ps1 audit` d'abord, puis `task.ps1 clean` (simulation), puis `clean -Apply`.

8. **Un service attaché à une console ne survit pas à la mort de cette console** : le 31/10/2026, l'arrêt du
   `conhost.exe` (7084) d'une enveloppe a rendu le PostgreSQL jetable inutilisable, bien que son postmaster fût vivant
   (ses nouveaux processus clients échouaient avec l'exception `0xC0000142`, « échec d'initialisation de DLL »). Ce lien
   n'apparaît pas dans l'arbre parent-enfant ni dans les objets Job. Avant d'arrêter une enveloppe ou un conhost, ne pas
   se fier à l'arbre de processus : redémarrer le service par `task.ps1 start -Daemonizes` qui lui donne sa propre console.

## Commandes

```powershell
.\task.ps1 audit                                   # lecture seule : processus, ports, connexions, tâches, fichiers démesurés
.\task.ps1 status [-Name n]                        # état des tâches enregistrées
.\task.ps1 run   -Name build-b -TimeoutSec 900 -Exe cargo -ArgList build,--release -WorkDir ..\isolation-pool
.\task.ps1 start -Name neon-c  -TimeoutSec 7200 -Exe .\mon-script.cmd
.\task.ps1 start -Name srv-c-neon -Persistent -ReadyPort 3001 -Exe <exe> -ArgList serve,neon,3001
.\task.ps1 start -Name pg-spikes -Daemonizes -ReadyPort 5433 -Exe "C:\Program Files\PostgreSQL\18\bin\pg_ctl.exe" -ArgList -D,<dossier>,-l,<journal>,start
.\task.ps1 stop  -Name srv-c-neon                  # vérifie le PID et l'heure de démarrage avant d'arrêter l'arbre
.\task.ps1 clean [-Apply]                          # ne vise que : tâches expirées ou perdues, python sans script
```

États : `running`, `ready` (port ouvert), `service` (service lancé par une commande qui s'est terminée), `done`,
`failed`, `timeout` (124), `log_overflow` (125), `not_ready`, `stopped`. `status` signale en plus `EXPIRÉE`,
`perdue (superviseur absent)` et `service arrêté`.

## Limites connues

- Un processus lancé hors de `task.ps1` n'est pas suivi : `audit` le voit, `clean` ne l'arrête que s'il est un
  `python` sans script, sans enfant ni connexion.
- `Stop-Tree` s'appuie sur `taskkill /T` : un processus qui s'est détaché de son parent n'est pas atteint.
- L'échantillon de charge de `audit` dure 3 s.

## Incident du 31/10/2026 (15:20) : arrêt de processus sans lien

La version initiale de `task.ps1` arrêtait, à la fin d'une tâche, tout « descendant encore vivant » trouvé par
parenté de PID (`ParentProcessId`). Pendant une compilation de 158 s, la réutilisation des numéros de processus
a rattaché à tort plus de 200 processus sans lien (services, PostgreSQL de Mimis, mesure Neon en cours). La commande
`Stop-Process -Force` a été appliquée à tous : 58 ont disparu (les 155 autres, services protégés, ont résisté faute de
droits), dont la mesure de surcoût Neon et son superviseur. Le PostgreSQL de Mimis (5432) a survécu. Une application de
l'utilisateur a pu être touchée ; on ne peut pas le démontrer a posteriori.

Corrections : (1) l'arrêt automatique des « restes » est supprimé : on signale seulement les descendants vivants ;
(2) `Stop-Tree` ne s'appuie plus sur `taskkill /T` : il ne retient un enfant que s'il a été **créé après son parent**,
puis arrête les enfants avant la racine ; (3) jamais d'arrêt sans que la tâche enregistrée ait elle-même lancé le
processus. **Règle** : une parenté de PID seule ne prouve rien sous Windows ; toujours croiser avec les dates de création.

## Défaut des arguments en arrière-plan (31/10/2026, 16:20)

`task.ps1 start` transmettait les arguments au superviseur sous forme de JSON dans la ligne de commande. En Windows
PowerShell 5.1, `ConvertFrom-Json` rend un tableau comme **un seul objet** : le programme recevait un argument unique
(ici « db C:\...\reference.xlsx neon » : message d'usage, code 2), (pourquoi des arguments sans espace passaient quand même n'a pas été élucidé). Correction :
les arguments passent par un fichier `spikes/out/tasks/<nom>.args.json` (option `-ArgFile`) et sont énumérés un à un.
Testé avec un chemin contenant des espaces. **Règle** : tester `start` avec un argument à espaces avant tout lancement long.

