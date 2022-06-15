use actix::prelude::*;
use crate::minesweeper::Minesweeper;
use crate::minesweeper::Player;
use serde_json::json;
use crate::myws::MyWs;
use std::collections::HashSet;

pub struct Games{
    pub game_list: Vec<(String, Minesweeper)>,
    pub user_list: HashSet<(String, Addr<MyWs>)>,
}

impl Actor for Games{
    type Context = Context<Self>;
}

impl Games{
    pub fn new() -> Self {
        Self {
            game_list: Vec::new(),
            user_list: HashSet::new(),
        }
    }


    fn update_playerlist(&mut self, idx: usize){

        let mut playerlist = "/players".to_string();
        for session in &self.game_list[idx].1.sessions{
            playerlist.push_str(format!(" {} {}", session.name, session.color).as_str());
        }

        let mut idxx = 0;
        for session in &self.game_list[idx].1.sessions{

            if idxx == 0{
                session.addr.do_send(ServerMsg{
                    data: "/moderator".to_string()});
            }

            session.addr.do_send(ServerMsg{
                data: format!("/playernum {}/{}",
                              &self.game_list[idx].1.sessions.len(),
                              &self.game_list[idx].1.maxplayers)
            });

            session.addr.do_send(ServerMsg{
                data: playerlist.clone()});

            idxx += 1;
        }
    }

    fn update_rooms(&self){
        for user in &self.user_list{
            user.1.do_send(ServerMsg{data: "/updaterooms".to_string()});
        }
    }
}

pub struct GameList;
pub struct ConnectToGame{
    pub gameid: String,
    pub addr: Addr<MyWs>,
    pub username: String,
}

pub struct CreateRoom{
    pub gameid: String,
    pub username: String,
    pub addr: Addr<MyWs>,
    pub gamesize: u8,
    pub mines: u8,
    pub maxplayers: u8,
}

pub struct DisconnectFromGame{
    pub username: String,
    pub addr: Addr<MyWs>,
    pub gameid: String,
}
pub struct MakeMove{
    pub gameid: String,
    pub username: String,
    pub addr: Addr<MyWs>,
    pub x: usize,
    pub y: usize,
}
pub struct GameState{
    pub addr: Addr<MyWs>,
    pub username: String,
}

pub struct PlaceFlag{
    pub gameid: String,
    pub name: String,
    pub addr: Addr<MyWs>,
    pub x: usize,
    pub y: usize,
}

pub struct ResetGame{
    pub gameid: String,
}

pub struct Kick{
    pub gameid: String,
    pub username: String,
    pub target_username: String,
}

pub struct ChangeColor{
    pub username: String,
    pub gameid: String,
}

pub struct ServerMsg{pub data: String}
pub struct Online {pub username: String, pub addr: Addr<MyWs>}
pub struct Offline {pub username: String, pub addr: Addr<MyWs>}

impl Message for ChangeColor {type Result = ();}
impl Message for Kick {type Result = ();}
impl Message for ResetGame { type Result = (); }
impl Message for PlaceFlag{ type Result = (); }
impl Message for Online{ type Result = (); }
impl Message for Offline{ type Result = (); }
impl Message for CreateRoom{ type Result = (); }
impl Message for MakeMove{ type Result = (); }
impl Message for GameState{ type Result = (); }
impl Message for ServerMsg{ type Result = (); }
impl Message for DisconnectFromGame{ type Result = (); }
impl Message for GameList{type Result = Result<String, ()>;}
impl Message for ConnectToGame{ type Result = Result<String, ()>; }

impl Handler<ChangeColor> for Games{
    type Result = ();

    fn handle(&mut self, msg: ChangeColor, _ctx: &mut Context<Self>){
        let mut idx = 0;
        for game in &mut self.game_list{
            if msg.gameid == game.0{

                let color = game.1.gen_color();

                for session in &mut game.1.sessions{
                    if session.name == msg.username && game.1.gameover{
                        session.color = color;
                        self.update_playerlist(idx);
                        return
                    }
                }
            }
            idx += 1;
        }
    }
}

impl Handler<Kick> for Games{
    type Result = ();

