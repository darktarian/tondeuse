use std::{
    env, fmt::{self}, str::FromStr, sync::{Arc, Mutex}, thread
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Orientation {
    #[default]
    N,
    E,
    S,
    W,
}

impl Orientation {
    fn gauche(self) -> Self {
        match self {
            Orientation::N => Orientation::W,
            Orientation::W => Orientation::S,
            Orientation::S => Orientation::E,
            Orientation::E => Orientation::N,
        }
    }
    fn droite(self) -> Self {
        match self {
            Orientation::N => Orientation::E,
            Orientation::E => Orientation::S,
            Orientation::S => Orientation::W,
            Orientation::W => Orientation::N,
        }
    }
}

impl FromStr for Orientation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "N" => Ok(Orientation::N),
            "E" => Ok(Orientation::E),
            "S" => Ok(Orientation::S),
            "W" => Ok(Orientation::W),
            _ => Err(format!("Orientation invalide: {}", s)),
        }
    }
}

impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Orientation::N => "N",
            Orientation::E => "E",
            Orientation::S => "S",
            Orientation::W => "W",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Position {
    x: u8,
    y: u8,
    orientation: Orientation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Tondeuse {
    order: u8,
    pos: Position,
    //lsite des actions à faire
    mouvement: Vec<char>,
    //max taille pelouse
    max_x: u8,
    //max taille pelouse
    max_y: u8,
}

impl Tondeuse {
    fn avancer(&mut self, pelouse: Arc<Mutex<Pelouse>>) {
        let mut pelouse = pelouse.lock().unwrap();
        match self.pos.orientation {
            Orientation::N if self.pos.y < self.max_y => {
                let next_y = self.pos.y + 1;
                if pelouse.is_free(self.pos.x, next_y) {
                    pelouse.libere(self.pos.x, self.pos.y);
                    pelouse.occupe(self.pos.x, next_y);
                    self.pos.y += 1
                }
                //self.pos.y += 1
            }
            Orientation::E if self.pos.x < self.max_x => {
                let next_x = self.pos.x + 1;
                if pelouse.is_free(next_x, self.pos.y) {
                    pelouse.libere(self.pos.x, self.pos.y);
                    pelouse.occupe(next_x, self.pos.y);
                    self.pos.x += 1
                }
                //self.pos.x += 1
            }
            Orientation::S if self.pos.y > 0 => {
                let next_y = self.pos.y - 1;
                if pelouse.is_free(self.pos.x, next_y) {
                    pelouse.libere(self.pos.x, self.pos.y);
                    pelouse.occupe(self.pos.x, next_y);
                    self.pos.y -= 1
                }
                //self.pos.y -= 1
            }
            Orientation::W if self.pos.x > 0 => {
                let next_x = self.pos.x - 1;
                if pelouse.is_free(next_x, self.pos.y) {
                    pelouse.libere(self.pos.x, self.pos.y);
                    pelouse.occupe(next_x, self.pos.y);
                    self.pos.x -= 1
                }
                //self.pos.x -= 1
            }
            _ => {}
        }
    }

    fn executer(&mut self, instructions: Vec<char>, pelouse: Arc<Mutex<Pelouse>>) {
        for cmd in instructions {
            match cmd {
                'L' => self.pos.orientation = self.pos.orientation.gauche(),
                'R' => self.pos.orientation = self.pos.orientation.droite(),
                'F' => self.avancer(pelouse.clone()),
                _ => {}
            }
        }
    }

    /// Lance la tondeuse dans un thread
    fn run(self, pelouse: Arc<Mutex<Pelouse>>) -> thread::JoinHandle<Tondeuse> {
        thread::spawn(move || {
            let mut t = self.clone();
            t.executer(self.mouvement, pelouse);
            t
        })
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Pelouse {
    max_x: u8,
    max_y: u8,
    //le vec qui contient toutes les zones occupées
    occupied: Vec<(u8, u8)>,
}

impl Pelouse {
    // pour verifier si la case visée est libre
    fn is_free(&self, x: u8, y: u8) -> bool {
        //x >= 0 && y >= 0 &&
        x <= self.max_x && y <= self.max_y && !self.occupied.contains(&(x, y))
    }
    // pour occuper la case
    fn occupe(&mut self, x: u8, y: u8) {
        self.occupied.push((x, y));
    }
    // pour mettre à jour et retirer la case précédement occupée.
    fn libere(&mut self, x: u8, y: u8) {
        self.occupied.retain(|&(px, py)| px != x || py != y);
    }
}

//Pour créer notre tondeuse à partir de de l'input
fn get_initial_tondeuse(line: &str, mvt: &str, pelouse: &Pelouse, order: u8) -> Tondeuse {
    let mut infos = line.split_whitespace();
    let x = infos
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or_default();
    let y = infos
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or_default();
    let orientation = infos
        .next()
        .and_then(|o| Orientation::from_str(o).ok())
        .unwrap_or_default();

    Tondeuse {
        order,
        mouvement: mvt.to_string().chars().collect(),
        pos: Position { x, y, orientation },
        max_x: pelouse.max_x,
        max_y: pelouse.max_y,
    }
}


//fonction principale de traitement.
//on error default value are applied.
fn executor(content: Vec<&str>) -> Vec<Tondeuse>{
    //cas de parsing de la taille max de le pelouse.
    let mut pelouse = Pelouse::default();
    let mut size_pelouse = content[0].split_whitespace();
    pelouse.max_x = size_pelouse
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or_default();
    pelouse.max_y = size_pelouse
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or_default();

    // gestion des tondeuses
    let mut all_tondeuses: Vec<Tondeuse> = Vec::new();

    //chunk_exact -> s'arrete si moins de 2 lignes restantes.
    for (x, bloc) in content[1..].chunks_exact(2).enumerate() {
        let position = bloc[0];
        let mouvements = bloc[1];
        let tondeuse = get_initial_tondeuse(position, mouvements, &pelouse, x as u8);

        //on initalise les positions de départ sur la pelouse
        pelouse.occupe(tondeuse.pos.x, tondeuse.pos.y);
        all_tondeuses.push(tondeuse);
        
    }

    //on partage la pelouse et donc ses cases occupées.
    let lawn = Arc::new(Mutex::new(pelouse));
    let mut all_th = Vec::new();

    // Lancement en parallèle
    for t in all_tondeuses {
        let t1 = t.run(lawn.clone());
        all_th.push(t1);
    }

    // Récupération des résultats
    let mut final_tondeuses = Vec::new();
    for t in all_th {
        if let Ok(tondeuse) = t.join() {
            final_tondeuses.push(tondeuse);
        }
    }
    final_tondeuses
}

fn main() {
    //pour le passage du fichier de commande en argument.
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
            println!("At least, one arg is needed : command file ! ")
        }
        2 => {
            let file: String = args[1].parse().unwrap_or_default();
            let binding = std::fs::read_to_string(file).expect("something wrong here !");
            let content: Vec<&str> = binding.lines().collect();
            let result = executor(content);
                
            for t in result {
                println!(
                    "Tondeuse n°{} x:{}, y:{}, orientation:{}",
                    t.order, t.pos.x, t.pos.y, t.pos.orientation
                );
            }
        }
        _ => {
            println!("Oups something wrong.")
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn with_original_datas() {
        let input = vec!["5 5", "1 2 N", "LFLFLFLFF", "3 3 E", "FFRFFRFRRF"];
        let reference = vec![
            Tondeuse {
                pos: Position {
                    x: 1,
                    y: 3,
                    orientation: Orientation::N,
                },
                mouvement: vec!['L', 'F', 'L', 'F', 'L', 'F', 'L', 'F', 'F'],
                max_x: 5,
                max_y: 5,
                order: 0,
            },
            Tondeuse {
                pos: Position {
                    x: 5,
                    y: 1,
                    orientation: Orientation::E,
                },
                mouvement: vec!['F', 'F', 'R', 'F', 'F', 'R', 'F', 'R', 'R', 'F'],
                max_x: 5,
                max_y: 5,
                order: 1,
            },
        ];

        let result = executor(input);
        assert_eq!(reference, result);
    }

    #[test]
    fn with_datas1() {
        let input = vec!["5 5", "1 3 N", "LFLFLFLFF", "2 3 E", "FFRFFRFRRF"];
        let reference = vec![
            Tondeuse {
                pos: Position {
                    x: 1,
                    y: 4,
                    orientation: Orientation::N,
                },
                mouvement: vec!['L', 'F', 'L', 'F', 'L', 'F', 'L', 'F', 'F'],
                max_x: 5,
                max_y: 5,
                order: 0,
            },
            Tondeuse {
                pos: Position {
                    x: 4,
                    y: 1,
                    orientation: Orientation::E,
                },
                mouvement: vec!['F', 'F', 'R', 'F', 'F', 'R', 'F', 'R', 'R', 'F'],
                max_x: 5,
                max_y: 5,
                order: 1,
            },
        ];

        let result = executor(input);
        assert_eq!(reference, result);
        
    }

    #[test]
    fn with_datas2() {
        let input = vec!["5 5", "1 0 N", "FFLFRFFF", "3 3 E", "LLFFFLFFFR"];
        let reference = vec![
            Tondeuse {
                pos: Position {
                    x: 0,
                    y: 5,
                    orientation: Orientation::N,
                },
                mouvement: vec!['F', 'F', 'L', 'F', 'R', 'F', 'F', 'F'],
                max_x: 5,
                max_y: 5,
                order: 0,
            },
            Tondeuse {
                pos: Position {
                    x: 0,
                    y: 0,
                    orientation: Orientation::W,
                },
                mouvement: vec!['L', 'L', 'F', 'F', 'F', 'L', 'F', 'F', 'F', 'R'],
                max_x: 5,
                max_y: 5,
                order: 1,
            },
        ];

        let result = executor(input);
        assert_eq!(reference, result);
        
    }
}
