use rand::Rng;
use rand::thread_rng;
use rand::seq::SliceRandom;
use std::vec::Vec;
use actix::prelude::*;
use crate::MyWs;

#[derive(Clone)]
pub struct Player{
    pub name: String,
    pub color: String,
    pub addr: Addr<MyWs>,
    pub ready: bool,
    pub flags: Vec<(usize, usize)>,
}

pub struct Minesweeper{
    map: Vec<Vec<u8>>,
    pub sessions: Vec<Player>,
    pub maxplayers: u8,
    pub vision: Vec<Vec<u8>>,
    pub map_size: usize,
    pub bombs: u8,
    pub gameover: bool,
    pub started: bool,
}


impl Minesweeper{
    pub fn new(map_size: usize, bombs: u8, maxplayers: u8) -> Self{
        let mut map = Vec::new(); 
        let mut vision = Vec::new(); 

        //adding 0's depending on the size of the map
        for y in 0..map_size{
            map.push(Vec::new());
            vision.push(Vec::new());
            for _x in 0..map_size{
                map[y].push(0);     
                vision[y].push(9);
            }
        }

        //generating map with bombs
        let mut generated = 0;
        let mut rng = rand::thread_rng();

        while generated < bombs{
            let x = rng.gen_range(0..map_size);
            let y = rng.gen_range(0..map_size);

            if map[y][x] == 0{
                map[y][x] = 1;
                generated += 1;
            }
        }

        Self{
            map: map,
            sessions: Vec::new(),
            maxplayers: maxplayers,
            gameover: false,
            vision: vision,
            map_size: map_size,
            bombs: bombs,
            started: false,
        }
    }

    pub fn gen_color(&self) -> String{
        let mut color_list = vec!(
            "#ff2919", //red
            "#ff6e19", //orange
            "#fffb19", //yellow
            "#34ff19", //green
            "#19ff98", //cyan
            "#1925ff", //darkblue
            "#8519ff", //violet
            "#ff19d9", //pink
            "#454545", //gray
            "white"); //white :)
        color_list.shuffle(&mut thread_rng());

        for color_item in color_list{
            let mut owned = false;

            for session in &self.sessions{
                if color_item == session.color {owned = true;} 
            }

            if !owned {return color_item.to_string();}
        }

        return "pink".to_string();
    }

    pub fn reset(&mut self){
        //adding 0's depending on the size of the map
        for y in 0..self.map_size{
            for x in 0..self.map_size{
                self.map[y][x] = 0;     
                self.vision[y][x] = 9;
            }
        }

        //generating map with bombs
        let mut generated = 0;
        let mut rng = rand::thread_rng();

        while generated < self.bombs{
            let x = rng.gen_range(0..self.map_size);
            let y = rng.gen_range(0..self.map_size);

            if self.map[y][x] == 0{
                self.map[y][x] = 1;
                generated += 1;
            }
        }

        for session in &mut self.sessions{
            session.flags.clear(); 
        }
        self.gameover = false;
    }

    pub fn place_flag(&mut self, x: usize, y: usize, name: String) -> bool{

        //if there is a flag then remove it and return
        //so that the flag wouldnt add up again
        if !self.gameover && self.vision[y][x] == 9{
            for session in &mut self.sessions{
                let mut idx = 0;
                for flag in session.flags.clone(){
                    if flag == (x, y){
                        session.flags.remove(idx);
                        return true;
                    }
                    idx += 1;
                }
            }

            //if there is no flag then place it
            for session in &mut self.sessions{
                if session.name == name{
                    session.flags.push((x, y));
                    return true;
                }
            }
        }
        false
    }

    pub fn is_flag(&self, x: usize, y: usize) -> bool{
        for session in &self.sessions{
            for flag in &session.flags{
                if flag == &(x, y) {return true;}
            }
        }
        false
    }

    //9 == unclicked / unknown cell
    //10 == mine that was stepped on
    //11 == mine that was not stepped on
    //12 == mine that was solved (flag placed on it)
    pub fn step_on(&mut self, x: usize, y: usize) -> &'static str{