    fn handle(&mut self, msg: Kick, _ctx: &mut Context<Self>){
        let mut idx = 0;
        for game in &mut self.game_list{
            if msg.gameid == game.0 && msg.username == game.1.sessions[0].name{
                let mut idxx = 0;
                for session in game.1.sessions.clone(){
                    //find users index and remove the user
                    if session.name == msg.target_username{
                        session.addr.do_send(ServerMsg{data: "/quitedroom".to_string()});
                        game.1.sessions.remove(idxx);

                        //if room is empty remove it and then stop the operation
                        if game.1.sessions.len() == 0{
                            self.game_list.remove(idx);
                            self.update_rooms();
                            return
                        }
                    }
                    idxx += 1;
                }

                for session in game.1.sessions.clone(){
                    session.addr.do_send(ServerMsg{data: game.1.get_game_state()});
                    session.addr.do_send(ServerMsg{data: game.1.get_flags()});
                }

                self.update_playerlist(idx);
                self.update_rooms();
                return
            }
            idx += 1;
        }
    }
}

impl Handler<PlaceFlag> for Games{
    type Result = ();

    fn handle(&mut self, msg: PlaceFlag, _ctx: &mut Context<Self>){
        for game in &mut self.game_list{
            if game.0 == msg.gameid && game.1.gameover == false{
                let result = game.1.place_flag(msg.x, msg.y, msg.name.clone());
                if result{
                    for session in &game.1.sessions{
                        if session.name == msg.name{
                            for sessionn in &game.1.sessions{
                                sessionn.addr.do_send(ServerMsg{data: game.1.get_flags()});
                                sessionn.addr.do_send(ServerMsg{
                                    data: format!("/moved {} {} {}", msg.x, msg.y, session.color)
                                });
                            }
                            return
                        }
                    }
                }
                return
            }
        }
    }
}

impl Handler<ResetGame> for Games{
    type Result = ();

    fn handle(&mut self, msg: ResetGame, _ctx: &mut Context<Self>){
        for game in &mut self.game_list{
            if game.0 == msg.gameid{
                if game.1.gameover{
                    game.1.reset();
                    for session in &game.1.sessions{
                        session.addr.do_send(ServerMsg{data: "/resetgame".to_string()});
                        session.addr.do_send(ServerMsg{data: game.1.get_game_state()});
                    }
                }
            }
        }
    }
}

impl Handler<CreateRoom> for Games{
    type Result = ();

    fn handle(&mut self, msg: CreateRoom, _ctx: &mut Context<Self>){
        self.game_list.push((msg.gameid.clone(), Minesweeper::new(msg.gamesize as usize, msg.mines, msg.maxplayers)));
        //disconnects from rooms before connecting to new
        for game in &mut self.game_list{
            let mut idx = 0;
            for session in game.1.sessions.clone(){
                if session.name == msg.username.clone(){
                    game.1.sessions.remove(idx);
                    msg.addr.do_send(ServerMsg{data: "/quitedroom".to_string()});
                }
                idx += 1;
            }
        }


        //puts user to room
        for game in &mut self.game_list{
            if game.0 == msg.gameid{
                let color = game.1.gen_color();
                game.1.sessions.push(Player {
                    name: msg.username.clone(),
                    color: color,
                    addr: msg.addr.clone(),
                    ready: false,
                    flags: Vec::new(),
                });

                msg.addr.do_send(ServerMsg{
                    data: format!("/joinedroom {}", game.1.map_size).to_string()});
                msg.addr.do_send(ServerMsg{data: game.1.get_game_state()});

                self.update_rooms();
                break

            }
        }

        let mut idx = 0;
        for game in &mut self.game_list{
            if game.0 == msg.gameid{
                self.update_playerlist(idx);
                return
            }
            idx += 1;
        }

    }
}

impl Handler<Online> for Games{
    type Result = ();

    fn handle(&mut self, msg: Online, _ctx: &mut Context<Self>){
        self.user_list.insert((msg.username.clone(), msg.addr.clone()));
        msg.addr.do_send(ServerMsg{data: format!("/online {}", msg.username)});
    }
}

impl Handler<Offline> for Games{
    type Result = ();

    fn handle(&mut self, msg: Offline, _ctx: &mut Context<Self>){
        self.user_list.remove(&(msg.username, msg.addr));
    }
}

impl Handler<GameState> for Games{
    type Result = ();

    fn handle(&mut self, msg: GameState, _ctx: &mut Context<Self>){
        for game in &mut self.game_list{
            for session in game.1.sessions.clone(){
                if session.name == msg.username.clone(){
                    session.addr.do_send(ServerMsg{data: game.1.get_game_state()});
                }    
            }
        }
    }
}

impl Handler<MakeMove> for Games{
    type Result = ();

