# Eclipse CLI Daemon

## Vue d'ensemble

Le **daemon Eclipse CLI** maintient une connexion MCP persistante vers eclipsectl-mcp, éliminant la latence de reconnexion à chaque commande.

## Performance

| Scénario | Sans daemon | Avec daemon |
|----------|-------------|-------------|
| Première commande | ~4-5s | ~4-5s (init + start daemon) |
| Commandes suivantes | ~4-5s (reconnexion) | **<0.1s** (socket IPC) |

## Architecture

```
┌─────────────────────────┐
│  eclipse project list   │  CLI client
└────────────┬────────────┘
             │ Unix socket (~0.05s)
             ▼
┌─────────────────────────┐
│   Eclipse Daemon        │  Background process
│   - Connexion MCP ✓     │  (auto-started)
│   - eclipsectl-mcp session    │
└────────────┬────────────┘
             │ HTTP MCP (~0.5s)
             ▼
┌─────────────────────────┐
│   eclipsectl-mcp Plugin       │  Windows/Eclipse
│   - Port 8124           │
└─────────────────────────┘
```

## Utilisation

### Transparent (recommandé)

Le daemon se lance **automatiquement** au premier appel CLI :

```bash
eclipse project list        # Démarre daemon (~4s)
eclipse project layout foo  # Réutilise daemon (<0.1s)
eclipse test all           # Réutilise daemon (<0.1s)
```

### Gestion manuelle

```bash
# Vérifier statut
eclipse daemon status

# Arrêter daemon
eclipse daemon stop

# Redémarrer daemon
eclipse daemon restart
```

## Comportement

### Auto-start
- La première commande lance automatiquement le daemon si absent
- Message affiché : `🚀 Starting Eclipse daemon...`

### Auto-shutdown
- Le daemon s'arrête automatiquement après **5 minutes d'inactivité**
- Économise les ressources quand non utilisé

### Fichiers
- **Socket** : `~/.config/eclipsectl-cli/daemon.sock`
- **PID** : `~/.config/eclipsectl-cli/daemon.pid`
- **Logs** : `~/.config/eclipsectl-cli/daemon.log`

## Avantages

### 1. Performance
- 40x plus rapide après le premier appel
- Idéal pour usage interactif ou scripts

### 2. Simplicité
- Aucune configuration nécessaire
- Compatibilité 100% avec commandes existantes

### 3. Robustesse
- Redémarre automatiquement si crash
- Gestion propre des signaux (SIGTERM, SIGINT)
- Timeout d'inactivité pour éviter fuites

## Limitations

### Connexion unique
Le daemon maintient **une seule connexion MCP** partagée par toutes les commandes CLI.

Si plusieurs commandes s'exécutent en parallèle, elles sont **sérialisées** (une à la fois).

### Instance unique (limitation actuelle)
**Actuellement** : Un seul daemon par utilisateur.

Si vous avez plusieurs instances Eclipse sur ports différents (ex: 8124, 8125), elles partageront le même daemon. Seul le premier port configuré sera utilisé.

**Solution future** : Support multi-daemon basé sur le port (voir `TODO.md`).

Pour l'instant, si vous avez besoin de plusieurs Eclipse simultanées :
- Arrêtez le daemon : `eclipse daemon stop`
- Changez `ECLIPSE_URL` / config
- Relancez : `eclipse project list`

## Dépannage

### Daemon ne démarre pas

```bash
# Vérifier logs
tail -f ~/.config/eclipsectl-cli/daemon.log

# Vérifier config
cat ~/.config/eclipsectl-cli/config.json

# Tester manuellement
python -m eclipsectl_cli.daemon
```

### Daemon bloqué

```bash
# Forcer arrêt
pkill -f "eclipsectl_cli.daemon"

# Nettoyer fichiers
rm ~/.config/eclipsectl-cli/daemon.sock
rm ~/.config/eclipsectl-cli/daemon.pid

# Relancer
eclipse project list
```

### Commandes toujours lentes

```bash
# Vérifier que daemon tourne
eclipse daemon status

# Si non, vérifier pourquoi il crash
tail -20 ~/.config/eclipsectl-cli/daemon.log
```

## Développement

### Architecture interne

**daemon.py** : Serveur Unix socket qui :
- Initialise `EclipseClient` une seule fois
- Garde la connexion MCP ouverte
- Écoute sur socket Unix
- Route les commandes vers les méthodes client

**bin/eclipse** : Façade CLI Bash qui :
- Détecte si le daemon tourne (socket et PID)
- Lance `python -m eclipsectl_cli.daemon` s'il est absent
- Construit les requêtes JSON
- Les envoie directement sur le socket Unix avec `nc`

Cette façade légère évite de démarrer Python et de charger une bibliothèque CLI à
chaque commande. Python ne reste nécessaire que pour le daemon persistant et le
serveur MCP `eclipsectl-cli`.

### Protocole socket

Format JSON simple :

```json
// Request
{
  "command": "list_projects",
  "args": {}
}

// Response
{
  "success": true,
  "result": "Project1\nProject2\n..."
}
```

### Tests

```bash
# Test de performance
bash scripts/test_daemon.sh

# Test manuel
eclipse project list          # Démarre daemon
eclipse daemon status         # Vérifie statut
eclipse project list          # Devrait être rapide
eclipse daemon stop           # Arrête daemon
```

## Migration depuis version sans daemon

**Aucune migration nécessaire !**

Toutes vos commandes `eclipse` existantes fonctionnent exactement pareil, mais plus vite.

Le daemon est 100% rétrocompatible.
