# gns3-mcp -- Ingénierie réseau pilotée par l'IA

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-stdio-green.svg)](https://modelcontextprotocol.io)
[![Tools](https://img.shields.io/badge/Tools-25-blueviolet.svg)](#outils-disponibles-25)
[![Docker](https://img.shields.io/badge/Docker-~19%20MB-informational.svg)](#démarrage-rapide)

**gns3-mcp** est un serveur MCP haute performance écrit en **Rust** qui expose l'**API REST GNS3 v2** à Claude, transformant votre assistant IA en un véritable ingénieur réseau -- capable de concevoir, déployer et administrer des topologies complexes via une simple conversation.

---

## Pourquoi gns3-mcp ?

Fini les clics répétitifs dans l'interface graphique. Avec `gns3-mcp`, Claude peut :

- **Concevoir** des topologies entières à partir d'une description textuelle.
- **Déployer** et câbler des nœuds (Cisco, Arista, Docker, VPCS...) en quelques secondes.
- **Inspecter** et documenter l'état en direct de votre laboratoire.
- **Piloter** le cycle de vie complet des équipements -- démarrage, arrêt, reconfiguration -- par la voix ou le texte.

Vous et Claude travaillez sur le même projet GNS3 simultanément : vous ajustez dans l'interface graphique pendant que Claude déploie et câble depuis la conversation. Temps réel, bidirectionnel.

---

## Points forts

- **Ultra-léger** : image Docker de ~19 Mo (`distroless/static`, binaire musl statique).
- **25 outils MCP** : CRUD complet pour projets, nœuds, liens, templates, serveurs de calcul, plus des opérations en lot et composites.
- **Résilient** : retry automatique avec backoff exponentiel sur les erreurs 5xx (3 tentatives, 100/200/400 ms), circuit breaker pour les pannes serveur.
- **Circuit breaker** : circuit breaker async trois etats (Closed/Open/Half-Open) qui empeche les tempetes de requetes quand GNS3 est indisponible.
- **Sécurisé nativement** : zéro dépendance OpenSSL/glibc (rustls), validation des UUIDs sur chaque entrée, credentials jamais loggés, conteneur non-root.
- **Architecture propre** : workspace Rust en 3 crates avec inversion de dépendance stricte et injection par trait.
- **Zéro friction** : transport stdio -- intégration native Claude Desktop et Claude Code, aucune infrastructure supplémentaire.

---

## Architecture

```
gns3-mcp/
+-- Cargo.toml              # racine du workspace + deps partagées
+-- crates/
|   +-- core/               # types, traits, erreurs (zéro dep réseau)
|   +-- gns3-client/        # client HTTP implémentant Gns3Api
|   +-- mcp-server/         # outils MCP, bootstrap stdio
+-- docs/adr/               # Architecture Decision Records
+-- .github/workflows/      # CI + pipelines Docker
+-- Dockerfile              # multistage : build Alpine -> runtime distroless
```

Le projet applique une règle de dépendance stricte (Inversion de Dépendance) :

```
mcp-server  -->  core          (traits uniquement)
mcp-server  -.-> gns3-client   (injecté dans main.rs uniquement)
gns3-client -->  core          (types + trait Gns3Api)
core        -->  rien          (serde, thiserror, uuid)
```

> [!NOTE]
> `mcp-server` n'a aucune connaissance de HTTP ni de reqwest. Il interagit uniquement via le trait `Gns3Api` défini dans `core`, ce qui rend chaque outil entièrement testable avec un mock -- aucun serveur GNS3 requis.

---

## Démarrage rapide

### Prérequis

- Serveur GNS3 2.2.x+, en cours d'exécution et accessible en HTTP
- Rust 1.75+ **ou** Docker

### Option 1 : Docker (recommandé)

```bash
docker build -t gns3-mcp:latest .

docker run --rm -i \
  -e GNS3_URL=http://localhost:3080 \
  --network host \
  gns3-mcp:latest
```

> **Note** : `--network host` est requis pour que le conteneur atteigne GNS3 sur `localhost`. Sur macOS/Windows Docker Desktop, utilisez `-e GNS3_URL=http://host.docker.internal:3080` a la place.

### Option 2 : Compilation depuis les sources

```bash
cargo build --release
GNS3_URL=http://localhost:3080 ./target/release/gns3-mcp
```

---

## Configuration Claude Desktop / Claude Code

Ajoutez cette section à votre fichier de configuration (`claude_desktop_config.json` ou `.claude/settings.json`) :

```json
{
  "mcpServers": {
    "gns3": {
      "command": "docker",
      "args": [
        "run", "--rm", "-i",
        "-e", "GNS3_URL",
        "-e", "GNS3_USER",
        "-e", "GNS3_PASSWORD",
        "--network", "host",
        "gns3-mcp:latest"
      ],
      "env": {
        "GNS3_URL": "http://localhost:3080"
      }
    }
  }
}
```

Les credentials utilisent le pass-through Docker (`-e VAR` sans `=valeur`). Définissez-les dans votre shell ou un fichier `.env` -- jamais en clair dans la configuration JSON.

### Variables d'environnement

| Variable | Requise | Défaut | Description |
|---|---|---|---|
| `GNS3_URL` | Oui | `http://127.0.0.1:3080` | URL de base du serveur GNS3 |
| `GNS3_USER` | Non | -- | Nom d'utilisateur (Basic auth) |
| `GNS3_PASSWORD` | Non | -- | Mot de passe (Basic auth) |
| `GNS3_TIMEOUT_SECS` | Non | `30` | Timeout des requêtes HTTP en secondes |
| `RUST_LOG` | Non | `info` | Niveau de log (trace, debug, info, warn, error) |

---

## Outils disponibles (25)

### Projets

| Outil | Ce qu'il fait |
|---|---|
| `gns3_get_version` | Vérifie la connectivité et la version du serveur GNS3 |
| `gns3_list_projects` | Liste tous les projets avec leur statut |
| `gns3_create_project` | Crée et ouvre un nouveau projet |
| `gns3_open_project` | Ouvre un projet existant (requis avant toute opération sur les nœuds et liens) |
| `gns3_close_project` | Ferme un projet et libère les ressources |
| `gns3_delete_project` | Supprime définitivement un projet |
| `gns3_get_topology` | Snapshot complet : tous les nœuds + tous les liens en un seul appel |

### Nœuds

| Outil | Ce qu'il fait |
|---|---|
| `gns3_list_templates` | Découvre les appliances disponibles (routeurs, switches, etc.) |
| `gns3_create_node` | Déploie un nœud depuis un template sur le canvas |
| `gns3_list_nodes` | Liste les nœuds avec statut, type et ports console |
| `gns3_start_node` | Démarre un nœud unique |
| `gns3_stop_node` | Arrête un nœud unique |
| `gns3_delete_node` | Supprime un nœud du projet |
| `gns3_update_node` | Met a jour le nom, le compute ou les proprietes d'un noeud |
| `gns3_configure_switch` | Configure le mapping des ports d'un switch (VLANs, trunks) |
| `gns3_start_all_nodes` | Démarre tous les nœuds du projet en une seule opération |
| `gns3_stop_all_nodes` | Arrête tous les nœuds du projet en une seule opération |

### Liens

| Outil | Ce qu'il fait |
|---|---|
| `gns3_create_link` | Câble deux interfaces de nœuds ensemble |
| `gns3_list_links` | Liste toutes les connexions du projet |
| `gns3_delete_link` | Supprime une connexion |

### Infrastructure

| Outil | Ce qu'il fait |
|---|---|
| `gns3_list_computes` | Liste les serveurs de calcul avec utilisation CPU/mémoire |

### Templates

| Outil | Ce qu'il fait |
|---|---|
| `gns3_update_template` | Met a jour les proprietes d'un template (RAM, interfaces, image) |

### Dessins

| Outil | Ce qu'il fait |
|---|---|
| `gns3_add_drawing` | Ajoute un dessin SVG sur le canvas du projet |

### Snapshots & Export

| Outil | Ce qu'il fait |
|---|---|
| `gns3_export_project` | Exporte un projet sous forme d'archive portable |
| `gns3_snapshot_project` | Cree un snapshot nomme pour le rollback |

### Workflow typique

```
1. gns3_get_version        -- vérifier la connectivité
2. gns3_create_project     -- ou gns3_list_projects pour réutiliser un projet existant
3. gns3_open_project       -- déverrouiller les opérations sur nœuds et liens
4. gns3_list_templates     -- découvrir les appliances disponibles
5. gns3_create_node        -- répéter pour chaque équipement
6. gns3_create_link        -- répéter pour chaque câble
7. gns3_start_all_nodes    -- démarrer tout en une fois
8. gns3_get_topology       -- inspecter la topologie en direct
```

---

## Résilience

- **Retry sur 5xx** : backoff exponentiel (100 ms, 200 ms, 400 ms), jusqu'à 3 tentatives. Les erreurs 4xx et les erreurs réseau échouent immédiatement -- aucun temps perdu sur des requêtes incorrectes.
- **Circuit breaker** : arrete automatiquement les appels GNS3 apres 5 echecs consecutifs. Recuperation apres 30 s avec une requete sonde -- protege le serveur pendant les pannes.
- **Timeout configurable** : variable `GNS3_TIMEOUT_SECS`, défaut 30 s.
- **Erreurs actionnables** : chaque erreur retournée à Claude explique ce qui a échoué et suggère l'étape suivante.

---

## Sécurité

| Propriété | Détail |
|---|---|
| **Credentials** | Variables d'environnement uniquement -- jamais loggés, jamais dans les messages d'erreur, jamais visibles par le MCP |
| **Validation des entrées** | Chaque UUID reçu de Claude est validé avant tout appel API |
| **Conteneur** | `distroless/static:nonroot` -- pas de shell, pas de gestionnaire de paquets, utilisateur non privilégié |
| **TLS** | `rustls` pur Rust -- aucune dépendance OpenSSL ni glibc |
| **Binaire** | Lié statiquement en musl -- zéro dépendance dynamique |

---

## Développement

```bash
cargo check --workspace                    # vérification des types
cargo test --workspace                     # 61+ tests unitaires, aucun serveur GNS3 requis
cargo clippy --workspace -- -D warnings    # lint (politique zéro avertissement)
cargo fmt --check                          # vérification du formatage
```

La CI s'exécute automatiquement à chaque push et pull request via GitHub Actions.

---

## Compatibilité

gns3-mcp utilise le protocole standard **JSON-RPC sur stdio** -- le protocole MCP ouvert. Tout client implémentant MCP peut l'utiliser, pas uniquement Claude.

| Client | Statut | Notes |
|---|---|---|
| **Claude Desktop / Code** | Natif | Support stdio intégré |
| **OpenAI / ChatGPT** | Supporté | Support MCP annoncé en mars 2025 |
| **Cursor** | Natif | Client MCP intégré |
| **Windsurf** | Natif | Client MCP intégré |
| **Continue.dev** | Natif | Fait le pont entre MCP et tout backend LLM (Ollama, llama.cpp, etc.) |
| **LM Studio** | Via wrapper | Nécessite une couche client MCP (ex : SDK Python `mcp`) |
| **Agents custom** | Oui | Tout code capable de lancer un subprocess et d'écrire du JSON-RPC sur stdin |

> [!NOTE]
> **Capacité du modèle** : la qualité de l'expérience dépend de la capacité du LLM à comprendre les descriptions d'outils et à séquencer les appels correctement (ex : `open_project` avant `create_node`). Les grands modèles (70B+, Claude, GPT-4) gèrent cela sans difficulté. Les modèles locaux plus petits (7-13B) peuvent rencontrer des difficultés avec l'orchestration multi-étapes.

---

## Décisions d'architecture

La justification des choix de conception est documentée dans `docs/adr/` :

| ADR | Décision |
|---|---|
| 0001 | **stdio plutôt que SSE** -- support natif Claude Desktop, zéro infrastructure |
| 0002 | **Alpine + distroless** -- binaire musl statique, image finale ~19 Mo |
| 0003 | **rustls plutôt qu'OpenSSL** -- aucune friction glibc pour la cross-compilation musl |
| 0004 | **Trait Gns3Api dans core** -- découple le serveur du client, active les tests avec mock |
| 0005 | **Circuit breaker** -- fail-fast sur pannes serveur, auto-recuperation |
| 0006 | **Erreurs non-exhaustives** -- evolution semver-safe des enums d'erreur |

---

## Contribuer

Voir [CONTRIBUTING.md](CONTRIBUTING.md) pour la mise en place du developpement et les conventions.

## Securite

Voir [SECURITY.md](SECURITY.md) pour notre politique de divulgation des vulnerabilites.

---

## Licence

MIT -- voir [LICENSE](LICENSE)
