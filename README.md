# redirect_app

Petite application web autonome (Rust / axum) affichant une page de redirection
propre (Tailwind CSS embarqué, hors-ligne) vers une nouvelle URL, avec un
décompte de 10 secondes (configurable), un bouton pour rediriger immédiatement
et un bouton pour annuler la redirection automatique.

## Build

```powershell
cargo build --release
```

Le binaire `target\release\redirect_app.exe` (~3,5 Mo) contient tout
(HTML + CSS Tailwind embarqués) : aucun autre fichier n'est nécessaire à côté,
sauf le fichier de configuration.

## Déploiement sur un serveur Windows

1. Copier `redirect_app.exe` sur le serveur.
2. Copier `config.env.example` à côté, le renommer en `config.env`, et adapter
   les valeurs (voir ci-dessous).
3. Lancer `redirect_app.exe` (ou le déclarer comme service Windows / tâche
   planifiée / NSSM).
4. Accéder à `http://<serveur>:<port>/` pour vérifier.

Pour changer l'URL de redirection plus tard : modifier `config.env` puis
redémarrer l'exécutable. Aucune recompilation nécessaire.

<details>
<summary>
Démarrage automatique au boot (tâche planifiée, sans compte dédié)
</summary>

`redirect_app.exe` est une application console classique (pas un service Windows
natif). Pour la démarrer automatiquement au démarrage du serveur, même sans
session ouverte et sans créer de compte de service, utilisez une tâche
planifiée exécutée en tant que `SYSTEM` (compte intégré, aucun mot de passe à
gérer) :

```powershell
$exePath = "D:\Redirect_app\redirect_app.exe"
$workDir = "D:\Redirect_app"

$action    = New-ScheduledTaskAction -Execute $exePath -WorkingDirectory $workDir
$trigger   = New-ScheduledTaskTrigger -AtStartup
$principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest
$settings  = New-ScheduledTaskSettingsSet -RestartOnIdle -RestartCount 999 -RestartInterval (New-TimeSpan -Minutes 1) -StartWhenAvailable

Register-ScheduledTask -TaskName "RedirectApp" -Action $action -Trigger $trigger -Principal $principal -Settings $settings
```

Ce script est identique sur tous les serveurs (copier/coller), il ne nécessite
aucun outil tiers ni compte à créer, et redémarre automatiquement le processus
en cas de crash.

Commandes utiles pour la gestion de la tâche :

```powershell
Start-ScheduledTask -TaskName "RedirectApp"      # démarrer manuellement
Stop-ScheduledTask  -TaskName "RedirectApp"      # arrêter (ne tue pas le process, voir note)
Get-ScheduledTaskInfo -TaskName "RedirectApp"     # état / dernier run
Unregister-ScheduledTask -TaskName "RedirectApp" -Confirm:$false  # supprimer
```

> ⚠️ `Stop-ScheduledTask` arrête la tâche planifiée mais ne termine pas
forcément le processus `redirect_app.exe` sous-jacent : utilisez
`Stop-Process -Name redirect_app -Force` si besoin de le tuer explicitement.

</details>

## Variables de configuration (`config.env`)

| Variable                | Description                                      | Défaut                   |
|-------------------------|---------------------------------------------------|---------------------------|
| `REDIRECT_URL`           | URL de destination affichée / utilisée            | `https://example.com`     |
| `REDIRECT_MESSAGE_TITLE` | Titre affiché en haut de la page (guillemets `"`) | `Cette page a déménagé`   |
| `REDIRECT_MESSAGE`       | Message affiché (entourer de guillemets `"`)      | message générique en FR   |
| `REDIRECT_DELAY_SECONDS` | Délai avant redirection automatique (secondes)    | `10`                       |
| `LISTEN_ADDR`            | Adresse/port d'écoute (`IP:port`)                 | `0.0.0.0:8080`             |

⚠️ Toujours entourer `REDIRECT_MESSAGE_TITLE` et `REDIRECT_MESSAGE` de guillemets
doubles `"..."` si la valeur contient des accents ou de la ponctuation, sinon le
parseur peut échouer silencieusement et retomber sur la valeur par défaut.

## Notes

- Le CSS Tailwind (~2,9 Mo, build complet non purgé) est téléchargé une fois
  (`assets/tailwind.min.css`) et embarqué dans le binaire à la compilation —
  aucun accès Internet n'est requis à l'exécution.
- Si le port par défaut est réservé par Windows (plage exclue), changez
  `LISTEN_ADDR` dans `config.env` (ex: `0.0.0.0:9090`).
