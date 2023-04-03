# IA Blobwar

Projet réalisé à l'ENSIMAG. 
Réalisation de toute la partie intelligence artificielle du jeu et optimisations.
Sans iterative deepening, la stratégie alpha beta est capable d'aller jusqu'à 8 coups d'avances dans un temps raisonnable.

# Pour compiler : 

Sans l'iterative deepening : <br>

<cargo run --release --bin blobwar>

Avec l'iterative deepening : 
  
- <cargo build --release> <br>
- <cargo build --release --bin blobwar_iterative_deepening> <br>
- Utiliser comme stratégie : IterativeDeepening::new(IterativeStrategy::AlphaBeta) <br>
- <cargo run --release --bin blobwar> <br>
                             
/!\ L'iterative deepening ne marche pas sur MacOs

Vous pouvez changer de stratégie dans le src/main.rs
