use core::net;
use std::{
    env,
    sync::{Arc, Mutex},
    thread,
};

#[derive(Debug, Clone, Copy, Default)]
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
    fn parse_orientation(o: &str) -> Orientation {
        match o {
            "N" => Orientation::N,
            "E" => Orientation::E,
            "S" => Orientation::S,
            "W" => Orientation::W,
            //par defaut c'est le Nord ^^
            _ => Orientation::default(),
        }
    }
}

#[derive(Clone, Debug)]
struct Position {
    x: u8,
    y: u8,
    orientation: Orientation,
}

#[derive(Debug, Clone)]
pub(crate) struct Tondeuse {
    pos: Position,
    mouvement: Vec<char>,
    max_x: u8,
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
    fn run_async(self, pelouse: Arc<Mutex<Pelouse>>) -> thread::JoinHandle<Tondeuse> {
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
    occupied: Vec<(u8, u8)>,
}

impl Pelouse {
    fn is_free(&self, x: u8, y: u8) -> bool {
        //x >= 0 && y >= 0 &&
        x <= self.max_x && y <= self.max_y && !self.occupied.contains(&(x, y))
    }
    fn occupe(&mut self, x: u8, y: u8) {
        self.occupied.push((x, y));
    }
    fn libere(&mut self, x: u8, y: u8) {
        self.occupied.retain(|&(px, py)| px != x || py != y);
    }
}

fn get_initial_tondeuse(line: &str, mvt: &str, pelouse: Pelouse) -> Tondeuse {
    let mut infos = line.split_whitespace();
    let x = infos.next().unwrap().parse().unwrap_or_default();
    let y = infos.next().unwrap().parse().unwrap_or_default();
    let orientation = Orientation::parse_orientation(infos.next().unwrap());

    Tondeuse {
        mouvement: mvt.to_string().chars().collect(),
        pos: Position { x, y, orientation },
        max_x: pelouse.max_x,
        max_y: pelouse.max_y,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {println!("At least, one arg is needed : command file ! ")}
        2 => {
            let file: String = args[1].parse().unwrap_or_default();
            let binding = std::fs::read_to_string(file).expect("something wrong here !");
            let content: Vec<&str> = binding.lines().collect();

            //cas le taille max de le pelouse.
            let mut pelouse = Pelouse::default();
            let mut size_pelouse = content[0].split_whitespace();
            pelouse.max_x = size_pelouse.next().unwrap().parse().unwrap();
            pelouse.max_y = size_pelouse.next().unwrap().parse().unwrap();

            // gestion des tondeuses
            let mut all_tondeuses: Vec<Tondeuse> = Vec::new();

            for bloc in content[1..].chunks(2) {
                if bloc.len() == 2 {
                    let position = bloc[0];
                    let mouvements = bloc[1];
                    let tondeuse = get_initial_tondeuse(position, mouvements, pelouse.clone());

                    //on initalise les position de départ sur la pelouse
                    pelouse.occupe(tondeuse.pos.x, tondeuse.pos.y);

                    all_tondeuses.push(tondeuse);
                }
            }

            //println!("{:?}",tondeuses);
            let lawn = Arc::new(Mutex::new(pelouse));

            let mut all_th = Vec::new();
            // Lancement en parallèle
            for t in all_tondeuses {
                let t1 = t.run_async(lawn.clone());
                all_th.push(t1);
            }

            // Récupération des résultats
            for t in all_th {
                let tondeuse: Tondeuse = match t.join() {
                    Ok(t) => t,
                    Err(_) => todo!(),
                };
                println!(
                    "Tondeuse → {} {} {:?}",
                    tondeuse.pos.x, tondeuse.pos.y, tondeuse.pos.orientation
                );
            }
        }
        _ =>{ println!("Oups something wrong.")}
    }
    

}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn with_original_datas() {
        let input = r"5 5
    1 2 N
    LFLFLFLFF
    3 3 E
    FFRFFRFRRF";

        main();
    }
}
