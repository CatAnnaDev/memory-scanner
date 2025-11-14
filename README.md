# 🔍 Memory Pattern Scanner

Une bibliothèque Rust multi-plateforme pour scanner la mémoire des processus et trouver des patterns de bytes spécifiques avec support des wildcards.

## ✨ Fonctionnalités

- 🎯 **Recherche de patterns** : Trouvez des séquences de bytes spécifiques dans la mémoire d'un processus
- 🃏 **Support des wildcards** : Utilisez `xx` ou `?` pour ignorer certains bytes
- 🖥️ **Multi-plateforme** : Fonctionne sur Windows, Linux et macOS
- ⚡ **Performant** : Scan optimisé avec limite de résultats configurable
- 🛡️ **Sécurisé** : Gère proprement les régions mémoire protégées

## 📦 Installation

```bash
git clone https://github.com/votre-username/memory-scanner.git
cd memory-scanner
cargo build --release
```

### Dépendances

**Windows uniquement :**
```toml
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["winnt", "memoryapi", "processthreadsapi", "handleapi"] }
```

**Unix (Linux/macOS) :**
```toml
[target.'cfg(unix)'.dependencies]
libc = "0.2"
```

## 🚀 Utilisation

### Ligne de commande

```bash
# Syntaxe de base
./target/release/mem_finder <PID> <pattern>

# Exemples
./target/release/mem_finder 1234 "48 8B xx 48 89 xx"
./target/release/mem_finder 5678 "22 55 77 xx 60"
./target/release/mem_finder 9012 "? 8D 4C ? 08"
```

### En tant que bibliothèque

```rust
use memory_scanner::{MemoryScanner, Pattern};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Créer un pattern avec wildcards
    let pattern = Pattern::from_string("48 8B xx 48 89 xx")?;
    
    // S'attacher au processus
    let scanner = MemoryScanner::attach(1234)?;
    
    // Scanner la mémoire (max 100 résultats)
    let results = scanner.scan(&pattern, 100);
    
    // Afficher les résultats
    for result in results {
        println!("Trouvé à 0x{:X}: {:02X?}", 
                 result.address, 
                 result.matched_bytes);
    }
    
    Ok(())
}
```

### Format des patterns

Les patterns utilisent la notation hexadécimale séparée par des espaces :

- **Bytes exacts** : `48 8B C3` (cherche exactement ces 3 bytes)
- **Wildcards** : `48 xx C3` ou `48 ? C3` (ignore le byte du milieu)
- **Mixte** : `48 8B xx xx 89 xx 24` (combinaison de bytes exacts et wildcards)

## 🖥️ Configuration par plateforme

### Windows

✅ **Fonctionne directement** avec les droits administrateur

```bash
# PowerShell en tant qu'administrateur
.\target\release\mem_finder.exe 1234 "48 8B xx"
```

**Permissions requises :** Administrateur pour accéder à certains processus système

---

### Linux

✅ **Fonctionne avec sudo** ou configuration ptrace

```bash
# Méthode 1 : Utiliser sudo
sudo ./target/release/mem_finder 1234 "48 8B xx"

# Méthode 2 : Autoriser ptrace (permanent)
echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope
./target/release/mem_finder 1234 "48 8B xx"
```

**Permissions requises :** 
- `sudo` ou capacité `CAP_SYS_PTRACE`
- Ou désactiver `ptrace_scope` (moins sécurisé)

---

### macOS

⚠️ **Configuration spéciale requise** à cause de System Integrity Protection (SIP)

#### Option 1 : Désactiver partiellement SIP (RECOMMANDÉ)

```bash
# 1. Redémarrer en Recovery Mode (Cmd+R au boot)
# 2. Ouvrir Terminal
# 3. Exécuter :
csrutil enable --without debug

# 4. Redémarrer
reboot

# 5. Vérifier
csrutil status
# Devrait afficher: "enabled (Apple Internal: enabled; Kext Signing: enabled; 
#                    Filesystem Protections: enabled; Debugging Restrictions: disabled)"
```

Maintenant vous pouvez utiliser le scanner normalement :

```bash
./target/release/mem_finder 1234 "48 8B xx"
```

#### Option 2 : Signer avec entitlements

Si vous ne voulez pas modifier SIP, vous pouvez signer l'application :