        //return states:
        //moved (made legit move game goes on)
        //mine (clicked on mine)
        //not moving (clicked on known cell)
        if self.vision[y][x] == 9 && !self.gameover{

            //if it's a mine
            if self.map[y][x] == 1 {
                //after losing prevent from clicking the second time.
                self.gameover = true;
                
                //reveal other mines that we're not stepped on
                for y in 0..self.map.len(){
                    for x in 0..self.map.len(){
                        if self.map[y][x] == 1 {
                            self.vision[y][x] = 11;
                        }
                    }
                }

                //find solved mines
                for session in &self.sessions{
                    for flag in &session.flags{
                        if self.map[flag.1][flag.0] == 1{
                            self.vision[flag.1][flag.0] = 12; 
                        }
                    }
                }

                self.vision[y][x] = 10;

                return "mine";
            }

            //if it's a zero click on all nearby cells
            else if self.map[y][x] == 0 && self.nearby_mines(x, y) == 0{

                //make this as zero / clicked state
                self.vision[y][x] = 0;
                //top
                if y > 0 {self.step_on(x, y - 1);} 
                //left
                if x > 0 {self.step_on(x - 1, y);} 
                //bottom
                if y < self.map_size - 1 {self.step_on(x, y + 1);} 
                //right
                if x < self.map_size - 1 {self.step_on(x + 1, y);} 
                //top left
                if y > 0 && x > 0 {self.step_on(x - 1, y - 1);}
                //top right
                if y > 0 && x < self.map_size - 1 {self.step_on(x + 1, y - 1);}
                //bottom left
                if y < self.map_size - 1 && x > 0 {self.step_on(x - 1, y + 1);}
                //bottom right
                if y < self.map_size - 1 && x < self.map_size - 1{
                    self.step_on(x + 1, y + 1);
                }
            }

            //if at least one mine is touching then do simple step
            else {self.vision[y][x] = self.nearby_mines(x, y);}

            //remove flags on cell that are revealed
            for session in &mut self.sessions{
                let mut idx = 0;
                for flag in session.flags.clone(){
                    let mut removed = false;
                    if self.vision[flag.1][flag.0] != 9{
                        session.flags.remove(idx); 
                        removed = true;
                    }
                    if !removed {idx += 1;}
                }
            }
            return "moved";
        }
        return "not moving";
    }

    pub fn nearby_mines(&self, x: usize, y: usize) -> u8{
        let mut bf = 0;
        //top
        if y > 0 && self.map[y - 1][x] == 1 {bf += 1;} 
        //left
        if x > 0 && self.map[y][x - 1] == 1 {bf += 1;} 
        //bottom
        if y < self.map_size-1 && self.map[y + 1][x] == 1 {bf += 1;} 
        //right
        if x < self.map_size-1 && self.map[y][x + 1] == 1 {bf += 1;} 
        //top left
        if y > 0 && x > 0 && self.map[y - 1][x - 1] == 1 {bf += 1;}
        //top right
        if y > 0 && x < self.map_size-1 && self.map[y - 1][x + 1] == 1{bf += 1;}
        //bottom left
        if y < self.map_size-1 && x > 0 && self.map[y + 1][x - 1] == 1 {bf += 1;}
        //bottom right
        if y < self.map_size-1 && x < self.map_size-1 && self.map[y + 1][x + 1] 
            == 1 {bf += 1;}
        bf
    }

    pub fn get_game_state(&self) -> String{
        let mut string = "/gamestate".to_string();
        for y in 0..self.map_size{
            for x in 0..self.map_size{
                string.push_str(&format!(" {}", self.vision[y][x]));
            }
        }
        string
    }

    pub fn get_flags(&self) -> String{
        let mut string = "/flag".to_string();
        for session in &self.sessions{
            for flag in &session.flags{
                string.push_str(&format!(" {} {}", flag.0, flag.1));
            }
        }
        string
    }

    pub fn _println_debug(&self){
        println!("");
        for y in 0..self.map_size{
            for x in 0..self.map_size{
                if self.map[y][x] == 1 {print!("* ");}
                else {print!("{} ", self.nearby_mines(x, y));}
            }
            println!("");
        }
    }

    pub fn _println_vision(&self){
        println!("");
        for y in 0..self.map_size{
            for x in 0..self.map_size{
                if self.vision[y][x] == 9 {print!("# ");}
                else {print!("{} ", self.vision[y][x]);}
            }
            println!("");
        }
    }

}