    fn handle(&mut self, msg: MakeMove, _ctx: &mut Context<Self>){
        for game in &mut self.game_list{
            if game.0 == msg.gameid && !game.1.is_flag(msg.x.clone(), msg.y.clone()){
                for session in game.1.sessions.clone(){
                    if session.name == msg.username{

                        let result = game.1.step_on(msg.x, msg.y);
                        if result == "moved"{
                            for sessionn in &game.1.sessions{
                                sessionn.addr.do_send(ServerMsg{data: game.1.get_game_state()});
                                sessionn.addr.do_send(ServerMsg{
                                    data: format!("/moved {} {} {}", msg.x, msg.y, session.color)
                                });
                            }
                        }

                        else if result == "mine"{
                            for sessionn in &game.1.sessions{
                                sessionn.addr.do_send(ServerMsg{data: game.1.get_game_state()});
                                sessionn.addr.do_send(ServerMsg{
                                    data: format!("/moved {} {} {}", msg.x, msg.y, session.color)
                                });
                                sessionn.addr.do_send(ServerMsg{data: format!("/gameover {}", msg.username)});
                            }
                        }

                        //TODO: gamewin event
                        //else if result == "win"{
                        //    for sessionn in &game.1.sessions{
                        //        sessionn.addr.do_send(ServerMsg{data: game.1.get_game_state()});
                        //        sessionn.addr.do_send(ServerMsg{data: "/gamewin".to_string()});
                        //    }
                        //}
                    }
                }
            }
        }
    }
}

impl Handler<ConnectToGame> for Games{
    type Result = Result<String, ()>;

    fn handle(&mut self, msg: ConnectToGame, _ctx: &mut Context<Self>) -> Self::Result{

        //disconnects from rooms before connecting to new
        for game in &mut self.game_list{
            let mut idx = 0;
            for session in game.1.sessions.clone(){
                if session.name == msg.username.clone(){
                    game.1.sessions.remove(idx);
                    msg.addr.do_send(ServerMsg{data: "/quitedroom".to_string()});
                }
                idx += 1;
            }
        }

        //puts user to room if sessions < maxplayers
        let mut idx = 0;
        for game in &mut self.game_list{
            if game.0 == msg.gameid && game.1.sessions.len() < game.1.maxplayers as usize{
                let color = game.1.gen_color();
                game.1.sessions.push(Player {
                    name: msg.username,
                    color: color,
                    addr: msg.addr.clone(),
                    ready: false,
                    flags: Vec::new(),
                });


                msg.addr.do_send(ServerMsg{
                    data: format!("/joinedroom {}", game.1.map_size).to_string()});
                msg.addr.do_send(ServerMsg{data: game.1.get_game_state()});

                if game.1.gameover{
                    msg.addr.do_send(ServerMsg{data: "/gameover".to_string()});
                }
                else{
                    msg.addr.do_send(ServerMsg{data: game.1.get_flags()});
                }
                self.update_rooms();
                self.update_playerlist(idx);
                return Ok("connected".to_string());
            }
            idx += 1;
        }
        Err(())

    }
}

impl Handler<DisconnectFromGame> for Games{
    type Result = ();    

    fn handle(&mut self, msg: DisconnectFromGame, _ctx: &mut Context<Self>){
        //changed to hashset from vector so change the logic
        let mut idx = 0;
        for game in &mut self.game_list{
            let mut idxx = 0;
            if game.0 == msg.gameid{

                for session in game.1.sessions.clone(){
                    //find users index and remove the user
                    if session.name == msg.username.clone(){
                        game.1.sessions.remove(idxx);
                        msg.addr.do_send(ServerMsg{data: "/quitedroom".to_string()});

                        //if room is empty remove it and then stop the operation
                        if game.1.sessions.len() == 0{
                            self.game_list.remove(idx);
                            self.update_rooms();
                            return
                        }
                    }
                    idxx += 1;
                }
                for session in game.1.sessions.clone(){
                    session.addr.do_send(ServerMsg{data: game.1.get_game_state()});
                    session.addr.do_send(ServerMsg{data: game.1.get_flags()});
                }

                self.update_playerlist(idx);
                self.update_rooms();
                return
            }
            idx += 1; 
        }
    }
}

impl Handler<GameList> for Games{
    type Result = Result<String, ()>;

    fn handle(&mut self, _msg: GameList, _ctx: &mut Context<Self>) -> Self::Result{
        let mut vector: Vec<(String, String, String)> = Vec::new();
        for game in &mut self.game_list{
            vector.push((
                    game.0.clone(), //roomname
                    game.1.sessions.len().to_string(), //player num
                    game.1.maxplayers.to_string(), //maxplayers
            ));
        }
        Ok(json!({"games": vector}).to_string())
    }
}