**1. Créer `entitlements.plist` :**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.debugger</key>
    <true/>
    <key>com.apple.security.get-task-allow</key>
    <true/>
</dict>
</plist>
```

**2. Signer le binaire :**

```bash
# Lister vos certificats
security find-identity -v -p codesigning

# Signer avec votre certificat Apple Development
codesign --force --sign "Apple Development: votre.email@example.com (TEAMID)" \
  --entitlements entitlements.plist \
  --options runtime \
  target/release/mem_finder

# Vérifier
codesign -d --entitlements - target/release/mem_finder
```

**Note :** Les entitlements seuls ne suffisent généralement pas sur macOS récent. La désactivation partielle de SIP (Option 1) reste nécessaire dans la plupart des cas.

#### Option 3 : Utiliser sudo

```bash
sudo ./target/release/mem_finder 1234 "48 8B xx"
```

**Note :** Sur macOS récent, même sudo peut échouer avec SIP activé.

---

## 🏗️ Architecture

```
src/
├── lib.rs          # Code commun (Pattern, ScanResult, interface publique)
├── windows.rs      # Implémentation Windows (VirtualQueryEx, ReadProcessMemory)
├── linux.rs        # Implémentation Linux (/proc/pid/maps, /proc/pid/mem)
├── macos.rs        # Implémentation macOS (proc_pidinfo, Mach VM)
└── main.rs         # Exemple CLI
```

### Fonctionnement interne

1. **Énumération des régions mémoire** : Liste toutes les régions accessibles du processus
2. **Filtrage** : Ignore les régions non lisibles ou protégées
3. **Lecture** : Lit le contenu de chaque région en mémoire
4. **Pattern matching** : Recherche le pattern dans chaque buffer
5. **Résultats** : Retourne les adresses et bytes matchés

## 🧪 Tests

```bash
# Lancer les tests unitaires
cargo test

# Test avec un processus réel
# Terminal 1
sleep 1000 &
echo $!  # Noter le PID

# Terminal 2
cargo run --release -- <PID> "48 xx xx 89"
```

## 🔒 Sécurité et éthique

⚠️ **Avertissement** : Cet outil est conçu pour :
- Le développement et le debugging
- L'analyse de sécurité légale
- La recherche en reverse engineering

**N'utilisez jamais cet outil pour :**
- Tricher dans les jeux en ligne
- Contourner des protections sans autorisation
- Violer les conditions d'utilisation de logiciels
- Toute activité illégale

L'utilisateur est responsable de l'usage qu'il fait de cet outil.

## 🐛 Dépannage

### Windows : "Accès refusé"
- Lancez en tant qu'administrateur
- Vérifiez que l'antivirus ne bloque pas le programme

### Linux : "Operation not permitted"
```bash
# Vérifier ptrace_scope
cat /proc/sys/kernel/yama/ptrace_scope
# Si = 1, le changer temporairement
echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope
```

### macOS : "task_for_pid failed"
- SIP bloque l'accès → Utilisez `csrutil enable --without debug`
- Ou lancez avec sudo
- Vérifiez que le PID existe : `ps -p <PID>`

### Aucun résultat trouvé
- Le processus peut utiliser ASLR (Address Space Layout Randomization)
- Le pattern peut être incorrect
- La région mémoire peut être protégée ou swappée
- Essayez avec plus de wildcards

## 📝 TODO

- [ ] Support de patterns avec masques binaires
- [ ] Scan récursif avec suivi de pointeurs
- [ ] Export des résultats en JSON/CSV
- [ ] Interface graphique (GUI)
- [ ] Support du scan différentiel (avant/après)
- [ ] Optimisation multi-thread

## 🤝 Contribution

Les contributions sont les bienvenues ! N'hésitez pas à :
- Ouvrir des issues pour les bugs
- Proposer des améliorations
- Soumettre des pull requests

## 📄 Licence

MIT License - voir le fichier LICENSE pour plus de détails

## 👤 Auteur

Créé avec ❤️ en Rust

---

**Note importante pour macOS** : Apple renforce continuellement les restrictions de sécurité. Sur les versions récentes (Monterey, Ventura, Sonoma), `task_for_pid` est de plus en plus verrouillé même avec les bonnes permissions. La désactivation partielle de SIP reste la solution la plus fiable pour le développement.
